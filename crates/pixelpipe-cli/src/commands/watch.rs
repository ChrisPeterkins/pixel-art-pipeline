use pixelpipe_core::error::PipelineError;
use std::path::Path;

pub fn run(config_path: &Path, _debounce: u64) -> pixelpipe_core::error::Result<()> {
    if !config_path.exists() {
        return Err(PipelineError::FileNotFound(config_path.to_path_buf()));
    }

    // Watch mode will be implemented in Milestone 6
    println!("Watch mode is not yet implemented.");
    println!("For now, use `pixelpipe build` to run the pipeline manually.");
    Ok(())
}
