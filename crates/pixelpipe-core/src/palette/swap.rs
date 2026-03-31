use crate::palette::extract::Color;
use image::{Rgba, RgbaImage};
use std::collections::HashMap;

/// Build a color mapping from source palette to target palette.
/// Colors are matched by index position — palettes must be the same length.
pub fn build_color_map(source: &[Color], target: &[Color]) -> HashMap<[u8; 3], [u8; 3]> {
    let mut map = HashMap::new();
    for (s, t) in source.iter().zip(target.iter()) {
        map.insert([s[0], s[1], s[2]], [t[0], t[1], t[2]]);
    }
    map
}

/// Swap colors in an image using a source→target color map.
/// Only exact RGB matches are swapped. Alpha is preserved.
pub fn swap_colors(image: &RgbaImage, color_map: &HashMap<[u8; 3], [u8; 3]>) -> RgbaImage {
    let mut output = image.clone();
    for pixel in output.pixels_mut() {
        if pixel.0[3] == 0 {
            continue; // skip transparent
        }
        let rgb = [pixel.0[0], pixel.0[1], pixel.0[2]];
        if let Some(&new_rgb) = color_map.get(&rgb) {
            *pixel = Rgba([new_rgb[0], new_rgb[1], new_rgb[2], pixel.0[3]]);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_color_map() {
        let source = vec![[255, 0, 0, 255], [0, 255, 0, 255]];
        let target = vec![[0, 0, 255, 255], [255, 255, 0, 255]];
        let map = build_color_map(&source, &target);

        assert_eq!(map.get(&[255, 0, 0]), Some(&[0, 0, 255]));
        assert_eq!(map.get(&[0, 255, 0]), Some(&[255, 255, 0]));
    }

    #[test]
    fn test_swap_colors() {
        let source = vec![[255, 0, 0, 255], [0, 255, 0, 255]];
        let target = vec![[0, 0, 255, 255], [255, 255, 0, 255]];
        let map = build_color_map(&source, &target);

        let mut img = RgbaImage::new(2, 1);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255])); // red → blue
        img.put_pixel(1, 0, Rgba([0, 255, 0, 255])); // green → yellow

        let result = swap_colors(&img, &map);
        assert_eq!(result.get_pixel(0, 0).0, [0, 0, 255, 255]);
        assert_eq!(result.get_pixel(1, 0).0, [255, 255, 0, 255]);
    }

    #[test]
    fn test_swap_preserves_alpha() {
        let source = vec![[255, 0, 0, 255]];
        let target = vec![[0, 0, 255, 255]];
        let map = build_color_map(&source, &target);

        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 128])); // semi-transparent red

        let result = swap_colors(&img, &map);
        assert_eq!(result.get_pixel(0, 0).0, [0, 0, 255, 128]); // blue with same alpha
    }

    #[test]
    fn test_swap_skips_unmatched_colors() {
        let source = vec![[255, 0, 0, 255]];
        let target = vec![[0, 0, 255, 255]];
        let map = build_color_map(&source, &target);

        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, Rgba([0, 255, 0, 255])); // green — not in map

        let result = swap_colors(&img, &map);
        assert_eq!(result.get_pixel(0, 0).0, [0, 255, 0, 255]); // unchanged
    }

    #[test]
    fn test_swap_skips_transparent() {
        let source = vec![[0, 0, 0, 255]];
        let target = vec![[255, 255, 255, 255]];
        let map = build_color_map(&source, &target);

        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, Rgba([0, 0, 0, 0])); // transparent black

        let result = swap_colors(&img, &map);
        assert_eq!(result.get_pixel(0, 0).0, [0, 0, 0, 0]); // still transparent
    }
}
