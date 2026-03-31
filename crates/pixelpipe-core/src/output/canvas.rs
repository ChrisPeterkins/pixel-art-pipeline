use crate::pipeline::SheetResult;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct CanvasAtlas {
    image: String,
    size: CanvasSize,
    sprites: BTreeMap<String, CanvasSprite>,
}

#[derive(Serialize)]
struct CanvasSize {
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct CanvasSprite {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

/// Serialize a SheetResult into a simple Canvas JSON format for drawImage() usage.
pub fn serialize(sheet: &SheetResult, image_filename: &str) -> String {
    let mut sprites = BTreeMap::new();

    for frame in &sheet.frames {
        sprites.insert(
            frame.name.clone(),
            CanvasSprite {
                x: frame.x,
                y: frame.y,
                w: frame.width,
                h: frame.height,
            },
        );
    }

    let atlas = CanvasAtlas {
        image: image_filename.to_string(),
        size: CanvasSize {
            w: sheet.width,
            h: sheet.height,
        },
        sprites,
    };

    serde_json::to_string_pretty(&atlas).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{FramePlacement, SheetResult};
    use image::RgbaImage;

    #[test]
    fn test_canvas_json_output() {
        let sheet = SheetResult {
            image: RgbaImage::new(64, 32),
            frames: vec![
                FramePlacement {
                    name: "grass".to_string(),
                    x: 0,
                    y: 0,
                    width: 16,
                    height: 16,
                },
                FramePlacement {
                    name: "stone".to_string(),
                    x: 17,
                    y: 0,
                    width: 16,
                    height: 16,
                },
            ],
            width: 64,
            height: 32,
        };

        let json = serialize(&sheet, "terrain.png");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["image"], "terrain.png");
        assert_eq!(parsed["size"]["w"], 64);
        assert_eq!(parsed["size"]["h"], 32);
        assert_eq!(parsed["sprites"]["grass"]["x"], 0);
        assert_eq!(parsed["sprites"]["grass"]["w"], 16);
        assert_eq!(parsed["sprites"]["stone"]["x"], 17);
    }
}
