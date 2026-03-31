pub mod canvas;
pub mod css;
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
            // Collect all scale variant names per sheet for CSS retina queries
            let mut scale_names_by_sheet: std::collections::HashMap<String, Vec<(u32, String)>> =
                std::collections::HashMap::new();
            for scaled in &ctx.scaled_sheets {
                scale_names_by_sheet
                    .entry(scaled.sheet_name.clone())
                    .or_default()
                    .push((scaled.scale_factor, format!("{}.png", scaled.name)));
            }

            for scaled in &ctx.scaled_sheets {
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

                let all_scales = scale_names_by_sheet
                    .get(&scaled.sheet_name)
                    .cloned()
                    .unwrap_or_default();

                write_metadata(
                    &output_dir,
                    &scaled.name,
                    &png_name,
                    &scaled.sheet_name,
                    &sheet_for_output,
                    scaled.scale_factor,
                    output_formats,
                    &all_scales,
                )?;
            }
        } else {
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
                        base_name,
                        sheet_result,
                        1,
                        &sheet_config.output_formats,
                        &[],
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
    sheet_name: &str,
    sheet: &SheetResult,
    scale: u32,
    formats: &[OutputFormat],
    all_scale_names: &[(u32, String)],
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
                let css_content =
                    css::serialize(sheet, png_name, sheet_name, scale, all_scale_names);
                let css_path = output_dir.join(format!("{}.css", base_name));
                write_string(&css_path, &css_content)?;
                log::info!("Wrote {}", css_path.display());
            }
            OutputFormat::Canvas => {
                let json = canvas::serialize(sheet, png_name);
                let json_path = output_dir.join(format!("{}-canvas.json", base_name));
                write_string(&json_path, &json)?;
                log::info!("Wrote {}", json_path.display());
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
