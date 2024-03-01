use std::iter;

use anyhow::anyhow;
use image::RgbaImage;
use openh264::{formats::YUVSource, OpenH264API};

/// # Errors
///
/// - if `h264` decoding fails
/// - if provided `frame_height`x`frame_width` can't fit a frame
pub fn make(frame_width: u32, frame_height: u32, h264_video: &[u8]) -> anyhow::Result<RgbaImage> {
    let mut decoder = openh264::decoder::Decoder::new(OpenH264API::from_source())?;
    let mut rgba8_buf: Vec<u8> = vec![];
    let mut num_nal_units = 0;
    for packet in openh264::nal_units(h264_video) {
        if let Some(yuv) = decoder.decode(packet)? {
            let start = rgba8_buf.len();
            rgba8_buf.extend(iter::repeat(0).take(yuv_pixel_count(&yuv)));
            yuv.write_rgba8(&mut rgba8_buf[start..]);
            num_nal_units += 1;
        } else {
            log::warn!("missed frame");
        }
    }
    let full_height = frame_height * num_nal_units;
    RgbaImage::from_vec(frame_width, full_height, rgba8_buf).ok_or_else(|| {
        anyhow!("invalid dimensions: can't fit a frame into {frame_width}x{full_height}")
    })
}

#[allow(clippy::cast_sign_loss)]
fn yuv_pixel_count(yuv: &impl YUVSource) -> usize {
    yuv.width() as usize * yuv.height() as usize * 4
}
