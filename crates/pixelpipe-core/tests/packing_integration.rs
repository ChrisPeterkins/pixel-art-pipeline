use pixelpipe_core::config;
use pixelpipe_core::pipeline;
use pixelpipe_test_fixtures::{create_test_sprites, solid_image};
use std::fs;
use std::path::PathBuf;

fn setup_test_project(test_name: &str) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir().join("pixelpipe-tests").join(test_name);
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();

    let input_dir = base.join("raw").join("sprites");
    create_test_sprites(&input_dir);

    (base, input_dir)
}

#[test]
fn test_full_pipeline_packs_sprites_and_writes_output() {
    let (base, _) = setup_test_project("full_pipeline");
    let output_dir = base.join("dist");

    // Write a config file
    let config_yaml = format!(
        r#"
project:
  name: "integration-test"
  input_dir: "./raw"
  output_dir: "{}"

defaults:
  padding: 1
  power_of_two: true

sheets:
  - name: "test-sheet"
    inputs: ["sprites/*.png"]
    output_formats: ["phaser"]
"#,
        output_dir.display()
    );

    let config_path = base.join("pixelpipe.yaml");
    fs::write(&config_path, &config_yaml).unwrap();

    // Load config and run pipeline
    let cfg = config::load_config(&config_path).unwrap();
    let ctx = pipeline::run_pipeline(cfg, base.clone()).unwrap();

    // Verify sheet was packed
    assert!(ctx.sheets.contains_key("test-sheet"));
    let sheet = &ctx.sheets["test-sheet"];
    assert_eq!(sheet.frames.len(), 5); // 5 test sprites
    assert!(sheet.width > 0);
    assert!(sheet.height > 0);

    // Verify width and height are powers of two
    assert!(sheet.width.is_power_of_two());
    assert!(sheet.height.is_power_of_two());

    // Verify output files were written
    let png_path = output_dir.join("test-sheet.png");
    let json_path = output_dir.join("test-sheet.json");
    assert!(png_path.exists(), "Atlas PNG was not written");
    assert!(json_path.exists(), "Phaser JSON was not written");

    // Verify JSON is valid and contains all frames
    let json_str = fs::read_to_string(&json_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(json["frames"].is_object());
    assert_eq!(json["meta"]["app"], "pixelpipe");
    assert_eq!(json["meta"]["image"], "test-sheet.png");

    // Verify each test sprite appears in JSON
    let frame_names: Vec<String> = json["frames"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    assert_eq!(frame_names.len(), 5);

    // Verify atlas image can be loaded back
    let atlas = image::open(&png_path).unwrap().to_rgba8();
    assert_eq!(atlas.width(), sheet.width);
    assert_eq!(atlas.height(), sheet.height);

    // Cleanup
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_round_trip_pixel_accuracy() {
    let base = std::env::temp_dir()
        .join("pixelpipe-tests")
        .join("round_trip");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();

    let input_dir = base.join("raw").join("sprites");
    fs::create_dir_all(&input_dir).unwrap();

    // Create known sprites with distinct solid colors
    let colors: Vec<([u8; 4], &str)> = vec![
        ([255, 0, 0, 255], "red"),
        ([0, 255, 0, 255], "green"),
        ([0, 0, 255, 255], "blue"),
    ];

    for (color, name) in &colors {
        let img = solid_image(16, 16, *color);
        img.save(input_dir.join(format!("{}.png", name))).unwrap();
    }

    let output_dir = base.join("dist");
    let config_yaml = format!(
        r#"
project:
  name: "round-trip-test"
  input_dir: "./raw"
  output_dir: "{}"

defaults:
  padding: 0
  power_of_two: false

sheets:
  - name: "colors"
    inputs: ["sprites/*.png"]
    output_formats: ["phaser"]
"#,
        output_dir.display()
    );

    let config_path = base.join("pixelpipe.yaml");
    fs::write(&config_path, &config_yaml).unwrap();

    let cfg = config::load_config(&config_path).unwrap();
    let ctx = pipeline::run_pipeline(cfg, base.clone()).unwrap();

    let sheet = &ctx.sheets["colors"];
    let atlas = &sheet.image;

    // For each placed frame, verify pixels match the original color
    for frame in &sheet.frames {
        let expected_color: [u8; 4] = match frame.name.as_str() {
            "red" => [255, 0, 0, 255],
            "green" => [0, 255, 0, 255],
            "blue" => [0, 0, 255, 255],
            _ => panic!("Unexpected frame name: {}", frame.name),
        };

        // Check every pixel in this frame's region
        for dy in 0..frame.height {
            for dx in 0..frame.width {
                let pixel = atlas.get_pixel(frame.x + dx, frame.y + dy);
                assert_eq!(
                    pixel.0, expected_color,
                    "Pixel mismatch at ({}, {}) in frame '{}': expected {:?}, got {:?}",
                    dx, dy, frame.name, expected_color, pixel.0
                );
            }
        }
    }

    // Cleanup
    let _ = fs::remove_dir_all(&base);
}
