use pixelpipe_core::config;
use pixelpipe_core::error::PipelineError;
use pixelpipe_core::pipeline;
use std::path::Path;

pub fn run(
    config_path: &Path,
    _only: Option<String>,
    _dry_run: bool,
) -> pixelpipe_core::error::Result<()> {
    if !config_path.exists() {
        return Err(PipelineError::FileNotFound(config_path.to_path_buf()));
    }

    let cfg = config::load_config(config_path)?;

    let base_dir = config_path
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    println!("Building project '{}'...", cfg.project.name);

    let _ctx = pipeline::run_pipeline(cfg, base_dir)?;

    println!("Build complete.");
    Ok(())
}
