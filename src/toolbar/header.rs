use eframe::egui::{self, RichText};

use super::palette::rainbow;

pub(super) fn show_header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("AliDraw")
                .size(34.0)
                .color(egui::Color32::from_rgb(21, 124, 236))
                .strong(),
        );
    });

    let (stripe_rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 8.0), egui::Sense::hover());
    (0..7).for_each(|index| {
        let segment = egui::Rect::from_min_max(
            egui::pos2(
                stripe_rect.left() + stripe_rect.width() * index as f32 / 7.0,
                stripe_rect.top(),
            ),
            egui::pos2(
                stripe_rect.left() + stripe_rect.width() * (index + 1) as f32 / 7.0,
                stripe_rect.bottom(),
            ),
        );
        ui.painter().rect_filled(segment, 0.0, rainbow(index));
    });

    ui.add_space(10.0);
}
