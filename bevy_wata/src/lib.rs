pub use wata;

use anyhow::anyhow;
use bevy::prelude::*;
use bevy::{
    asset::{AssetLoader, AsyncReadExt as _},
    render::render_asset::RenderAssetUsages,
};
use image::{DynamicImage, GenericImageView as _};
use std::io;

#[derive(Copy, Clone, Debug, Default)]
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Wata>()
            .init_asset::<FlatWata>()
            .init_asset_loader::<WataLoader>()
            .init_asset_loader::<FlatWataLoader>();
    }
}

#[derive(Clone, Debug, Default, TypePath, Asset)]
pub struct FlatWata {
    #[dependency]
    pub textures: Vec<Handle<Image>>,
    pub frame_dimensions: UVec2,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct FlatWataLoader;

impl AssetLoader for FlatWataLoader {
    type Asset = FlatWata;
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

            let textures = (0..config.num_frames)
                .map(|frame_index| {
                    let view = DynamicImage::from(
                        img.view(0, frame_index * frame_height, frame_width, frame_height)
                            .to_image(),
                    );
                    let texture = load_context.add_labeled_asset(
                        format!("texture-{frame_index}"),
                        Image::from_dynamic(
                            view,
                            config.is_srgb.unwrap_or_default(),
                            RenderAssetUsages::all(),
                        ),
                    );
                    texture
                })
                .collect();

            Ok(FlatWata {
                textures,
                frame_dimensions: UVec2::new(frame_width, frame_height),
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["zip", "wata"]
    }
}

#[derive(Clone, Debug, Component, Default)]
pub struct FlatWataPlayer {
    pub asset: Handle<FlatWata>,
    pub frame_index: u32,
}

impl FlatWataPlayer {
    #[must_use]
    #[inline]
    pub fn new(asset: Handle<FlatWata>) -> Self {
        Self {
            asset,
            frame_index: 0,
        }
    }

    /// # Errors
    ///
    /// - [`Err`] if [`FlatWata`] can't be found in [`Assets`]
    /// - [`None`] if there are no more frames to show
    pub fn get_frame(
        &self,
        wata: &Assets<FlatWata>,
    ) -> anyhow::Result<Option<(URect, Handle<Image>)>> {
        let asset = wata
            .get(&self.asset)
            .ok_or_else(|| anyhow!("`FlatWata` asset not found"))?;
        let Some(texture) = asset.textures.get(self.frame_index as usize) else {
            return Ok(None);
        };
        Ok(Some((
            URect::from_corners(UVec2::ZERO, asset.frame_dimensions),
            texture.clone(),
        )))
    }

    /// # Errors
    ///
    /// - [`Err`] if [`FlatWata`] can't be found in [`Assets`]
    pub fn advance(
        &mut self,
        n: u32,
        looped: bool,
        wata: &Assets<FlatWata>,
    ) -> anyhow::Result<bool> {
        let asset = wata
            .get(&self.asset)
            .ok_or_else(|| anyhow!("`FlatWata` asset not found"))?;
        let mut new_frame_index = self.frame_index + n;
        if new_frame_index as usize >= asset.textures.len() {
            if looped {
                new_frame_index -= asset.textures.len() as u32;
            } else {
                return Ok(false);
            }
        }
        self.frame_index = new_frame_index;
        Ok(true)
    }
}

#[derive(Clone, Debug, Default, TypePath, Asset)]
pub struct Wata {
    #[dependency]
    pub texture: Handle<Image>,
    pub frame_dimensions: UVec2,
    pub num_frames: u32,
}

impl Wata {
    pub fn frame(&self, frame_index: u32) -> URect {
        let min_y = self.frame_dimensions.y * frame_index;
        let max_y = min_y + self.frame_dimensions.y;
        URect {
            min: UVec2::new(0, min_y),
            max: UVec2::new(self.frame_dimensions.x, max_y),
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
                frame_dimensions: UVec2::new(frame_width, frame_height),
                num_frames: config.num_frames,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["zip", "wata"]
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
