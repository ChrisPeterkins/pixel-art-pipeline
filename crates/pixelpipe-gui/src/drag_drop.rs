use crate::app::PixelpipeApp;
use std::path::PathBuf;

pub fn handle(app: &mut PixelpipeApp, ctx: &egui::Context) {
    let dropped = ctx.input(|i| i.raw.dropped_files.clone());

    for file in &dropped {
        if let Some(ref path) = file.path {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            match ext.as_str() {
                "png" => import_sprite(app, path),
                "yaml" | "yml" => load_config(app, path),
                _ => {
                    log::warn!("Ignoring dropped file with unsupported extension: {}", ext);
                }
            }
        }
    }
}

fn import_sprite(app: &mut PixelpipeApp, path: &PathBuf) {
    let input_dir = app.base_dir.join(
        app.config
            .as_ref()
            .map(|c| c.project.input_dir.clone())
            .unwrap_or_else(|| PathBuf::from("raw")),
    );

    // Try to find the first sheet's input subdirectory
    let dest_dir = app
        .config
        .as_ref()
        .and_then(|c| c.sheets.first())
        .and_then(|s| s.inputs.first())
        .and_then(|p| {
            // Extract directory portion from glob pattern like "sprites/*.png"
            let parts: Vec<&str> = p.split('/').collect();
            if parts.len() > 1 {
                Some(input_dir.join(parts[0]))
            } else {
                None
            }
        })
        .unwrap_or(input_dir);

    if let Err(e) = std::fs::create_dir_all(&dest_dir) {
        log::error!("Failed to create directory {}: {}", dest_dir.display(), e);
        return;
    }

    let file_name = path.file_name().unwrap_or_default();
    let dest = dest_dir.join(file_name);

    match std::fs::copy(path, &dest) {
        Ok(_) => log::info!("Imported {} to {}", path.display(), dest.display()),
        Err(e) => log::error!("Failed to import {}: {}", path.display(), e),
    }
}

fn load_config(app: &mut PixelpipeApp, path: &PathBuf) {
    app.config_path = path.clone();
    if let Some(parent) = path.parent() {
        app.base_dir = parent.to_path_buf();
    }

    if let Ok(text) = std::fs::read_to_string(path) {
        app.config_state.yaml_text = text;
        app.config_state.dirty = false;
    }

    if let Ok(config) = pixelpipe_core::config::load_config(path) {
        app.config = Some(config);
    }

    // Clear stale state
    app.pipeline_ctx = None;
    app.texture_cache.clear();
    app.palette_state.palettes.clear();
    app.atlas_state.selected_sheet = None;
}
