#![allow(clippy::needless_pass_by_value)]

use bevy::prelude::*;
use bevy::{asset::LoadState, window::PrimaryWindow};
use bevy_rand::prelude::*;
use bevy_wata::{Wata, WataPlayer, Plugin};
use rand::{seq::SliceRandom as _, Rng};
use std::{f32::consts::PI, iter};

const NUM_PLAYERS: u32 = 4100;

const SCATTER_RADIUS: f32 = 410.0;

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
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Wata Demo (v0.1.1)".to_string(),
                    ..default()
                }),
                ..default()
            }),
            EntropyPlugin::<WyRand>::with_seed(RANDOM_SEED),
            Plugin,
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
                spawn_wata_players_grid,
                set_window_cursor(CursorIcon::Default),
            ),
        )
        .add_systems(Update, (play_wata_frames, block_on_wata_load).chain())
        .run();
}

fn spawn_wata_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_batch([
        asset_server.load::<Wata>("butterfly.wata"),
        asset_server.load::<Wata>("eyes.wata"),
        asset_server.load::<Wata>("neck.wata"),
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

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_wata_players_grid(
    mut commands: Commands,
    handles: Query<&Handle<Wata>>,
    assets: Res<Assets<Wata>>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    let handles: Vec<_> = handles.iter().collect();

    let batch = iter::repeat_with(|| {
        let &handle = handles
            .choose(&mut *rng)
            .expect("at least one `Wata` asset");
        wata_player_bundle(handle.clone(), &assets, &mut rng)
    })
    .take(NUM_PLAYERS as usize);
    for bundle in batch {
        commands.spawn(bundle);
    }
}

fn wata_player_bundle(
    handle: Handle<Wata>,
    assets: &Assets<Wata>,
    rng: &mut GlobalEntropy<WyRand>,
) -> (WataPlayer, SpriteBundle) {
    let texture = assets
        .get(&handle)
        .expect("every `Wata` loaded")
        .texture
        .clone();
    let num_frames = assets.get(&handle).expect("every `Wata` loaded").num_frames;
    let player = WataPlayer {
        asset: handle,
        frame_index: rng.gen_range(0..num_frames),
    };
    let frame = player
        .get_frame(&assets)
        .expect("every `Wata` loaded")
        .expect("at least one frame");
    let sprite = SpriteBundle {
        sprite: Sprite {
            rect: Some(frame.as_rect()),
            custom_size: Some(Vec2::new(64.0, 36.0)),
            ..default()
        },
        texture,
        transform: {
            let radius = SCATTER_RADIUS * f32::sqrt(rng.gen_range(0.0..1.0));
            let angle = rng.gen_range(0.0..2.0 * PI);
            Transform::from_xyz(
                angle.cos() * radius,
                angle.sin() * radius,
                rng.gen_range(0.0..1.0),
            )
        },
        ..default()
    };
    (player, sprite)
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
    mut q: Query<(&mut WataPlayer, &mut Transform, &mut Sprite)>,
    wata: Res<Assets<Wata>>,
    state: Res<State<GameState>>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    if *state == GameState::Loading {
        return;
    }
    for (mut play, mut transform, mut sprite) in &mut q {
        let frame = play
            .get_frame(&wata)
            .expect("every `Wata` loaded")
            .unwrap_or_else(|| {
                play.frame_index = 0;
                transform.translation.z = rng.gen_range(0.0..1.0);
                play.get_frame(&wata)
                    .expect("every `Wata` loaded")
                    .expect("at least one frame")
            });
        sprite.rect = Some(frame.as_rect());
        play.frame_index += 1;
    }
}
