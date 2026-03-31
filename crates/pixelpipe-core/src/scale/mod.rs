use crate::error::Result;
use crate::pipeline::{FramePlacement, PipelineContext, PipelinePhase, ScaledSheet, SheetResult};
use image::imageops::FilterType;
use image::RgbaImage;

pub struct ScalePhase;

impl PipelinePhase for ScalePhase {
    fn name(&self) -> &str {
        "multi-resolution-scaling"
    }

    fn execute(&self, ctx: &mut PipelineContext) -> Result<()> {
        let factors = &ctx.config.scaling.factors;

        if factors.is_empty() || (factors.len() == 1 && factors[0] == 1) {
            log::info!("No additional scale factors configured, skipping scaling phase");
            return Ok(());
        }

        // Scale each packed sheet at each factor
        for (sheet_name, sheet_result) in &ctx.sheets {
            let naming = &ctx.config.scaling.naming;

            for &factor in factors {
                let scaled_name = naming
                    .replace("{name}", sheet_name)
                    .replace("{scale}", &factor.to_string());

                let scaled = scale_sheet(sheet_result, factor);

                log::info!(
                    "Scaled '{}' @{}x -> {}x{}",
                    sheet_name,
                    factor,
                    scaled.image.width(),
                    scaled.image.height()
                );

                ctx.scaled_sheets.push(ScaledSheet {
                    name: scaled_name,
                    sheet_name: sheet_name.clone(),
                    scale_factor: factor,
                    image: scaled.image,
                    frames: scaled.frames,
                    width: scaled.width,
                    height: scaled.height,
                });
            }
        }

        Ok(())
    }
}

/// Scale a sheet's image and all frame coordinates by an integer factor.
fn scale_sheet(sheet: &SheetResult, factor: u32) -> ScaledSheetData {
    let new_width = sheet.width * factor;
    let new_height = sheet.height * factor;

    let image = if factor == 1 {
        sheet.image.clone()
    } else {
        image::imageops::resize(&sheet.image, new_width, new_height, FilterType::Nearest)
    };

    let frames = sheet
        .frames
        .iter()
        .map(|f| FramePlacement {
            name: f.name.clone(),
            x: f.x * factor,
            y: f.y * factor,
            width: f.width * factor,
            height: f.height * factor,
        })
        .collect();

    ScaledSheetData {
        image,
        frames,
        width: new_width,
        height: new_height,
    }
}

struct ScaledSheetData {
    image: RgbaImage,
    frames: Vec<FramePlacement>,
    width: u32,
    height: u32,
}

/// Scale a single image by an integer factor using nearest-neighbor.
pub fn scale_image(image: &RgbaImage, factor: u32) -> RgbaImage {
    if factor == 1 {
        return image.clone();
    }
    let new_w = image.width() * factor;
    let new_h = image.height() * factor;
    image::imageops::resize(image, new_w, new_h, FilterType::Nearest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_scale_image_1x_is_identity() {
        let mut img = RgbaImage::new(4, 4);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        img.put_pixel(3, 3, Rgba([0, 0, 255, 255]));

        let scaled = scale_image(&img, 1);
        assert_eq!(scaled.width(), 4);
        assert_eq!(scaled.height(), 4);
        assert_eq!(scaled.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(scaled.get_pixel(3, 3).0, [0, 0, 255, 255]);
    }

    #[test]
    fn test_scale_image_2x_nearest_neighbor() {
        // 2x2 image with 4 distinct colors
        let mut img = RgbaImage::new(2, 2);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255])); // red
        img.put_pixel(1, 0, Rgba([0, 255, 0, 255])); // green
        img.put_pixel(0, 1, Rgba([0, 0, 255, 255])); // blue
        img.put_pixel(1, 1, Rgba([255, 255, 0, 255])); // yellow

        let scaled = scale_image(&img, 2);
        assert_eq!(scaled.width(), 4);
        assert_eq!(scaled.height(), 4);

        // Each original pixel should become a 2x2 block
        // Red block (top-left)
        assert_eq!(scaled.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(scaled.get_pixel(1, 0).0, [255, 0, 0, 255]);
        assert_eq!(scaled.get_pixel(0, 1).0, [255, 0, 0, 255]);
        assert_eq!(scaled.get_pixel(1, 1).0, [255, 0, 0, 255]);

        // Green block (top-right)
        assert_eq!(scaled.get_pixel(2, 0).0, [0, 255, 0, 255]);
        assert_eq!(scaled.get_pixel(3, 0).0, [0, 255, 0, 255]);
        assert_eq!(scaled.get_pixel(2, 1).0, [0, 255, 0, 255]);
        assert_eq!(scaled.get_pixel(3, 1).0, [0, 255, 0, 255]);

        // Blue block (bottom-left)
        assert_eq!(scaled.get_pixel(0, 2).0, [0, 0, 255, 255]);
        assert_eq!(scaled.get_pixel(1, 3).0, [0, 0, 255, 255]);

        // Yellow block (bottom-right)
        assert_eq!(scaled.get_pixel(2, 2).0, [255, 255, 0, 255]);
        assert_eq!(scaled.get_pixel(3, 3).0, [255, 255, 0, 255]);
    }

    #[test]
    fn test_scale_image_4x() {
        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, Rgba([42, 128, 200, 255]));

        let scaled = scale_image(&img, 4);
        assert_eq!(scaled.width(), 4);
        assert_eq!(scaled.height(), 4);

        // Every pixel should be the same color
        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(scaled.get_pixel(x, y).0, [42, 128, 200, 255]);
            }
        }
    }

    #[test]
    fn test_scale_sheet_coordinates() {
        let sheet = SheetResult {
            image: RgbaImage::new(64, 32),
            frames: vec![
                FramePlacement {
                    name: "a".to_string(),
                    x: 1,
                    y: 1,
                    width: 16,
                    height: 16,
                },
                FramePlacement {
                    name: "b".to_string(),
                    x: 18,
                    y: 1,
                    width: 8,
                    height: 8,
                },
            ],
            width: 64,
            height: 32,
        };

        let scaled = scale_sheet(&sheet, 2);
        assert_eq!(scaled.width, 128);
        assert_eq!(scaled.height, 64);
        assert_eq!(scaled.image.width(), 128);
        assert_eq!(scaled.image.height(), 64);

        assert_eq!(scaled.frames[0].x, 2);
        assert_eq!(scaled.frames[0].y, 2);
        assert_eq!(scaled.frames[0].width, 32);
        assert_eq!(scaled.frames[0].height, 32);

        assert_eq!(scaled.frames[1].x, 36);
        assert_eq!(scaled.frames[1].y, 2);
        assert_eq!(scaled.frames[1].width, 16);
        assert_eq!(scaled.frames[1].height, 16);
    }
}
