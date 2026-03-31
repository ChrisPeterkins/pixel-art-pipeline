pub mod maxrects;

use crate::config::schema::SheetConfig;
use crate::config::resolve_input_files;
use crate::error::{PipelineError, Result};
use crate::pipeline::{FramePlacement, PipelineContext, PipelinePhase, SheetResult};
use image::RgbaImage;
use maxrects::MaxRectsPacker;
use std::path::Path;

pub struct PackPhase;

impl PipelinePhase for PackPhase {
    fn name(&self) -> &str {
        "sprite-sheet-packing"
    }

    fn execute(&self, ctx: &mut PipelineContext) -> Result<()> {
        if ctx.config.sheets.is_empty() {
            log::info!("No sheets configured, skipping packing phase");
            return Ok(());
        }

        for sheet_config in &ctx.config.sheets {
            log::info!("Packing sheet '{}'", sheet_config.name);
            let result = pack_sheet(sheet_config, &ctx.config.defaults, &ctx.base_dir, &ctx.config.project.input_dir)?;
            ctx.sheets.insert(sheet_config.name.clone(), result);
        }

        Ok(())
    }
}

/// Pack a single sprite sheet from its config.
fn pack_sheet(
    sheet: &SheetConfig,
    defaults: &crate::config::schema::Defaults,
    base_dir: &Path,
    input_dir: &Path,
) -> Result<SheetResult> {
    let resolve_dir = base_dir.join(input_dir);
    let files = resolve_input_files(&resolve_dir, &sheet.inputs)?;

    // Load all input images
    let mut sprites: Vec<(String, RgbaImage)> = Vec::new();
    for file_path in &files {
        let img = image::open(file_path)
            .map_err(|e| PipelineError::Image {
                path: file_path.clone(),
                source: e,
            })?
            .to_rgba8();

        let name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        sprites.push((name, img));
    }

    if sprites.is_empty() {
        return Err(PipelineError::Packing(format!(
            "No sprites found for sheet '{}'",
            sheet.name
        )));
    }

    let padding = sheet.padding.unwrap_or(defaults.padding);
    let max_size = sheet.max_sheet_size.unwrap_or(defaults.max_sheet_size);
    let power_of_two = sheet.power_of_two.unwrap_or(defaults.power_of_two);

    // Build rect list for the packer
    let rects: Vec<(usize, u32, u32)> = sprites
        .iter()
        .enumerate()
        .map(|(i, (_, img))| (i, img.width(), img.height()))
        .collect();

    let packer = MaxRectsPacker::new(max_size, max_size, padding, power_of_two);
    let pack_result = packer
        .pack(rects)
        .map_err(|e| PipelineError::Packing(format!("Sheet '{}': {}", sheet.name, e)))?;

    // Composite sprites onto atlas canvas
    let mut atlas = RgbaImage::new(pack_result.width, pack_result.height);
    let mut frames = Vec::new();

    for placement in &pack_result.placements {
        let (ref name, ref img) = sprites[placement.id];
        image::imageops::overlay(&mut atlas, img, placement.x as i64, placement.y as i64);

        frames.push(FramePlacement {
            name: name.clone(),
            x: placement.x,
            y: placement.y,
            width: placement.width,
            height: placement.height,
        });
    }

    log::info!(
        "Sheet '{}': packed {} sprites into {}x{} atlas",
        sheet.name,
        frames.len(),
        pack_result.width,
        pack_result.height
    );

    Ok(SheetResult {
        image: atlas,
        frames,
        width: pack_result.width,
        height: pack_result.height,
    })
}
