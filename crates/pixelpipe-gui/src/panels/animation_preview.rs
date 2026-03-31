use crate::app::PixelpipeApp;

pub fn draw(app: &mut PixelpipeApp, ui: &mut egui::Ui) {
    ui.heading("Animation Preview");

    let Some(ref ctx) = app.pipeline_ctx else {
        ui.label("No build results. Run the pipeline first.");
        return;
    };

    if ctx.animations.is_empty() {
        ui.separator();
        ui.label("No animations in this project.");
        ui.add_space(12.0);
        ui.label("Animation assembly (Milestone 5) is not yet implemented in pixelpipe-core.");
        ui.add_space(8.0);
        ui.label("To use this panel:");
        ui.label("  1. Add an 'animations' section to your pixelpipe.yaml");
        ui.label("  2. Define frame patterns, timing, and output types");
        ui.label("  3. Once core support lands, animations will appear here");
        ui.add_space(12.0);
        ui.label("Example config:");
        ui.code(
            r#"animations:
  - name: "hero-walk"
    frames:
      - pattern: "characters/hero/walk_*.png"
        sort: "natural"
    timing:
      frame_duration_ms: 100
    outputs:
      - type: "gif"
        loop: true
      - type: "strip"
        direction: "horizontal"
        metadata: true"#,
        );
    }
}
