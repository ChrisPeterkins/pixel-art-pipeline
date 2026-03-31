use crate::error::{PipelineError, Result};
use image::RgbaImage;
use std::collections::BTreeSet;
use std::path::Path;

/// An RGBA color.
pub type Color = [u8; 4];

/// A named palette of colors.
#[derive(Debug, Clone)]
pub struct Palette {
    pub name: String,
    pub colors: Vec<Color>,
}

/// Extract unique colors from an image (ignoring fully transparent pixels).
pub fn extract_from_image(image: &RgbaImage) -> Vec<Color> {
    let mut seen = BTreeSet::new();
    for pixel in image.pixels() {
        // Skip fully transparent pixels
        if pixel.0[3] == 0 {
            continue;
        }
        seen.insert(pixel.0);
    }
    seen.into_iter().collect()
}

/// Load a palette from a source image file.
pub fn load_from_source(path: &Path, max_colors: Option<u32>) -> Result<Vec<Color>> {
    let img = image::open(path)
        .map_err(|e| PipelineError::Image {
            path: path.to_path_buf(),
            source: e,
        })?
        .to_rgba8();

    let colors = extract_from_image(&img);

    if let Some(max) = max_colors {
        if colors.len() > max as usize {
            return Err(PipelineError::Palette(format!(
                "Image '{}' has {} colors, exceeds max_colors limit of {}",
                path.display(),
                colors.len(),
                max
            )));
        }
    }

    Ok(colors)
}

/// Parse hex color strings (e.g. "#ff0000" or "#ff0000ff") into RGBA.
pub fn parse_hex_colors(hex_colors: &[String]) -> Result<Vec<Color>> {
    hex_colors.iter().map(|s| parse_hex(s)).collect()
}

fn parse_hex(hex: &str) -> Result<Color> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| PipelineError::Palette(format!("Invalid hex color: #{}", hex)))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| PipelineError::Palette(format!("Invalid hex color: #{}", hex)))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| PipelineError::Palette(format!("Invalid hex color: #{}", hex)))?;
            Ok([r, g, b, 255])
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| PipelineError::Palette(format!("Invalid hex color: #{}", hex)))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| PipelineError::Palette(format!("Invalid hex color: #{}", hex)))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| PipelineError::Palette(format!("Invalid hex color: #{}", hex)))?;
            let a = u8::from_str_radix(&hex[6..8], 16)
                .map_err(|_| PipelineError::Palette(format!("Invalid hex color: #{}", hex)))?;
            Ok([r, g, b, a])
        }
        _ => Err(PipelineError::Palette(format!(
            "Hex color must be 6 or 8 characters (got {}): #{}",
            hex.len(),
            hex
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_extract_from_solid_image() {
        let mut img = RgbaImage::new(4, 4);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255, 0, 0, 255]);
        }
        let colors = extract_from_image(&img);
        assert_eq!(colors, vec![[255, 0, 0, 255]]);
    }

    #[test]
    fn test_extract_skips_transparent() {
        let mut img = RgbaImage::new(2, 2);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        img.put_pixel(1, 0, Rgba([0, 0, 0, 0])); // transparent
        img.put_pixel(0, 1, Rgba([0, 255, 0, 255]));
        img.put_pixel(1, 1, Rgba([0, 0, 0, 0])); // transparent

        let colors = extract_from_image(&img);
        assert_eq!(colors.len(), 2);
        assert!(colors.contains(&[255, 0, 0, 255]));
        assert!(colors.contains(&[0, 255, 0, 255]));
    }

    #[test]
    fn test_extract_deduplicates() {
        let mut img = RgbaImage::new(4, 4);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([42, 42, 42, 255]);
        }
        // One different pixel
        img.put_pixel(0, 0, Rgba([100, 100, 100, 255]));

        let colors = extract_from_image(&img);
        assert_eq!(colors.len(), 2);
    }

    #[test]
    fn test_parse_hex_6_char() {
        assert_eq!(parse_hex("#ff0000").unwrap(), [255, 0, 0, 255]);
        assert_eq!(parse_hex("#00ff00").unwrap(), [0, 255, 0, 255]);
        assert_eq!(parse_hex("#1a1c2c").unwrap(), [26, 28, 44, 255]);
    }

    #[test]
    fn test_parse_hex_8_char() {
        assert_eq!(parse_hex("#ff000080").unwrap(), [255, 0, 0, 128]);
    }

    #[test]
    fn test_parse_hex_no_hash() {
        assert_eq!(parse_hex("ff0000").unwrap(), [255, 0, 0, 255]);
    }

    #[test]
    fn test_parse_hex_invalid() {
        assert!(parse_hex("#xyz").is_err());
        assert!(parse_hex("#12345").is_err());
    }
}
