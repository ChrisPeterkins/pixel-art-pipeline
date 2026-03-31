use indicatif::{ProgressBar, ProgressStyle};
use pixelpipe_core::config;
use pixelpipe_core::error::PipelineError;
use pixelpipe_core::pipeline::{self, BuildSummary, PipelineOptions};
use std::path::Path;
use std::time::Instant;

pub fn run(
    config_path: &Path,
    only: Option<String>,
    dry_run: bool,
) -> pixelpipe_core::error::Result<()> {
    if !config_path.exists() {
        return Err(PipelineError::FileNotFound(config_path.to_path_buf()));
    }

    let cfg = config::load_config(config_path)?;
    let project_name = cfg.project.name.clone();

    let base_dir = config_path
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    if dry_run {
        println!(
            "{} Dry run for project '{}'",
            colored::Colorize::cyan("info:"),
            project_name
        );
    } else {
        println!("Building project '{}'...", project_name);
    }

    if let Some(ref phase) = only {
        println!("  Running only: {}", phase);
    }

    // Progress spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Running pipeline...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let start = Instant::now();
    let options = PipelineOptions { dry_run, only };
    let ctx = pipeline::run_pipeline_with_options(cfg, base_dir, options)?;
    let duration = start.elapsed();

    spinner.finish_and_clear();

    // Build summary
    let summary = BuildSummary::from_context(&ctx);

    println!();
    if dry_run {
        println!(
            "{} Dry run complete in {:.0?}",
            colored::Colorize::green("ok"),
            duration
        );
    } else {
        println!(
            "{} Built '{}' in {:.0?}",
            colored::Colorize::green("ok"),
            project_name,
            duration
        );
    }

    // Print summary details
    if summary.sheets_packed > 0 {
        println!(
            "  Sheets: {} packed{}",
            summary.sheets_packed,
            if summary.scaled_variants > 0 {
                format!(" ({} scaled variants)", summary.scaled_variants)
            } else {
                String::new()
            }
        );
    }
    if summary.animations_assembled > 0 {
        println!("  Animations: {} assembled", summary.animations_assembled);
    }
    if !dry_run && summary.files_written > 0 {
        println!(
            "  Output: {} files written to {}",
            summary.files_written,
            ctx.config.project.output_dir.display()
        );
    }

    Ok(())
}
