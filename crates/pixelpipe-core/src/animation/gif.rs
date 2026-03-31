use crate::error::{PipelineError, Result};
use gif::{Encoder, Frame, Repeat};
use image::RgbaImage;

/// Encode a sequence of RGBA frames into an animated GIF.
///
/// All frames must have the same dimensions.
/// `delays` is in milliseconds per frame.
/// Returns raw GIF bytes.
pub fn encode_gif(frames: &[RgbaImage], delays: &[u32], loop_anim: bool) -> Result<Vec<u8>> {
    if frames.is_empty() {
        return Err(PipelineError::Animation(
            "Cannot encode GIF with zero frames".to_string(),
        ));
    }

    let width = frames[0].width() as u16;
    let height = frames[0].height() as u16;

    let mut buf = Vec::new();
    {
        let mut encoder = Encoder::new(&mut buf, width, height, &[])
            .map_err(|e| PipelineError::Animation(format!("GIF encoder init failed: {}", e)))?;

        if loop_anim {
            encoder
                .set_repeat(Repeat::Infinite)
                .map_err(|e| PipelineError::Animation(format!("GIF set repeat failed: {}", e)))?;
        }

        for (i, frame_img) in frames.iter().enumerate() {
            // GIF delay is in centiseconds (1/100th of a second)
            let delay_cs = (delays.get(i).copied().unwrap_or(100) / 10).max(1) as u16;

            // Convert RGBA to indexed color using the gif crate's built-in quantization
            let mut rgba_pixels: Vec<u8> = frame_img.as_raw().to_vec();

            let mut gif_frame =
                Frame::from_rgba_speed(width, height, &mut rgba_pixels, 10);
            gif_frame.delay = delay_cs;
            gif_frame.dispose = gif::DisposalMethod::Background;

            encoder.write_frame(&gif_frame).map_err(|e| {
                PipelineError::Animation(format!("GIF write frame {} failed: {}", i, e))
            })?;
        }
    }

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn solid_frame(w: u32, h: u32, color: [u8; 4]) -> RgbaImage {
        let mut img = RgbaImage::new(w, h);
        for pixel in img.pixels_mut() {
            *pixel = Rgba(color);
        }
        img
    }

    #[test]
    fn test_encode_single_frame() {
        let frames = vec![solid_frame(8, 8, [255, 0, 0, 255])];
        let delays = vec![100];
        let gif_data = encode_gif(&frames, &delays, true).unwrap();

        // GIF starts with "GIF89a" magic
        assert_eq!(&gif_data[0..6], b"GIF89a");
        assert!(gif_data.len() > 20);
    }

    #[test]
    fn test_encode_multiple_frames() {
        let frames = vec![
            solid_frame(4, 4, [255, 0, 0, 255]),
            solid_frame(4, 4, [0, 255, 0, 255]),
            solid_frame(4, 4, [0, 0, 255, 255]),
        ];
        let delays = vec![100, 100, 100];
        let gif_data = encode_gif(&frames, &delays, true).unwrap();

        assert_eq!(&gif_data[0..6], b"GIF89a");
        // Multi-frame GIF should be larger than single frame
        assert!(gif_data.len() > 50);
    }

    #[test]
    fn test_encode_empty_fails() {
        let result = encode_gif(&[], &[], true);
        assert!(result.is_err());
    }
}
