pub mod phaser;

use crate::config::schema::OutputFormat;
use crate::error::{PipelineError, Result};
use crate::pipeline::{FramePlacement, PipelineContext, PipelinePhase, SheetResult};
use std::fs;
use std::path::Path;

pub struct OutputPhase;

impl PipelinePhase for OutputPhase {
    fn name(&self) -> &str {
        "output-serialization"
    }

    fn execute(&self, ctx: &mut PipelineContext) -> Result<()> {
        let output_dir = ctx.base_dir.join(&ctx.config.project.output_dir);
        fs::create_dir_all(&output_dir).map_err(|e| PipelineError::Io {
            path: output_dir.clone(),
            source: e,
        })?;

        if !ctx.scaled_sheets.is_empty() {
            // Write scaled variants
            for scaled in &ctx.scaled_sheets {
                // Find the matching sheet config for output formats
                let sheet_config = ctx
                    .config
                    .sheets
                    .iter()
                    .find(|s| s.name == scaled.sheet_name);

                let output_formats = match sheet_config {
                    Some(cfg) => &cfg.output_formats,
                    None => continue,
                };

                let png_name = format!("{}.png", scaled.name);
                let png_path = output_dir.join(&png_name);
                scaled
                    .image
                    .save(&png_path)
                    .map_err(|e| PipelineError::Image {
                        path: png_path.clone(),
                        source: e,
                    })?;
                log::info!("Wrote {}", png_path.display());

                // Build a temporary SheetResult reference for serialization
                let sheet_for_output = SheetResult {
                    image: scaled.image.clone(),
                    frames: scaled
                        .frames
                        .iter()
                        .map(|f| FramePlacement {
                            name: f.name.clone(),
                            x: f.x,
                            y: f.y,
                            width: f.width,
                            height: f.height,
                        })
                        .collect(),
                    width: scaled.width,
                    height: scaled.height,
                };

                write_metadata(
                    &output_dir,
                    &scaled.name,
                    &png_name,
                    &sheet_for_output,
                    scaled.scale_factor,
                    output_formats,
                )?;
            }
        } else {
            // No scaling configured — write base sheets directly
            for sheet_config in &ctx.config.sheets {
                if let Some(sheet_result) = ctx.sheets.get(&sheet_config.name) {
                    let base_name = &sheet_config.name;
                    let png_name = format!("{}.png", base_name);
                    let png_path = output_dir.join(&png_name);
                    sheet_result
                        .image
                        .save(&png_path)
                        .map_err(|e| PipelineError::Image {
                            path: png_path.clone(),
                            source: e,
                        })?;
                    log::info!("Wrote {}", png_path.display());

                    write_metadata(
                        &output_dir,
                        base_name,
                        &png_name,
                        sheet_result,
                        1,
                        &sheet_config.output_formats,
                    )?;
                }
            }
        }

        Ok(())
    }
}

fn write_metadata(
    output_dir: &Path,
    base_name: &str,
    png_name: &str,
    sheet: &SheetResult,
    scale: u32,
    formats: &[OutputFormat],
) -> Result<()> {
    for format in formats {
        match format {
            OutputFormat::Phaser => {
                let json = phaser::serialize(sheet, png_name, scale);
                let json_path = output_dir.join(format!("{}.json", base_name));
                write_string(&json_path, &json)?;
                log::info!("Wrote {}", json_path.display());
            }
            OutputFormat::Css => {
                log::warn!("CSS output not yet implemented, skipping");
            }
            OutputFormat::Canvas => {
                log::warn!("Canvas JSON output not yet implemented, skipping");
            }
        }
    }
    Ok(())
}

fn write_string(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|e| PipelineError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}
