use crate::app::PixelpipeApp;
use crate::state::{BuildChannel, BuildMessage, BuildStatus};
use std::sync::mpsc;
use std::thread;

pub fn start_build(app: &mut PixelpipeApp, egui_ctx: &egui::Context) {
    if app.build_state.status == BuildStatus::Running {
        return;
    }

    app.build_state.status = BuildStatus::Running;
    app.build_state.last_error = None;

    let (tx, rx) = mpsc::channel::<BuildMessage>();
    app.build_channel = Some(BuildChannel { rx });

    let config_path = app.config_path.clone();
    let base_dir = app.base_dir.clone();
    let repaint_ctx = egui_ctx.clone();

    thread::spawn(move || {
        let start = std::time::Instant::now();

        let config = match pixelpipe_core::config::load_config(&config_path) {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(BuildMessage::Complete(Err(e.to_string())));
                repaint_ctx.request_repaint();
                return;
            }
        };

        match pixelpipe_core::pipeline::run_pipeline(config, base_dir) {
            Ok(ctx) => {
                let duration = start.elapsed().as_millis();
                let _ = tx.send(BuildMessage::Duration(duration));
                let _ = tx.send(BuildMessage::Complete(Ok(ctx)));
            }
            Err(e) => {
                let _ = tx.send(BuildMessage::Complete(Err(e.to_string())));
            }
        }
        repaint_ctx.request_repaint();
    });
}

pub fn poll(app: &mut PixelpipeApp) {
    let Some(ref channel) = app.build_channel else {
        return;
    };

    let mut duration_ms = 0u128;

    while let Ok(msg) = channel.rx.try_recv() {
        match msg {
            BuildMessage::Duration(d) => {
                duration_ms = d;
            }
            BuildMessage::Complete(result) => {
                match result {
                    Ok(pipeline_ctx) => {
                        app.build_state.status = BuildStatus::Success { duration_ms };
                        app.texture_cache.clear();
                        app.palette_state.palettes.clear();
                        app.pipeline_ctx = Some(pipeline_ctx);
                        // Re-parse config
                        if let Ok(config) =
                            pixelpipe_core::config::load_config(&app.config_path)
                        {
                            app.config = Some(config);
                        }
                    }
                    Err(e) => {
                        app.build_state.status = BuildStatus::Failed;
                        app.build_state.last_error = Some(e);
                    }
                }
                app.build_channel = None;
                return;
            }
        }
    }
}
