use crate::app::PixelpipeApp;
use pixelpipe_core::config::schema::Config;

pub fn draw(app: &mut PixelpipeApp, ui: &mut egui::Ui) {
    ui.heading("Config Editor");

    // Load config text from file on first view
    if app.config_state.yaml_text.is_empty() && app.config_path.exists() {
        if let Ok(text) = std::fs::read_to_string(&app.config_path) {
            app.config_state.yaml_text = text;
        }
    }

    // Toolbar
    ui.horizontal(|ui| {
        ui.label(format!("{}", app.config_path.display()));

        if app.config_state.dirty {
            ui.label(egui::RichText::new("(unsaved)").color(egui::Color32::YELLOW));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Reload").clicked() {
                if let Ok(text) = std::fs::read_to_string(&app.config_path) {
                    app.config_state.yaml_text = text;
                    app.config_state.dirty = false;
                    app.config_state.validation_error = None;
                    app.config_state.validation_ok = false;
                }
            }

            if ui.button("Validate").clicked() {
                match serde_yaml_ng::from_str::<Config>(&app.config_state.yaml_text) {
                    Ok(_) => {
                        app.config_state.validation_error = None;
                        app.config_state.validation_ok = true;
                    }
                    Err(e) => {
                        app.config_state.validation_error = Some(e.to_string());
                        app.config_state.validation_ok = false;
                    }
                }
            }

            if ui.button("Save").clicked() {
                if let Err(e) = std::fs::write(&app.config_path, &app.config_state.yaml_text) {
                    app.config_state.validation_error = Some(format!("Save failed: {}", e));
                } else {
                    app.config_state.dirty = false;
                    // Re-parse config for the app
                    if let Ok(config) =
                        serde_yaml_ng::from_str::<Config>(&app.config_state.yaml_text)
                    {
                        app.config = Some(config);
                    }
                }
            }
        });
    });

    // Validation status
    if let Some(ref error) = app.config_state.validation_error {
        ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
    } else if app.config_state.validation_ok {
        ui.colored_label(egui::Color32::GREEN, "Config is valid.");
    }

    ui.separator();

    // YAML text editor
    let response = ui.add(
        egui::TextEdit::multiline(&mut app.config_state.yaml_text)
            .font(egui::TextStyle::Monospace)
            .desired_width(f32::INFINITY)
            .desired_rows(30),
    );

    if response.changed() {
        app.config_state.dirty = true;
        app.config_state.validation_ok = false;
    }
}
