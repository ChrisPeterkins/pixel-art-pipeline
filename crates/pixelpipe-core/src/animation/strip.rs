use crate::config::schema::StripDirection;
use crate::error::{PipelineError, Result};
use image::RgbaImage;
use serde::Serialize;

/// Composite frames into a horizontal or vertical sprite strip.
pub fn build_strip(
    frames: &[RgbaImage],
    direction: &StripDirection,
) -> Result<RgbaImage> {
    if frames.is_empty() {
        return Err(PipelineError::Animation(
            "Cannot build strip with zero frames".to_string(),
        ));
    }

    let frame_w = frames[0].width();
    let frame_h = frames[0].height();

    let (strip_w, strip_h) = match direction {
        StripDirection::Horizontal => (frame_w * frames.len() as u32, frame_h),
        StripDirection::Vertical => (frame_w, frame_h * frames.len() as u32),
    };

    let mut strip = RgbaImage::new(strip_w, strip_h);

    for (i, frame) in frames.iter().enumerate() {
        let (x, y) = match direction {
            StripDirection::Horizontal => (frame_w * i as u32, 0),
            StripDirection::Vertical => (0, frame_h * i as u32),
        };
        image::imageops::overlay(&mut strip, frame, x as i64, y as i64);
    }

    Ok(strip)
}

/// Metadata for a sprite strip animation.
#[derive(Debug, Serialize)]
pub struct StripMetadata {
    pub image: String,
    pub frame_width: u32,
    pub frame_height: u32,
    pub frame_count: u32,
    pub direction: String,
    pub timing: TimingMetadata,
    pub frames: Vec<FrameMetadata>,
}

#[derive(Debug, Serialize)]
pub struct TimingMetadata {
    pub mode: String,
    pub frame_duration_ms: Option<u32>,
    pub total_duration_ms: u32,
}

#[derive(Debug, Serialize)]
pub struct FrameMetadata {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub duration_ms: u32,
}

/// Generate strip metadata JSON.
pub fn build_metadata(
    image_filename: &str,
    frame_w: u32,
    frame_h: u32,
    frame_count: u32,
    direction: &StripDirection,
    delays: &[u32],
) -> StripMetadata {
    let uniform = delays.windows(2).all(|w| w[0] == w[1]);
    let total: u32 = delays.iter().sum();

    let dir_str = match direction {
        StripDirection::Horizontal => "horizontal",
        StripDirection::Vertical => "vertical",
    };

    let frames: Vec<FrameMetadata> = (0..frame_count)
        .map(|i| {
            let (x, y) = match direction {
                StripDirection::Horizontal => (frame_w * i, 0),
                StripDirection::Vertical => (0, frame_h * i),
            };
            FrameMetadata {
                x,
                y,
                w: frame_w,
                h: frame_h,
                duration_ms: delays.get(i as usize).copied().unwrap_or(100),
            }
        })
        .collect();

    StripMetadata {
        image: image_filename.to_string(),
        frame_width: frame_w,
        frame_height: frame_h,
        frame_count,
        direction: dir_str.to_string(),
        timing: TimingMetadata {
            mode: if uniform {
                "uniform".to_string()
            } else {
                "variable".to_string()
            },
            frame_duration_ms: if uniform { delays.first().copied() } else { None },
            total_duration_ms: total,
        },
        frames,
    }
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
    fn test_horizontal_strip() {
        let frames = vec![
            solid_frame(8, 8, [255, 0, 0, 255]),
            solid_frame(8, 8, [0, 255, 0, 255]),
            solid_frame(8, 8, [0, 0, 255, 255]),
        ];

        let strip = build_strip(&frames, &StripDirection::Horizontal).unwrap();
        assert_eq!(strip.width(), 24); // 8 * 3
        assert_eq!(strip.height(), 8);

        // First frame region should be red
        assert_eq!(strip.get_pixel(0, 0).0, [255, 0, 0, 255]);
        // Second frame region should be green
        assert_eq!(strip.get_pixel(8, 0).0, [0, 255, 0, 255]);
        // Third frame region should be blue
        assert_eq!(strip.get_pixel(16, 0).0, [0, 0, 255, 255]);
    }

    #[test]
    fn test_vertical_strip() {
        let frames = vec![
            solid_frame(8, 8, [255, 0, 0, 255]),
            solid_frame(8, 8, [0, 255, 0, 255]),
        ];

        let strip = build_strip(&frames, &StripDirection::Vertical).unwrap();
        assert_eq!(strip.width(), 8);
        assert_eq!(strip.height(), 16); // 8 * 2

        assert_eq!(strip.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(strip.get_pixel(0, 8).0, [0, 255, 0, 255]);
    }

    #[test]
    fn test_empty_strip_fails() {
        let result = build_strip(&[], &StripDirection::Horizontal);
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata_uniform() {
        let meta = build_metadata(
            "walk-strip.png",
            16,
            16,
            4,
            &StripDirection::Horizontal,
            &[100, 100, 100, 100],
        );

        assert_eq!(meta.frame_count, 4);
        assert_eq!(meta.timing.mode, "uniform");
        assert_eq!(meta.timing.frame_duration_ms, Some(100));
        assert_eq!(meta.timing.total_duration_ms, 400);
        assert_eq!(meta.frames.len(), 4);
        assert_eq!(meta.frames[0].x, 0);
        assert_eq!(meta.frames[1].x, 16);
        assert_eq!(meta.frames[2].x, 32);
    }

    #[test]
    fn test_metadata_variable() {
        let meta = build_metadata(
            "attack-strip.png",
            16,
            16,
            3,
            &StripDirection::Horizontal,
            &[80, 120, 200],
        );

        assert_eq!(meta.timing.mode, "variable");
        assert_eq!(meta.timing.frame_duration_ms, None);
        assert_eq!(meta.timing.total_duration_ms, 400);
    }
}
