use crate::panels;
use crate::state::*;
use crate::texture::TextureCache;
use crate::{drag_drop, panels::build_runner};
use pixelpipe_core::config::schema::Config;
use pixelpipe_core::pipeline::PipelineContext;
use std::path::PathBuf;

pub struct PixelpipeApp {
    // Core data
    pub config_path: PathBuf,
    pub base_dir: PathBuf,
    pub config: Option<Config>,
    pub pipeline_ctx: Option<PipelineContext>,

    // GPU textures
    pub texture_cache: TextureCache,

    // UI state
    pub active_panel: Panel,
    pub atlas_state: AtlasViewerState,
    pub palette_state: PaletteEditorState,
    pub config_state: ConfigEditorState,
    pub build_state: BuildRunnerState,

    // Background build
    pub build_channel: Option<BuildChannel>,
}

impl PixelpipeApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Try to find a pixelpipe.yaml in the current directory
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let config_path = cwd.join("pixelpipe.yaml");
        let base_dir = cwd;

        let mut config = None;
        let mut config_state = ConfigEditorState::default();

        if config_path.exists() {
            if let Ok(text) = std::fs::read_to_string(&config_path) {
                config_state.yaml_text = text;
            }
            if let Ok(cfg) = pixelpipe_core::config::load_config(&config_path) {
                config = Some(cfg);
            }
        }

        Self {
            config_path,
            base_dir,
            config,
            pipeline_ctx: None,
            texture_cache: TextureCache::new(),
            active_panel: Panel::Atlas,
            atlas_state: AtlasViewerState::default(),
            palette_state: PaletteEditorState::default(),
            config_state,
            build_state: BuildRunnerState::default(),
            build_channel: None,
        }
    }
}

impl eframe::App for PixelpipeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll for build results
        build_runner::poll(self);

        // Handle drag-and-drop
        drag_drop::handle(self, ctx);

        // Keyboard shortcuts
        ctx.input(|i| {
            if i.modifiers.command && i.key_pressed(egui::Key::B) {
                build_runner::start_build(self, ctx);
            }
        });

        // Left sidebar
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(160.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading("pixelpipe");
                ui.separator();

                let panels = [
                    (Panel::Atlas, "Atlas Viewer"),
                    (Panel::Palette, "Palette Editor"),
                    (Panel::Animation, "Animation"),
                    (Panel::Config, "Config"),
                ];

                for (panel, label) in &panels {
                    if ui
                        .selectable_label(self.active_panel == *panel, *label)
                        .clicked()
                    {
                        self.active_panel = *panel;
                    }
                }

                ui.separator();

                // Build button
                let build_enabled = self.build_state.status != BuildStatus::Running
                    && self.config_path.exists();

                let build_text = match &self.build_state.status {
                    BuildStatus::Running => "Building...",
                    _ => "Build",
                };

                if ui
                    .add_enabled(build_enabled, egui::Button::new(build_text))
                    .clicked()
                {
                    build_runner::start_build(self, ctx);
                }

                // Status
                match &self.build_state.status {
                    BuildStatus::Idle => {
                        ui.label("Status: Idle");
                    }
                    BuildStatus::Running => {
                        ui.spinner();
                    }
                    BuildStatus::Success { duration_ms } => {
                        ui.colored_label(
                            egui::Color32::GREEN,
                            format!("Done ({}ms)", duration_ms),
                        );
                    }
                    BuildStatus::Failed => {
                        ui.colored_label(egui::Color32::RED, "Build failed");
                    }
                }

                // Config info
                ui.separator();
                if let Some(ref config) = self.config {
                    ui.label(format!("Project: {}", config.project.name));
                    ui.label(format!("Sheets: {}", config.sheets.len()));
                } else {
                    ui.label("No config loaded");
                    ui.label("Drop a .yaml file or");
                    ui.label("run from project dir");
                }

                // Keyboard hint
                ui.separator();
                ui.small("Ctrl+B: Build");
            });

        // Bottom panel for build errors
        if let Some(ref error) = self.build_state.last_error {
            egui::TopBottomPanel::bottom("build_error")
                .resizable(true)
                .max_height(200.0)
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, "Build Error:");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.monospace(error);
                    });
                });
        }

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| match self.active_panel {
            Panel::Atlas => panels::atlas_viewer::draw(self, ui),
            Panel::Palette => panels::palette_editor::draw(self, ui),
            Panel::Animation => panels::animation_preview::draw(self, ui),
            Panel::Config => panels::config_editor::draw(self, ui),
        });
    }
}
