use crate::ReadConfig;
use anyhow::anyhow;
use image::{codecs::webp::WebPDecoder, ImageDecoder as _, RgbImage};
use std::io::{Read, Seek};
use zip::read::ZipArchive;

/// # Errors
///
/// - if any deserialization operation fails
/// - if read image is not divisible by read `num_frames`
pub fn read<R>(reader: R) -> anyhow::Result<(ReadConfig, RgbImage)>
where
    R: Read + Seek,
{
    let mut reader = ZipArchive::new(reader)?;
    let config: ReadConfig = {
        let mut config_buf = String::new();
        let mut config_file = reader.by_name("wata.toml")?;
        config_file.read_to_string(&mut config_buf)?;
        toml::from_str(&config_buf)?
    };
    let img = {
        let img_decoder = WebPDecoder::new(reader.by_name("frames.webp")?)?;
        let (full_width, full_height) = img_decoder.dimensions();
        check_grid(full_height, config.num_frames)?;
        let mut img_buf = vec![0; usize::try_from(img_decoder.total_bytes())?];
        img_decoder.read_image(&mut img_buf)?;
        RgbImage::from_vec(full_width, full_height, img_buf).ok_or_else(|| {
            anyhow!("invalid encoding: can't fit frames into {full_width}x{full_height}")
        })?
    };
    Ok((config, img))
}

fn check_grid(full_height: u32, num_frames: u32) -> anyhow::Result<()> {
    if full_height % num_frames != 0 {
        anyhow::bail!("cannot split image of height {full_height}px into {num_frames} frames",);
    }
    Ok(())
}
