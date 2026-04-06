use eframe::egui::{self, Color32, CornerRadius, Stroke, StrokeKind, Vec2};

use crate::brush::Tool;
use crate::palette::{COLORS, SWATCH_SIZE};

use super::{BORDER, SWATCH_COLUMNS, SWATCH_GAP};

/// The same 7 rainbow colors used in the header stripe and app icon.
pub(super) const RAINBOW_STRIPE: [Color32; 7] = [
    Color32::from_rgb(255, 59, 48),
    Color32::from_rgb(255, 149, 0),
    Color32::from_rgb(255, 204, 0),
    Color32::from_rgb(76, 217, 100),
    Color32::from_rgb(90, 200, 250),
    Color32::from_rgb(0, 122, 255),
    Color32::from_rgb(175, 82, 222),
];

pub(super) fn rainbow(index: usize) -> Color32 {
    RAINBOW_STRIPE[index % RAINBOW_STRIPE.len()]
}

pub(super) fn show_palette_grid(
    ui: &mut egui::Ui,
    active_color_index: &mut usize,
    active_tool: &Tool,
) {
    ui.add_enabled_ui(!active_tool.is_eraser(), |ui| {
        COLORS
            .chunks(SWATCH_COLUMNS)
            .enumerate()
            .for_each(|(row_index, row)| {
                ui.horizontal(|ui| {
                    let spacing_x = ui.spacing().item_spacing.x;
                    let row_width = row.len() as f32 * SWATCH_SIZE
                        + row.len().saturating_sub(1) as f32 * spacing_x;
                    let left_pad = ((ui.available_width() - row_width) * 0.5).max(0.0);
                    ui.add_space(left_pad);

                    row.iter().enumerate().for_each(|(column_index, color)| {
                        let swatch_index = row_index * SWATCH_COLUMNS + column_index;
                        let mut button = egui::Button::new("")
                            .min_size(Vec2::splat(SWATCH_SIZE))
                            .fill(*color)
                            .corner_radius(CornerRadius::same(10))
                            .stroke(Stroke::new(2.0, BORDER));

                        if swatch_index == *active_color_index {
                            button = button.stroke(Stroke::new(4.0, Color32::BLACK));
                        }

                        let response = ui.add(button);
                        if response.clicked() {
                            *active_color_index = swatch_index;
                        }

                        if swatch_index == *active_color_index {
                            ui.painter().rect_stroke(
                                response.rect.expand(2.0),
                                CornerRadius::same(12),
                                Stroke::new(3.0, Color32::from_rgb(255, 215, 0)),
                                StrokeKind::Outside,
                            );
                        }
                    });
                });
                ui.add_space(SWATCH_GAP);
            });
    });

    ui.add_space(12.0);
}
