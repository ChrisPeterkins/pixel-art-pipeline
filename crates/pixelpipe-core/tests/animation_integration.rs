use image::{Rgba, RgbaImage};
use pixelpipe_core::config;
use pixelpipe_core::pipeline;
use std::fs;

#[test]
fn test_animation_gif_and_strip_output() {
    let base = std::env::temp_dir()
        .join("pixelpipe-tests")
        .join("animation_full");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();

    // Create animation frames
    let frames_dir = base.join("raw").join("walk");
    fs::create_dir_all(&frames_dir).unwrap();

    let colors: Vec<[u8; 4]> = vec![
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [255, 255, 0, 255],
    ];

    for (i, color) in colors.iter().enumerate() {
        let mut img = RgbaImage::new(16, 16);
        for pixel in img.pixels_mut() {
            *pixel = Rgba(*color);
        }
        img.save(frames_dir.join(format!("frame_{}.png", i))).unwrap();
    }

    let output_dir = base.join("dist");
    let config_yaml = format!(
        r#"
project:
  name: "animation-test"
  input_dir: "./raw"
  output_dir: "{}"

animations:
  - name: "walk"
    frames:
      - pattern: "walk/frame_*.png"
        sort: "natural"
    timing:
      frame_duration_ms: 100
    outputs:
      - type: "gif"
        loop: true
        output: "walk.gif"
      - type: "strip"
        direction: "horizontal"
        output: "walk-strip.png"
        metadata: true
"#,
        output_dir.display()
    );

    let config_path = base.join("pixelpipe.yaml");
    fs::write(&config_path, &config_yaml).unwrap();

    let cfg = config::load_config(&config_path).unwrap();
    let ctx = pipeline::run_pipeline(cfg, base.clone()).unwrap();

    // Verify animation result
    assert!(ctx.animations.contains_key("walk"));
    let anim = &ctx.animations["walk"];
    assert_eq!(anim.frame_count, 4);
    assert_eq!(anim.frame_width, 16);
    assert_eq!(anim.frame_height, 16);
    assert_eq!(anim.timing, vec![100, 100, 100, 100]);

    // Verify GIF output
    let gif_path = output_dir.join("walk.gif");
    assert!(gif_path.exists(), "GIF file not written");
    let gif_data = fs::read(&gif_path).unwrap();
    assert_eq!(&gif_data[0..6], b"GIF89a", "Not a valid GIF file");

    // Verify strip output
    let strip_path = output_dir.join("walk-strip.png");
    assert!(strip_path.exists(), "Strip PNG not written");
    let strip = image::open(&strip_path).unwrap().to_rgba8();
    assert_eq!(strip.width(), 64); // 16 * 4 frames
    assert_eq!(strip.height(), 16);

    // Verify strip pixel accuracy
    assert_eq!(strip.get_pixel(0, 0).0, [255, 0, 0, 255]); // frame 0: red
    assert_eq!(strip.get_pixel(16, 0).0, [0, 255, 0, 255]); // frame 1: green
    assert_eq!(strip.get_pixel(32, 0).0, [0, 0, 255, 255]); // frame 2: blue
    assert_eq!(strip.get_pixel(48, 0).0, [255, 255, 0, 255]); // frame 3: yellow

    // Verify metadata JSON
    let json_path = output_dir.join("walk-strip.json");
    assert!(json_path.exists(), "Strip metadata JSON not written");
    let json_str = fs::read_to_string(&json_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["frame_count"], 4);
    assert_eq!(json["frame_width"], 16);
    assert_eq!(json["frame_height"], 16);
    assert_eq!(json["direction"], "horizontal");
    assert_eq!(json["timing"]["mode"], "uniform");
    assert_eq!(json["timing"]["frame_duration_ms"], 100);
    assert_eq!(json["timing"]["total_duration_ms"], 400);
    assert_eq!(json["frames"].as_array().unwrap().len(), 4);

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_animation_variable_timing() {
    let base = std::env::temp_dir()
        .join("pixelpipe-tests")
        .join("animation_variable");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();

    let frames_dir = base.join("raw").join("attack");
    fs::create_dir_all(&frames_dir).unwrap();

    for i in 0..3u32 {
        let mut img = RgbaImage::new(8, 8);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([i as u8 * 80, 0, 0, 255]);
        }
        img.save(frames_dir.join(format!("frame_{}.png", i))).unwrap();
    }

    let output_dir = base.join("dist");
    let config_yaml = format!(
        r#"
project:
  name: "variable-timing-test"
  input_dir: "./raw"
  output_dir: "{}"

animations:
  - name: "attack"
    frames:
      - pattern: "attack/frame_*.png"
        sort: "natural"
    timing:
      durations_ms: [80, 120, 200]
    outputs:
      - type: "strip"
        direction: "horizontal"
        output: "attack-strip.png"
        metadata: true
"#,
        output_dir.display()
    );

    let config_path = base.join("pixelpipe.yaml");
    fs::write(&config_path, &config_yaml).unwrap();

    let cfg = config::load_config(&config_path).unwrap();
    let ctx = pipeline::run_pipeline(cfg, base.clone()).unwrap();

    let anim = &ctx.animations["attack"];
    assert_eq!(anim.timing, vec![80, 120, 200]);

    // Check metadata
    let json_str = fs::read_to_string(output_dir.join("attack-strip.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["timing"]["mode"], "variable");
    assert_eq!(json["timing"]["total_duration_ms"], 400);
    assert_eq!(json["frames"][0]["duration_ms"], 80);
    assert_eq!(json["frames"][1]["duration_ms"], 120);
    assert_eq!(json["frames"][2]["duration_ms"], 200);

    let _ = fs::remove_dir_all(&base);
}
