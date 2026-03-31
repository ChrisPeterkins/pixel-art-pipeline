///! Run with: cargo run --example generate_test_sprites
use image::{Rgba, RgbaImage};
use std::fs;
use std::path::Path;

fn main() {
    let dir = Path::new("raw/sprites");
    fs::create_dir_all(dir).unwrap();

    // 16x16 sword icon
    let mut sword = RgbaImage::new(16, 16);
    for y in 0..16u32 {
        for x in 0..16u32 {
            let color = if (4..6).contains(&x) && (2..12).contains(&y) {
                [192, 192, 210, 255] // blade
            } else if (3..7).contains(&x) && (12..15).contains(&y) {
                [139, 90, 43, 255] // handle
            } else if (2..8).contains(&x) && (11..13).contains(&y) {
                [218, 165, 32, 255] // crossguard
            } else {
                [0, 0, 0, 0] // transparent
            };
            sword.put_pixel(x, y, Rgba(color));
        }
    }
    sword.save(dir.join("sword.png")).unwrap();

    // 16x16 shield icon
    let mut shield = RgbaImage::new(16, 16);
    for y in 0..16u32 {
        for x in 0..16u32 {
            let cx = (x as f32 - 7.5).abs();
            let cy = (y as f32 - 7.5).abs();
            let color = if cx + cy < 8.0 {
                if cx + cy < 3.0 {
                    [218, 165, 32, 255] // emblem
                } else {
                    [70, 70, 200, 255] // shield body
                }
            } else {
                [0, 0, 0, 0]
            };
            shield.put_pixel(x, y, Rgba(color));
        }
    }
    shield.save(dir.join("shield.png")).unwrap();

    // 16x16 heart
    let mut heart = RgbaImage::new(16, 16);
    let heart_pixels: Vec<(u32, u32)> = vec![
        (3,2),(4,2),(5,2),(9,2),(10,2),(11,2),
        (2,3),(3,3),(4,3),(5,3),(6,3),(8,3),(9,3),(10,3),(11,3),(12,3),
        (1,4),(2,4),(3,4),(4,4),(5,4),(6,4),(7,4),(8,4),(9,4),(10,4),(11,4),(12,4),(13,4),
        (1,5),(2,5),(3,5),(4,5),(5,5),(6,5),(7,5),(8,5),(9,5),(10,5),(11,5),(12,5),(13,5),
        (2,6),(3,6),(4,6),(5,6),(6,6),(7,6),(8,6),(9,6),(10,6),(11,6),(12,6),
        (3,7),(4,7),(5,7),(6,7),(7,7),(8,7),(9,7),(10,7),(11,7),
        (4,8),(5,8),(6,8),(7,8),(8,8),(9,8),(10,8),
        (5,9),(6,9),(7,9),(8,9),(9,9),
        (6,10),(7,10),(8,10),
        (7,11),
    ];
    for (x, y) in heart_pixels {
        heart.put_pixel(x, y, Rgba([220, 20, 60, 255]));
    }
    heart.save(dir.join("heart.png")).unwrap();

    // 32x32 potion bottle
    let mut potion = RgbaImage::new(32, 32);
    for y in 0..32u32 {
        for x in 0..32u32 {
            let color = if (13..19).contains(&x) && (2..8).contains(&y) {
                [160, 160, 170, 255] // bottle neck
            } else if (10..22).contains(&x) && (8..10).contains(&y) {
                [160, 160, 170, 255] // bottle shoulder
            } else if (8..24).contains(&x) && (10..26).contains(&y) {
                if y < 16 {
                    [200, 200, 220, 128] // empty glass
                } else {
                    [180, 40, 220, 220] // purple liquid
                }
            } else if (10..22).contains(&x) && (26..28).contains(&y) {
                [160, 160, 170, 255] // bottle bottom
            } else {
                [0, 0, 0, 0]
            };
            potion.put_pixel(x, y, Rgba(color));
        }
    }
    potion.save(dir.join("potion.png")).unwrap();

    // 8x8 coin
    let mut coin = RgbaImage::new(8, 8);
    for y in 0..8u32 {
        for x in 0..8u32 {
            let cx = (x as f32 - 3.5).powi(2);
            let cy = (y as f32 - 3.5).powi(2);
            let color = if cx + cy < 10.0 {
                if cx + cy < 4.0 {
                    [255, 223, 0, 255] // bright center
                } else {
                    [218, 165, 32, 255] // gold edge
                }
            } else {
                [0, 0, 0, 0]
            };
            coin.put_pixel(x, y, Rgba(color));
        }
    }
    coin.save(dir.join("coin.png")).unwrap();

    println!("Generated 5 test sprites in raw/sprites/");
    println!("  - sword.png   (16x16)");
    println!("  - shield.png  (16x16)");
    println!("  - heart.png   (16x16)");
    println!("  - potion.png  (32x32)");
    println!("  - coin.png    (8x8)");
}
