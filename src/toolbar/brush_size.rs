use eframe::egui::{self, Color32, Rect, Stroke, StrokeKind, Vec2};

use crate::palette::BRUSH_SIZES;

use super::{BORDER, MAX_VERTICAL_SLIDER_HEIGHT, MIN_VERTICAL_SLIDER_HEIGHT};

pub(super) fn show_brush_size_control(ui: &mut egui::Ui, brush_size_index: &mut usize) {
    let max_index = BRUSH_SIZES.len().saturating_sub(1).max(1);
    let t = *brush_size_index as f32 / max_index as f32;
    let preview_radius = 1.5 + t * (26.0 - 1.5);

    let remaining_for_slider =
        (ui.available_height() - 2.0).clamp(MIN_VERTICAL_SLIDER_HEIGHT, MAX_VERTICAL_SLIDER_HEIGHT);
    ui.horizontal(|ui| {
        let slider_width = 40.0;
        let info_width = 110.0;
        let group_gap = 14.0;
        let group_width = slider_width + group_gap + info_width;
        let left_pad = ((ui.available_width() - group_width) * 0.5).max(0.0);
        ui.add_space(left_pad);

        let (slider_rect, response) = ui.allocate_exact_size(
            Vec2::new(slider_width, remaining_for_slider),
            egui::Sense::click_and_drag(),
        );

        let rail_rect = Rect::from_center_size(
            slider_rect.center(),
            Vec2::new(8.0, slider_rect.height() - 14.0),
        );
        ui.painter()
            .rect_filled(rail_rect, 6.0, Color32::from_rgb(238, 238, 244));
        ui.painter().rect_stroke(
            rail_rect,
            6.0,
            Stroke::new(2.0, Color32::from_rgb(125, 127, 139)),
            StrokeKind::Outside,
        );

        let max_index = BRUSH_SIZES.len().saturating_sub(1);

        if (response.clicked() || response.dragged())
            && let Some(pointer_pos) = response.interact_pointer_pos()
        {
            let normalized =
                ((rail_rect.bottom() - pointer_pos.y) / rail_rect.height()).clamp(0.0, 1.0);
            let new_index = (normalized * max_index as f32).round() as usize;
            *brush_size_index = new_index.min(max_index);
        }

        let value_t = if max_index == 0 {
            0.0
        } else {
            *brush_size_index as f32 / max_index as f32
        };
        let handle_y = rail_rect.bottom() - value_t * rail_rect.height();
        let handle_rect = Rect::from_center_size(
            egui::pos2(rail_rect.center().x, handle_y),
            Vec2::new(28.0, 16.0),
        );
        ui.painter()
            .rect_filled(handle_rect, 6.0, Color32::from_rgb(64, 64, 68));
        ui.painter().rect_stroke(
            handle_rect,
            6.0,
            Stroke::new(1.0, Color32::from_rgb(24, 24, 28)),
            StrokeKind::Outside,
        );

        ui.add_space(group_gap);
        ui.allocate_ui_with_layout(
            Vec2::new(info_width, remaining_for_slider),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                let (preview_rect, _) =
                    ui.allocate_exact_size(Vec2::new(70.0, 70.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(preview_rect, 12.0, Color32::from_rgb(255, 255, 255));
                ui.painter().rect_stroke(
                    preview_rect,
                    12.0,
                    Stroke::new(2.0, BORDER),
                    StrokeKind::Outside,
                );
                ui.painter()
                    .circle_filled(preview_rect.center(), preview_radius, Color32::BLACK);
            },
        );
    });
}
