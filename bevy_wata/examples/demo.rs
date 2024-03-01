#![allow(clippy::needless_pass_by_value)]

use bevy::prelude::*;
use bevy::{asset::LoadState, window::PrimaryWindow};
use bevy_rand::prelude::*;
use bevy_wata::{Wata, WataPlayer, WataPlugin};
use rand::seq::SliceRandom as _;

const RANDOM_SEED: [u8; 8] = u64::to_le_bytes(914_391_251);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, States, Default)]
pub enum GameState {
    #[default]
    Loading,
    Running,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EntropyPlugin::<WyRand>::with_seed(RANDOM_SEED),
            WataPlugin,
        ))
        .init_state::<GameState>()
        .add_systems(
            Startup,
            (spawn_wata_load, set_window_cursor(CursorIcon::Wait)),
        )
        .add_systems(
            OnEnter(GameState::Running),
            (
                spawn_camera,
                spawn_wata_players,
                set_window_cursor(CursorIcon::Default),
            ),
        )
        .add_systems(Update, (play_wata_frames, block_on_wata_load).chain())
        .run();
}

fn spawn_wata_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_batch([
        asset_server.load::<Wata>("eyes.wata"),
        asset_server.load::<Wata>("tongue.wata"),
    ]);
}

fn set_window_cursor(icon: CursorIcon) -> impl Fn(Query<&mut Window, With<PrimaryWindow>>) {
    move |mut q| {
        for mut window in q.iter_mut() {
            window.cursor.icon = icon;
        }
    }
}

#[allow(clippy::cast_precision_loss)]
fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle {
        // projection: OrthographicProjection {
        //     scaling_mode: camera::ScalingMode::FixedHorizontal(
        //         (WATA_PLAYERS_GRID.x * WATA_SETTINGS.width) as f32,
        //     ),
        //     viewport_origin: Vec2::Y,
        //     ..default()
        // },
        ..default()
    },));
}

#[allow(clippy::cast_precision_loss)]
fn spawn_wata_players(
    mut commands: Commands,
    handles: Query<&Handle<Wata>>,
    assets: Res<Assets<Wata>>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    let handles: Vec<_> = handles.iter().collect();

    let &handle = handles
        .choose(&mut *rng)
        .expect("at least one `Wata` asset");
    let player = WataPlayer::new(handle.clone());
    let frame = player
        .get_frame(&assets)
        .expect("every `Wata` loaded")
        .expect("at least one frame");
    let texture = assets
        .get(handle)
        .expect("every `Wata` loaded")
        .texture
        .clone();
    let sprite = SpriteBundle {
        sprite: Sprite {
            rect: Some(frame.as_rect()),
            ..default()
        },
        texture,
        ..default()
    };

    commands.spawn((player, sprite));
}

fn block_on_wata_load(
    q: Query<&Handle<Wata>>,
    asset_server: Res<AssetServer>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if *state == GameState::Running {
        return;
    }
    let loaded = q
        .iter()
        .all(|handle| asset_server.load_state(handle) == LoadState::Loaded);
    if loaded {
        next_state.set(GameState::Running);
    }
}

fn play_wata_frames(
    mut q: Query<(&mut WataPlayer, &mut Sprite)>,
    wata: Res<Assets<Wata>>,
    state: Res<State<GameState>>,
) {
    if *state == GameState::Loading {
        return;
    }
    for (mut play, mut sprite) in &mut q {
        let frame = play
            .get_frame(&wata)
            .expect("every `Wata` loaded")
            .unwrap_or_else(|| {
                play.frame_index = 0;
                play.get_frame(&wata)
                    .expect("every `Wata` loaded")
                    .expect("at least one frame")
            });
        sprite.rect = Some(frame.as_rect());
        play.frame_index += 1;
    }
}
