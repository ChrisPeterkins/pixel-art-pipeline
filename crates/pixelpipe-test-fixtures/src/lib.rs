use image::{Rgba, RgbaImage};
use std::path::{Path, PathBuf};

/// Create a solid-color test image.
pub fn solid_image(width: u32, height: u32, color: [u8; 4]) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = Rgba(color);
    }
    img
}

/// Create a checkerboard test image with two colors.
pub fn checkerboard_image(
    width: u32,
    height: u32,
    color_a: [u8; 4],
    color_b: [u8; 4],
) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = if (x + y) % 2 == 0 {
            Rgba(color_a)
        } else {
            Rgba(color_b)
        };
    }
    img
}

/// Write test images to a temporary directory and return the path.
pub fn create_test_sprites(dir: &Path) -> Vec<PathBuf> {
    std::fs::create_dir_all(dir).unwrap();

    let sprites = vec![
        ("red_8x8.png", solid_image(8, 8, [255, 0, 0, 255])),
        ("green_8x8.png", solid_image(8, 8, [0, 255, 0, 255])),
        ("blue_16x16.png", solid_image(16, 16, [0, 0, 255, 255])),
        (
            "checker_16x16.png",
            checkerboard_image(16, 16, [255, 255, 255, 255], [0, 0, 0, 255]),
        ),
        (
            "large_32x32.png",
            solid_image(32, 32, [128, 64, 192, 255]),
        ),
    ];

    let mut paths = Vec::new();
    for (name, img) in sprites {
        let path = dir.join(name);
        img.save(&path).unwrap();
        paths.push(path);
    }
    paths
}

/// Create a sequence of animation frames.
pub fn create_animation_frames(dir: &Path, count: u32) -> Vec<PathBuf> {
    std::fs::create_dir_all(dir).unwrap();

    let mut paths = Vec::new();
    for i in 0..count {
        let brightness = ((i as f32 / count as f32) * 255.0) as u8;
        let img = solid_image(16, 16, [brightness, brightness, brightness, 255]);
        let path = dir.join(format!("frame_{:02}.png", i));
        img.save(&path).unwrap();
        paths.push(path);
    }
    paths
}
