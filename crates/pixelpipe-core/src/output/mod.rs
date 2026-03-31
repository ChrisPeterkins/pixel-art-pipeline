pub mod phaser;

use crate::config::schema::OutputFormat;
use crate::error::{PipelineError, Result};
use crate::pipeline::{PipelineContext, PipelinePhase};
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

        // Write sprite sheets
        for sheet_config in &ctx.config.sheets {
            if let Some(sheet_result) = ctx.sheets.get(&sheet_config.name) {
                let base_name = &sheet_config.name;

                // Write the atlas PNG
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

                // Write metadata in each requested format
                for format in &sheet_config.output_formats {
                    match format {
                        OutputFormat::Phaser => {
                            let json = phaser::serialize(sheet_result, &png_name);
                            let json_path = output_dir.join(format!("{}.json", base_name));
                            write_string(&json_path, &json)?;
                            log::info!("Wrote {}", json_path.display());
                        }
                        OutputFormat::Css => {
                            // CSS output will be implemented in Milestone 6
                            log::warn!("CSS output not yet implemented, skipping");
                        }
                        OutputFormat::Canvas => {
                            // Canvas output will be implemented in Milestone 6
                            log::warn!("Canvas JSON output not yet implemented, skipping");
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn write_string(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|e| PipelineError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}
