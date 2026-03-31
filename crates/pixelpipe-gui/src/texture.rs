use egui::{ColorImage, TextureHandle, TextureOptions};
use image::RgbaImage;
use std::collections::HashMap;

/// Convert an image::RgbaImage to an egui ColorImage.
pub fn rgba_to_color_image(img: &RgbaImage) -> ColorImage {
    let size = [img.width() as usize, img.height() as usize];
    let pixels: Vec<egui::Color32> = img
        .pixels()
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    ColorImage { size, pixels }
}

/// Upload an RgbaImage to the GPU as an egui texture.
/// Uses Nearest filtering — critical for pixel art (no blurring).
pub fn upload_texture(
    ctx: &egui::Context,
    name: impl Into<String>,
    img: &RgbaImage,
) -> TextureHandle {
    let color_image = rgba_to_color_image(img);
    ctx.load_texture(name, color_image, TextureOptions::NEAREST)
}

/// A cache of uploaded textures keyed by name.
pub struct TextureCache {
    textures: HashMap<String, TextureHandle>,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub fn get_or_upload(
        &mut self,
        ctx: &egui::Context,
        name: &str,
        img: &RgbaImage,
    ) -> &TextureHandle {
        if !self.textures.contains_key(name) {
            let handle = upload_texture(ctx, name, img);
            self.textures.insert(name.to_string(), handle);
        }
        &self.textures[name]
    }

    pub fn clear(&mut self) {
        self.textures.clear();
    }
}
