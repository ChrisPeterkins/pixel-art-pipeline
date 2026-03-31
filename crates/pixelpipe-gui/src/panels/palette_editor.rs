use crate::app::PixelpipeApp;
use crate::state::LoadedPalette;
use egui::{Color32, CornerRadius, Sense, Stroke, StrokeKind, Vec2};

pub fn draw(app: &mut PixelpipeApp, ui: &mut egui::Ui) {
    ui.heading("Palette Editor");

    // Load palettes from pipeline context if not yet loaded
    if app.palette_state.palettes.is_empty() {
        if let Some(ref ctx) = app.pipeline_ctx {
            for sheet in ctx.sheets.values() {
                let colors = pixelpipe_core::palette::extract::extract_from_image(&sheet.image);
                app.palette_state.palettes.push(LoadedPalette {
                    name: "Extracted (sheet)".to_string(),
                    colors,
                });
            }
        }

        if let Some(ref config) = app.config {
            for def in &config.palettes.definitions {
                if let Some(ref hex_colors) = def.colors {
                    if let Ok(colors) =
                        pixelpipe_core::palette::extract::parse_hex_colors(hex_colors)
                    {
                        app.palette_state.palettes.push(LoadedPalette {
                            name: def.name.clone(),
                            colors,
                        });
                    }
                }
            }
        }
    }

    if app.palette_state.palettes.is_empty() {
        ui.label("No palettes loaded. Run a build or define palettes in your config.");
        return;
    }

    let swatch_size = 24.0;

    for (pal_idx, palette) in app.palette_state.palettes.iter().enumerate() {
        ui.separator();
        ui.label(format!("{} ({} colors)", palette.name, palette.colors.len()));

        ui.horizontal_wrapped(|ui| {
            for (color_idx, color) in palette.colors.iter().enumerate() {
                let fill = Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);

                let is_selected = app.palette_state.selected_palette == Some(pal_idx)
                    && app.palette_state.selected_color == Some(color_idx);

                let (response, painter) =
                    ui.allocate_painter(Vec2::splat(swatch_size), Sense::click());

                painter.rect_filled(response.rect, CornerRadius::same(2), fill);

                if is_selected {
                    painter.rect_stroke(
                        response.rect,
                        CornerRadius::same(2),
                        Stroke::new(2.0, Color32::WHITE),
                        StrokeKind::Outside,
                    );
                }

                if response.clicked() {
                    app.palette_state.selected_palette = Some(pal_idx);
                    app.palette_state.selected_color = Some(color_idx);
                    app.palette_state.edit_color = [
                        color[0] as f32 / 255.0,
                        color[1] as f32 / 255.0,
                        color[2] as f32 / 255.0,
                        color[3] as f32 / 255.0,
                    ];
                }

                if response.hovered() {
                    response.on_hover_text(format!(
                        "#{:02x}{:02x}{:02x}",
                        color[0], color[1], color[2]
                    ));
                }
            }
        });
    }

    // Color editor for selected swatch
    if app.palette_state.selected_palette.is_some() && app.palette_state.selected_color.is_some() {
        ui.separator();
        ui.label("Edit Color:");
        ui.horizontal(|ui| {
            let c = &app.palette_state.edit_color;
            let mut color32 = Color32::from_rgba_unmultiplied(
                (c[0] * 255.0) as u8,
                (c[1] * 255.0) as u8,
                (c[2] * 255.0) as u8,
                (c[3] * 255.0) as u8,
            );
            if egui::color_picker::color_edit_button_srgba(
                ui,
                &mut color32,
                egui::color_picker::Alpha::Opaque,
            )
            .changed()
            {
                app.palette_state.edit_color = [
                    color32.r() as f32 / 255.0,
                    color32.g() as f32 / 255.0,
                    color32.b() as f32 / 255.0,
                    color32.a() as f32 / 255.0,
                ];
            }

            ui.label(format!(
                "#{:02x}{:02x}{:02x}",
                color32.r(),
                color32.g(),
                color32.b()
            ));
        });
    }
}
