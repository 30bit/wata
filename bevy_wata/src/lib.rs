pub use wata;

use anyhow::anyhow;
use bevy::prelude::*;
use bevy::{
    asset::{AssetLoader, AsyncReadExt as _},
    render::render_asset::RenderAssetUsages,
};
use std::io;

#[derive(Clone, Debug, Default, TypePath, Asset)]
pub struct Wata {
    #[dependency]
    pub texture: Handle<Image>,
    pub frame_width: u32,
    pub frame_height: u32,
    pub num_frames: u32,
}

impl Wata {
    pub fn frame(&self, frame_index: u32) -> URect {
        let min_y = self.frame_height * frame_index;
        let max_y = min_y + self.frame_height;
        URect {
            min: UVec2::new(0, min_y),
            max: UVec2::new(self.frame_width, max_y),
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct WataLoader;

impl AssetLoader for WataLoader {
    type Asset = Wata;
    type Settings = ();
    type Error = anyhow::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).await?;
            let (config, img) = wata::read(io::Cursor::new(&buf))?;
            let (frame_width, full_height) = img.dimensions();
            let frame_height = full_height / config.num_frames;
            let texture = load_context.add_labeled_asset(
                "texture".into(),
                Image::from_dynamic(
                    img.into(),
                    config.is_srgb.unwrap_or_default(),
                    RenderAssetUsages::all(),
                ),
            );
            Ok(Wata {
                texture,
                frame_width,
                frame_height,
                num_frames: config.num_frames,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["zip", "wata"]
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct WataPlugin;

impl Plugin for WataPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Wata>();
        app.init_asset_loader::<WataLoader>();
    }
}

#[derive(Clone, Debug, Component, Default)]
pub struct WataPlayer {
    pub asset: Handle<Wata>,
    pub frame_index: u32,
}

impl WataPlayer {
    #[must_use]
    #[inline]
    pub fn new(asset: Handle<Wata>) -> Self {
        Self {
            asset,
            frame_index: 0,
        }
    }

    /// # Errors
    ///
    /// - [`Err`] if [`Wata`] can't be found in [`Assets`]
    /// - [`None`] if there are no more frames to show
    pub fn get_frame(&self, wata: &Assets<Wata>) -> anyhow::Result<Option<URect>> {
        let asset = wata
            .get(&self.asset)
            .ok_or_else(|| anyhow!("`Wata` asset not found"))?;
        if self.frame_index >= asset.num_frames {
            return Ok(None);
        }
        Ok(Some(asset.frame(self.frame_index)))
    }
}
