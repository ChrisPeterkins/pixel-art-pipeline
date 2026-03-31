use crate::app::PixelpipeApp;
use egui::{Color32, CornerRadius, Rect, Stroke, StrokeKind, Vec2};

pub fn draw(app: &mut PixelpipeApp, ui: &mut egui::Ui) {
    let Some(ref ctx) = app.pipeline_ctx else {
        ui.centered_and_justified(|ui| {
            ui.label("No build results. Click Build to run the pipeline.");
        });
        return;
    };

    // Collect sheet names
    let mut sheet_names: Vec<String> = ctx.sheets.keys().cloned().collect();
    sheet_names.sort();

    if sheet_names.is_empty() {
        ui.label("No sheets in build output.");
        return;
    }

    // Default selection to first sheet
    if app.atlas_state.selected_sheet.is_none() {
        app.atlas_state.selected_sheet = Some(sheet_names[0].clone());
    }

    // Toolbar
    ui.horizontal(|ui| {
        ui.label("Sheet:");
        let current = app
            .atlas_state
            .selected_sheet
            .clone()
            .unwrap_or_default();
        egui::ComboBox::from_id_salt("sheet_selector")
            .selected_text(&current)
            .show_ui(ui, |ui| {
                for name in &sheet_names {
                    if ui.selectable_label(&current == name, name).clicked() {
                        app.atlas_state.selected_sheet = Some(name.clone());
                        app.atlas_state.selected_frame = None;
                    }
                }
            });

        ui.separator();

        if ui.button(" - ").clicked() {
            app.atlas_state.zoom = (app.atlas_state.zoom - 1.0).max(1.0);
        }
        ui.label(format!("{:.0}x", app.atlas_state.zoom));
        if ui.button(" + ").clicked() {
            app.atlas_state.zoom = (app.atlas_state.zoom + 1.0).min(16.0);
        }

        ui.separator();
        ui.checkbox(&mut app.atlas_state.show_frame_borders, "Borders");
    });

    ui.separator();

    let selected_sheet_name = match &app.atlas_state.selected_sheet {
        Some(name) => name.clone(),
        None => return,
    };

    // Get the sheet data (re-borrow ctx)
    let Some(ref ctx) = app.pipeline_ctx else {
        return;
    };
    let Some(sheet) = ctx.sheets.get(&selected_sheet_name) else {
        return;
    };

    // Upload texture
    let texture = app
        .texture_cache
        .get_or_upload(ui.ctx(), &selected_sheet_name, &sheet.image);

    let zoom = app.atlas_state.zoom;
    let display_size = Vec2::new(sheet.width as f32 * zoom, sheet.height as f32 * zoom);

    // Scrollable area with the atlas
    egui::ScrollArea::both().show(ui, |ui| {
        let (response, painter) =
            ui.allocate_painter(display_size, egui::Sense::click());

        let rect = response.rect;

        // Draw checkerboard background for transparency
        painter.rect_filled(rect, CornerRadius::ZERO, Color32::from_gray(40));

        // Draw the atlas image
        painter.image(
            texture.id(),
            rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        // Draw frame borders
        if app.atlas_state.show_frame_borders {
            for (i, frame) in sheet.frames.iter().enumerate() {
                let frame_rect = Rect::from_min_size(
                    egui::pos2(
                        rect.min.x + frame.x as f32 * zoom,
                        rect.min.y + frame.y as f32 * zoom,
                    ),
                    Vec2::new(frame.width as f32 * zoom, frame.height as f32 * zoom),
                );

                let color = if Some(i) == app.atlas_state.selected_frame {
                    Color32::from_rgb(255, 220, 50)
                } else {
                    Color32::from_rgba_unmultiplied(100, 200, 255, 180)
                };

                painter.rect_stroke(
                    frame_rect,
                    CornerRadius::ZERO,
                    Stroke::new(1.0, color),
                    StrokeKind::Outside,
                );
            }
        }

        // Handle click to select frame
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let rel_x = ((pos.x - rect.min.x) / zoom) as u32;
                let rel_y = ((pos.y - rect.min.y) / zoom) as u32;

                let mut found = None;
                for (i, frame) in sheet.frames.iter().enumerate() {
                    if rel_x >= frame.x
                        && rel_x < frame.x + frame.width
                        && rel_y >= frame.y
                        && rel_y < frame.y + frame.height
                    {
                        found = Some(i);
                        break;
                    }
                }
                app.atlas_state.selected_frame = found;
            }
        }
    });

    // Selected frame info
    if let Some(idx) = app.atlas_state.selected_frame {
        let Some(ref ctx) = app.pipeline_ctx else {
            return;
        };
        if let Some(sheet) = ctx.sheets.get(&selected_sheet_name) {
            if let Some(frame) = sheet.frames.get(idx) {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Selected: {}  |  pos: ({}, {})  |  size: {}x{}",
                        frame.name, frame.x, frame.y, frame.width, frame.height
                    ));
                });
            }
        }
    }
}
