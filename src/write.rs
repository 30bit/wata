use crate::{ReadConfig, WriteConfig};
use image::{codecs::webp::WebPEncoder, Rgba, RgbaImage};
use std::{
    io::{Seek, Write},
    mem::size_of,
};
use zip::{write::FileOptions, ZipWriter};

const DEFAULT_COMPRESSION_METHOD: zip::CompressionMethod = zip::CompressionMethod::DEFLATE;

const ZIP_LARGE_FILE_SIZE: usize = 4 * usize::pow(10, 9) * size_of::<u8>();

/// # Errors
///
/// - if any serialization operation fails
/// - if image is not divisible by configured `frame_height`
pub fn write<W>(writer: W, frames: &RgbaImage, config: &WriteConfig) -> anyhow::Result<()>
where
    W: Write + Seek,
{
    let (full_width, full_height) = frames.dimensions();
    check_grid(full_height, config.frame_height)?;
    let mut zip = ZipWriter::new(writer);
    let options = FileOptions::default().compression_method(DEFAULT_COMPRESSION_METHOD);

    let is_img_large =
        (full_width as usize * full_height as usize * size_of::<Rgba<u8>>()) >= ZIP_LARGE_FILE_SIZE;
    zip.start_file("frames.webp", options.large_file(is_img_large))?;
    frames.write_with_encoder(WebPEncoder::new_lossless(&mut zip))?;

    zip.start_file("wata.toml", options)?;
    zip.write_all(
        toml::to_string_pretty(&ReadConfig {
            num_frames: full_height / config.frame_height,
            is_srgb: config.is_srgb,
        })?
        .as_bytes(),
    )?;

    zip.finish()?;
    Ok(())
}

fn check_grid(full_height: u32, frame_height: u32) -> anyhow::Result<()> {
    if full_height % frame_height != 0 {
        anyhow::bail!(
            "cannot split image of height {full_height}px into frames of height {frame_height}px",
        );
    }
    Ok(())
}
