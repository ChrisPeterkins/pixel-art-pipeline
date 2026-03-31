use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use pixelpipe_core::error::PipelineError;
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub fn run(config_path: &Path, debounce_ms: u64) -> pixelpipe_core::error::Result<()> {
    if !config_path.exists() {
        return Err(PipelineError::FileNotFound(config_path.to_path_buf()));
    }

    let cfg = pixelpipe_core::config::load_config(config_path)?;
    let base_dir = config_path
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();
    let input_dir = base_dir.join(&cfg.project.input_dir);

    println!("Watching for changes...");
    println!("  Config: {}", config_path.display());
    println!("  Input:  {}", input_dir.display());
    println!("  Debounce: {}ms", debounce_ms);
    println!("  Press Ctrl+C to stop.\n");

    // Initial build
    run_build(config_path);

    // Set up file watcher
    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Create(_)
                    | EventKind::Modify(_)
                    | EventKind::Remove(_) => {
                        let _ = tx.send(());
                    }
                    _ => {}
                }
            }
        },
        Config::default(),
    )
    .map_err(|e| PipelineError::Config(format!("Failed to create file watcher: {}", e)))?;

    // Watch input directory
    if input_dir.exists() {
        watcher
            .watch(&input_dir, RecursiveMode::Recursive)
            .map_err(|e| {
                PipelineError::Config(format!(
                    "Failed to watch {}: {}",
                    input_dir.display(),
                    e
                ))
            })?;
    }

    // Watch config file
    watcher
        .watch(config_path, RecursiveMode::NonRecursive)
        .map_err(|e| {
            PipelineError::Config(format!(
                "Failed to watch {}: {}",
                config_path.display(),
                e
            ))
        })?;

    let debounce = Duration::from_millis(debounce_ms);
    let mut last_build = Instant::now();

    loop {
        match rx.recv() {
            Ok(()) => {
                // Drain any queued events
                while rx.try_recv().is_ok() {}

                // Debounce
                let elapsed = last_build.elapsed();
                if elapsed < debounce {
                    std::thread::sleep(debounce - elapsed);
                    // Drain again after sleep
                    while rx.try_recv().is_ok() {}
                }

                last_build = Instant::now();
                println!("Change detected, rebuilding...");
                run_build(config_path);
            }
            Err(_) => break,
        }
    }

    Ok(())
}

fn run_build(config_path: &Path) {
    let start = Instant::now();

    let base_dir = config_path
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    match pixelpipe_core::config::load_config(config_path) {
        Ok(cfg) => {
            let project_name = cfg.project.name.clone();
            match pixelpipe_core::pipeline::run_pipeline(cfg, base_dir) {
                Ok(_) => {
                    let duration = start.elapsed();
                    println!(
                        "  {} Built '{}' in {:.0?}",
                        colored::Colorize::green("ok"),
                        project_name,
                        duration
                    );
                }
                Err(e) => {
                    eprintln!("  {} {}", colored::Colorize::red("error:"), e);
                }
            }
        }
        Err(e) => {
            eprintln!("  {} {}", colored::Colorize::red("config error:"), e);
        }
    }
}
