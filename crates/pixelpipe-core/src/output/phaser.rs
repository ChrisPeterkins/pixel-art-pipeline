use crate::pipeline::SheetResult;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct PhaserAtlas {
    frames: BTreeMap<String, PhaserFrame>,
    meta: PhaserMeta,
}

#[derive(Serialize)]
struct PhaserFrame {
    frame: PhaserRect,
    rotated: bool,
    trimmed: bool,
    #[serde(rename = "spriteSourceSize")]
    sprite_source_size: PhaserRect,
    #[serde(rename = "sourceSize")]
    source_size: PhaserSize,
}

#[derive(Serialize)]
struct PhaserRect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct PhaserSize {
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct PhaserMeta {
    app: String,
    version: String,
    image: String,
    format: String,
    size: PhaserSize,
    scale: u32,
}

/// Serialize a SheetResult into Phaser 3 JSON Hash format.
pub fn serialize(sheet: &SheetResult, image_filename: &str) -> String {
    let mut frames = BTreeMap::new();

    for frame in &sheet.frames {
        let key = format!("{}.png", frame.name);
        frames.insert(
            key,
            PhaserFrame {
                frame: PhaserRect {
                    x: frame.x,
                    y: frame.y,
                    w: frame.width,
                    h: frame.height,
                },
                rotated: false,
                trimmed: false,
                sprite_source_size: PhaserRect {
                    x: 0,
                    y: 0,
                    w: frame.width,
                    h: frame.height,
                },
                source_size: PhaserSize {
                    w: frame.width,
                    h: frame.height,
                },
            },
        );
    }

    let atlas = PhaserAtlas {
        frames,
        meta: PhaserMeta {
            app: "pixelpipe".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            image: image_filename.to_string(),
            format: "RGBA8888".to_string(),
            size: PhaserSize {
                w: sheet.width,
                h: sheet.height,
            },
            scale: 1,
        },
    };

    serde_json::to_string_pretty(&atlas).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{FramePlacement, SheetResult};
    use image::RgbaImage;

    #[test]
    fn test_phaser_json_output() {
        let sheet = SheetResult {
            image: RgbaImage::new(64, 64),
            frames: vec![
                FramePlacement {
                    name: "sword".to_string(),
                    x: 0,
                    y: 0,
                    width: 16,
                    height: 16,
                },
                FramePlacement {
                    name: "shield".to_string(),
                    x: 17,
                    y: 0,
                    width: 16,
                    height: 16,
                },
            ],
            width: 64,
            height: 64,
        };

        let json = serialize(&sheet, "sprites.png");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Check frames exist
        assert!(parsed["frames"]["shield.png"].is_object());
        assert!(parsed["frames"]["sword.png"].is_object());

        // Check frame data
        assert_eq!(parsed["frames"]["sword.png"]["frame"]["x"], 0);
        assert_eq!(parsed["frames"]["sword.png"]["frame"]["w"], 16);
        assert_eq!(parsed["frames"]["sword.png"]["rotated"], false);
        assert_eq!(parsed["frames"]["sword.png"]["trimmed"], false);

        // Check meta
        assert_eq!(parsed["meta"]["app"], "pixelpipe");
        assert_eq!(parsed["meta"]["image"], "sprites.png");
        assert_eq!(parsed["meta"]["format"], "RGBA8888");
        assert_eq!(parsed["meta"]["size"]["w"], 64);
    }
}
