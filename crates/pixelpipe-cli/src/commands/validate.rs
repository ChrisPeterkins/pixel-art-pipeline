use pixelpipe_core::config;
use pixelpipe_core::error::PipelineError;
use std::path::Path;

pub fn run(config_path: &Path) -> pixelpipe_core::error::Result<()> {
    if !config_path.exists() {
        return Err(PipelineError::FileNotFound(config_path.to_path_buf()));
    }

    println!("Validating {}...", config_path.display());

    let cfg = config::load_config(config_path)?;

    println!("  Project: {}", cfg.project.name);
    println!("  Input dir: {}", cfg.project.input_dir.display());
    println!("  Output dir: {}", cfg.project.output_dir.display());

    if !cfg.sheets.is_empty() {
        println!("  Sheets: {}", cfg.sheets.len());
        for sheet in &cfg.sheets {
            println!("    - {} ({} input patterns)", sheet.name, sheet.inputs.len());
        }
    }

    if !cfg.palettes.definitions.is_empty() {
        println!("  Palettes: {}", cfg.palettes.definitions.len());
        for palette in &cfg.palettes.definitions {
            println!("    - {}", palette.name);
        }
    }

    if !cfg.palettes.operations.is_empty() {
        println!(
            "  Palette operations: {}",
            cfg.palettes.operations.len()
        );
    }

    if !cfg.animations.is_empty() {
        println!("  Animations: {}", cfg.animations.len());
        for anim in &cfg.animations {
            println!(
                "    - {} ({} frame sources, {} outputs)",
                anim.name,
                anim.frames.len(),
                anim.outputs.len()
            );
        }
    }

    println!("  Scale factors: {:?}", cfg.scaling.factors);

    // Check that input directory exists
    if !cfg.project.input_dir.exists() {
        println!(
            "  Warning: input directory '{}' does not exist",
            cfg.project.input_dir.display()
        );
    }

    println!("\nConfig is valid.");
    Ok(())
}
