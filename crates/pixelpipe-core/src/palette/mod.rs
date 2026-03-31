pub mod constraint;
pub mod extract;
pub mod swap;

use crate::config::resolve_input_files;
use crate::config::schema::{EnforceStrategy, PaletteOperation};
use crate::error::{PipelineError, Result};
use crate::pipeline::{PipelineContext, PipelinePhase};
use extract::Palette;
use std::collections::HashMap;
use std::path::Path;

/// Phase that loads palette definitions and runs enforce/swap operations.
pub struct PalettePhase;

impl PipelinePhase for PalettePhase {
    fn name(&self) -> &str {
        "palette-management"
    }

    fn execute(&self, ctx: &mut PipelineContext) -> Result<()> {
        let defs = &ctx.config.palettes.definitions;
        let ops = &ctx.config.palettes.operations;

        if defs.is_empty() && ops.is_empty() {
            log::info!("No palettes configured, skipping palette phase");
            return Ok(());
        }

        let resolve_dir = ctx.base_dir.join(&ctx.config.project.input_dir);

        // Load all palette definitions
        let mut palettes: HashMap<String, Palette> = HashMap::new();
        for def in defs {
            let colors = if let Some(ref source) = def.source {
                let source_path = resolve_dir.join(source);
                extract::load_from_source(&source_path, def.max_colors)?
            } else if let Some(ref hex_colors) = def.colors {
                extract::parse_hex_colors(hex_colors)?
            } else {
                return Err(PipelineError::Palette(format!(
                    "Palette '{}' has neither 'source' nor 'colors'",
                    def.name
                )));
            };

            log::info!("Loaded palette '{}': {} colors", def.name, colors.len());
            palettes.insert(
                def.name.clone(),
                Palette {
                    name: def.name.clone(),
                    colors,
                },
            );
        }

        // Run operations
        for op in ops {
            match op {
                PaletteOperation::Enforce {
                    palette: palette_name,
                    targets,
                    strategy,
                } => {
                    run_enforce(&resolve_dir, &palettes, palette_name, targets, strategy)?;
                }
                PaletteOperation::Swap {
                    source_palette,
                    target_palette,
                    inputs,
                    output_dir,
                    output_suffix,
                } => {
                    let out_dir = output_dir
                        .as_ref()
                        .map(|p| ctx.base_dir.join(p))
                        .unwrap_or_else(|| ctx.base_dir.join(&ctx.config.project.output_dir));
                    let suffix = output_suffix.as_deref().unwrap_or("_swapped");

                    run_swap(
                        &resolve_dir,
                        &palettes,
                        source_palette,
                        target_palette,
                        inputs,
                        &out_dir,
                        suffix,
                    )?;
                }
            }
        }

        Ok(())
    }
}

fn get_palette<'a>(
    palettes: &'a HashMap<String, Palette>,
    name: &str,
) -> Result<&'a Palette> {
    palettes.get(name).ok_or_else(|| {
        PipelineError::Palette(format!("Palette '{}' not found in definitions", name))
    })
}

fn run_enforce(
    resolve_dir: &Path,
    palettes: &HashMap<String, Palette>,
    palette_name: &str,
    targets: &[String],
    strategy: &EnforceStrategy,
) -> Result<()> {
    let palette = get_palette(palettes, palette_name)?;
    let files = resolve_input_files(resolve_dir, targets)?;

    log::info!(
        "Enforcing palette '{}' on {} files (strategy: {:?})",
        palette_name,
        files.len(),
        strategy
    );

    for file_path in &files {
        let img = image::open(file_path)
            .map_err(|e| PipelineError::Image {
                path: file_path.clone(),
                source: e,
            })?
            .to_rgba8();

        let file_name = file_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        match strategy {
            EnforceStrategy::Nearest => {
                let enforced = constraint::enforce_nearest(&img, &palette.colors);
                enforced
                    .save(file_path)
                    .map_err(|e| PipelineError::Image {
                        path: file_path.clone(),
                        source: e,
                    })?;
                log::info!("  Enforced (nearest) on {}", file_name);
            }
            EnforceStrategy::Error => {
                constraint::enforce_error(&img, &palette.colors, file_name)?;
                log::info!("  Validated {}", file_name);
            }
            EnforceStrategy::Dither => {
                // Dithering is a future enhancement
                log::warn!("  Dither strategy not yet implemented, using nearest for {}", file_name);
                let enforced = constraint::enforce_nearest(&img, &palette.colors);
                enforced
                    .save(file_path)
                    .map_err(|e| PipelineError::Image {
                        path: file_path.clone(),
                        source: e,
                    })?;
            }
        }
    }

    Ok(())
}

fn run_swap(
    resolve_dir: &Path,
    palettes: &HashMap<String, Palette>,
    source_name: &str,
    target_name: &str,
    inputs: &[String],
    output_dir: &Path,
    suffix: &str,
) -> Result<()> {
    let source = get_palette(palettes, source_name)?;
    let target = get_palette(palettes, target_name)?;

    if source.colors.len() != target.colors.len() {
        return Err(PipelineError::Palette(format!(
            "Palette swap requires equal length palettes: '{}' has {} colors, '{}' has {}",
            source_name,
            source.colors.len(),
            target_name,
            target.colors.len()
        )));
    }

    let color_map = swap::build_color_map(&source.colors, &target.colors);
    let files = resolve_input_files(resolve_dir, inputs)?;

    std::fs::create_dir_all(output_dir).map_err(|e| PipelineError::Io {
        path: output_dir.to_path_buf(),
        source: e,
    })?;

    log::info!(
        "Swapping '{}' → '{}' on {} files",
        source_name,
        target_name,
        files.len()
    );

    for file_path in &files {
        let img = image::open(file_path)
            .map_err(|e| PipelineError::Image {
                path: file_path.clone(),
                source: e,
            })?
            .to_rgba8();

        let swapped = swap::swap_colors(&img, &color_map);

        let stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let ext = file_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png");
        let out_name = format!("{}{}.{}", stem, suffix, ext);
        let out_path = output_dir.join(&out_name);

        swapped
            .save(&out_path)
            .map_err(|e| PipelineError::Image {
                path: out_path.clone(),
                source: e,
            })?;
        log::info!("  Wrote {}", out_path.display());
    }

    Ok(())
}
