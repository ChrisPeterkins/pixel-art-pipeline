use crate::error::{PipelineError, Result};
use crate::palette::extract::Color;
use image::{Rgba, RgbaImage};

/// Find the nearest color in a palette using Euclidean distance in RGB space.
pub fn nearest_color(color: Color, palette: &[Color]) -> Color {
    let mut best = palette[0];
    let mut best_dist = color_distance_sq(color, best);

    for &candidate in &palette[1..] {
        let dist = color_distance_sq(color, candidate);
        if dist < best_dist {
            best_dist = dist;
            best = candidate;
        }
    }

    best
}

/// Squared Euclidean distance in RGB space (ignores alpha for matching).
fn color_distance_sq(a: Color, b: Color) -> u32 {
    let dr = a[0] as i32 - b[0] as i32;
    let dg = a[1] as i32 - b[1] as i32;
    let db = a[2] as i32 - b[2] as i32;
    (dr * dr + dg * dg + db * db) as u32
}

/// Check if a color exists in the palette (exact match).
pub fn is_in_palette(color: Color, palette: &[Color]) -> bool {
    // Skip fully transparent — always allowed
    if color[3] == 0 {
        return true;
    }
    palette.iter().any(|&c| c[0] == color[0] && c[1] == color[1] && c[2] == color[2])
}

/// Enforce a palette on an image using the "nearest" strategy.
/// Returns a new image with all non-transparent pixels snapped to the nearest palette color.
pub fn enforce_nearest(image: &RgbaImage, palette: &[Color]) -> RgbaImage {
    let mut output = image.clone();
    for pixel in output.pixels_mut() {
        if pixel.0[3] == 0 {
            continue; // skip transparent
        }
        let nearest = nearest_color(pixel.0, palette);
        // Preserve original alpha
        *pixel = Rgba([nearest[0], nearest[1], nearest[2], pixel.0[3]]);
    }
    output
}

/// Enforce a palette on an image using the "error" strategy.
/// Returns Err if any non-transparent pixel is not in the palette.
pub fn enforce_error(image: &RgbaImage, palette: &[Color], image_name: &str) -> Result<()> {
    let mut violations = Vec::new();

    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel.0[3] == 0 {
            continue;
        }
        if !is_in_palette(pixel.0, palette) {
            violations.push((x, y, pixel.0));
            if violations.len() >= 10 {
                break; // cap reported violations
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        let details: Vec<String> = violations
            .iter()
            .map(|(x, y, c)| format!("  ({}, {}): #{:02x}{:02x}{:02x}", x, y, c[0], c[1], c[2]))
            .collect();
        Err(PipelineError::Palette(format!(
            "Image '{}' has {} out-of-palette pixel(s):\n{}",
            image_name,
            violations.len(),
            details.join("\n")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nearest_color_exact_match() {
        let palette = vec![[255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]];
        assert_eq!(nearest_color([255, 0, 0, 255], &palette), [255, 0, 0, 255]);
    }

    #[test]
    fn test_nearest_color_closest() {
        let palette = vec![[255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]];
        // (200, 30, 30) is closest to red
        assert_eq!(nearest_color([200, 30, 30, 255], &palette), [255, 0, 0, 255]);
        // (30, 200, 30) is closest to green
        assert_eq!(nearest_color([30, 200, 30, 255], &palette), [0, 255, 0, 255]);
    }

    #[test]
    fn test_is_in_palette() {
        let palette = vec![[255, 0, 0, 255], [0, 255, 0, 255]];
        assert!(is_in_palette([255, 0, 0, 255], &palette));
        assert!(!is_in_palette([128, 128, 128, 255], &palette));
    }

    #[test]
    fn test_transparent_always_in_palette() {
        let palette = vec![[255, 0, 0, 255]];
        assert!(is_in_palette([0, 0, 0, 0], &palette));
    }

    #[test]
    fn test_enforce_nearest() {
        let palette = vec![[255, 0, 0, 255], [0, 0, 255, 255]];
        let mut img = RgbaImage::new(2, 1);
        img.put_pixel(0, 0, Rgba([200, 30, 30, 255])); // near red
        img.put_pixel(1, 0, Rgba([30, 30, 200, 255])); // near blue

        let result = enforce_nearest(&img, &palette);
        assert_eq!(result.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(result.get_pixel(1, 0).0, [0, 0, 255, 255]);
    }

    #[test]
    fn test_enforce_nearest_preserves_alpha() {
        let palette = vec![[255, 0, 0, 255]];
        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, Rgba([200, 30, 30, 128])); // semi-transparent

        let result = enforce_nearest(&img, &palette);
        assert_eq!(result.get_pixel(0, 0).0, [255, 0, 0, 128]); // alpha preserved
    }

    #[test]
    fn test_enforce_nearest_skips_transparent() {
        let palette = vec![[255, 0, 0, 255]];
        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, Rgba([0, 0, 0, 0]));

        let result = enforce_nearest(&img, &palette);
        assert_eq!(result.get_pixel(0, 0).0, [0, 0, 0, 0]); // unchanged
    }

    #[test]
    fn test_enforce_error_passes() {
        let palette = vec![[255, 0, 0, 255], [0, 0, 0, 0]];
        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        assert!(enforce_error(&img, &palette, "test.png").is_ok());
    }

    #[test]
    fn test_enforce_error_fails() {
        let palette = vec![[255, 0, 0, 255]];
        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, Rgba([0, 255, 0, 255])); // not in palette

        assert!(enforce_error(&img, &palette, "test.png").is_err());
    }
}
