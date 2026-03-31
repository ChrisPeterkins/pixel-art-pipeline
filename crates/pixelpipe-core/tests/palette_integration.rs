use image::{Rgba, RgbaImage};
use pixelpipe_core::config;
use pixelpipe_core::pipeline;
use std::fs;

#[test]
fn test_palette_enforce_nearest_snaps_colors() {
    let base = std::env::temp_dir()
        .join("pixelpipe-tests")
        .join("palette_enforce");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();

    let input_dir = base.join("raw");
    let sprites_dir = input_dir.join("sprites");
    let ref_dir = input_dir.join("reference");
    fs::create_dir_all(&sprites_dir).unwrap();
    fs::create_dir_all(&ref_dir).unwrap();

    // Create a reference palette image with exactly 2 colors: red and blue
    let mut palette_img = RgbaImage::new(2, 1);
    palette_img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
    palette_img.put_pixel(1, 0, Rgba([0, 0, 255, 255]));
    palette_img.save(ref_dir.join("palette.png")).unwrap();

    // Create a sprite with off-palette colors (near-red and near-blue)
    let mut sprite = RgbaImage::new(4, 4);
    for y in 0..4u32 {
        for x in 0..4u32 {
            if x < 2 {
                sprite.put_pixel(x, y, Rgba([200, 30, 30, 255])); // near red
            } else {
                sprite.put_pixel(x, y, Rgba([30, 30, 200, 255])); // near blue
            }
        }
    }
    sprite.save(sprites_dir.join("test.png")).unwrap();

    let output_dir = base.join("dist");
    let config_yaml = format!(
        r#"
project:
  name: "palette-enforce-test"
  input_dir: "./raw"
  output_dir: "{}"

defaults:
  padding: 0
  power_of_two: false

palettes:
  definitions:
    - name: "test-palette"
      source: "reference/palette.png"
      max_colors: 8
  operations:
    - type: "enforce"
      palette: "test-palette"
      targets: ["sprites/*.png"]
      strategy: "nearest"

sheets:
  - name: "sprites"
    inputs: ["sprites/*.png"]
    output_formats: ["phaser"]
"#,
        output_dir.display()
    );

    let config_path = base.join("pixelpipe.yaml");
    fs::write(&config_path, &config_yaml).unwrap();

    let cfg = config::load_config(&config_path).unwrap();
    let ctx = pipeline::run_pipeline(cfg, base.clone()).unwrap();

    // After enforcement, the sprite should have been snapped to palette colors
    // The packer then packs the enforced sprites
    let sheet = &ctx.sheets["sprites"];
    let atlas = &sheet.image;
    let frame = &sheet.frames[0];

    // Check that all pixels are now exactly red or blue
    for dy in 0..frame.height {
        for dx in 0..frame.width {
            let pixel = atlas.get_pixel(frame.x + dx, frame.y + dy).0;
            assert!(
                pixel == [255, 0, 0, 255] || pixel == [0, 0, 255, 255],
                "Pixel at ({}, {}) is {:?}, expected pure red or blue",
                dx, dy, pixel
            );
        }
    }

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_palette_swap_produces_recolored_images() {
    let base = std::env::temp_dir()
        .join("pixelpipe-tests")
        .join("palette_swap");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();

    let input_dir = base.join("raw");
    let sprites_dir = input_dir.join("characters");
    fs::create_dir_all(&sprites_dir).unwrap();

    // Create a sprite with red and green pixels
    let mut sprite = RgbaImage::new(4, 4);
    for y in 0..4u32 {
        for x in 0..4u32 {
            if (x + y) % 2 == 0 {
                sprite.put_pixel(x, y, Rgba([255, 0, 0, 255])); // red
            } else {
                sprite.put_pixel(x, y, Rgba([0, 255, 0, 255])); // green
            }
        }
    }
    sprite.save(sprites_dir.join("hero.png")).unwrap();

    let swap_output_dir = base.join("dist").join("swapped");
    let config_yaml = format!(
        r##"
project:
  name: "palette-swap-test"
  input_dir: "./raw"
  output_dir: "./dist/assets"

palettes:
  definitions:
    - name: "hero-colors"
      colors: ["#ff0000", "#00ff00"]
    - name: "enemy-colors"
      colors: ["#0000ff", "#ffff00"]
  operations:
    - type: "swap"
      source_palette: "hero-colors"
      target_palette: "enemy-colors"
      inputs: ["characters/*.png"]
      output_dir: "{}"
      output_suffix: "_enemy"
"##,
        swap_output_dir.display()
    );

    let config_path = base.join("pixelpipe.yaml");
    fs::write(&config_path, &config_yaml).unwrap();

    let cfg = config::load_config(&config_path).unwrap();
    let _ctx = pipeline::run_pipeline(cfg, base.clone()).unwrap();

    // Check that the swapped image exists
    let swapped_path = swap_output_dir.join("hero_enemy.png");
    assert!(swapped_path.exists(), "Swapped image was not written");

    // Load and verify colors were swapped
    let swapped = image::open(&swapped_path).unwrap().to_rgba8();
    assert_eq!(swapped.width(), 4);
    assert_eq!(swapped.height(), 4);

    for y in 0..4u32 {
        for x in 0..4u32 {
            let pixel = swapped.get_pixel(x, y).0;
            if (x + y) % 2 == 0 {
                // Was red → should be blue
                assert_eq!(pixel, [0, 0, 255, 255], "Expected blue at ({}, {})", x, y);
            } else {
                // Was green → should be yellow
                assert_eq!(pixel, [255, 255, 0, 255], "Expected yellow at ({}, {})", x, y);
            }
        }
    }

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_palette_enforce_error_catches_violations() {
    let base = std::env::temp_dir()
        .join("pixelpipe-tests")
        .join("palette_error");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();

    let input_dir = base.join("raw");
    let sprites_dir = input_dir.join("sprites");
    fs::create_dir_all(&sprites_dir).unwrap();

    // Create a sprite with a color NOT in the palette
    let mut sprite = RgbaImage::new(2, 2);
    sprite.put_pixel(0, 0, Rgba([128, 128, 128, 255])); // gray — not in palette
    sprite.put_pixel(1, 0, Rgba([255, 0, 0, 255]));
    sprite.put_pixel(0, 1, Rgba([255, 0, 0, 255]));
    sprite.put_pixel(1, 1, Rgba([255, 0, 0, 255]));
    sprite.save(sprites_dir.join("bad.png")).unwrap();

    let config_yaml = r##"
project:
  name: "palette-error-test"
  input_dir: "./raw"
  output_dir: "./dist"

palettes:
  definitions:
    - name: "strict-palette"
      colors: ["#ff0000", "#00ff00"]
  operations:
    - type: "enforce"
      palette: "strict-palette"
      targets: ["sprites/*.png"]
      strategy: "error"
"##;

    let config_path = base.join("pixelpipe.yaml");
    fs::write(&config_path, config_yaml).unwrap();

    let cfg = config::load_config(&config_path).unwrap();
    let result = pipeline::run_pipeline(cfg, base.clone());

    // Should fail because gray is not in the palette
    assert!(result.is_err());
    let err = format!("{}", result.err().unwrap());
    assert!(err.contains("out-of-palette"), "Error message: {}", err);

    let _ = fs::remove_dir_all(&base);
}
