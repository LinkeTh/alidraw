use eframe::egui::{self, Color32, CornerRadius, Stroke, Vec2};

use crate::icons::{self, ToolbarIcon};

use super::{BORDER, BOTTOM_ICON_SIZE, ICON_COLOR, ToolbarActions};

pub(super) fn show_footer_actions(
    ui: &mut egui::Ui,
    can_undo: bool,
    can_redo: bool,
    actions: &mut ToolbarActions,
) {
    ui.separator();
    ui.add_space(4.0);

    // Row 1: undo / redo
    ui.horizontal(|ui| {
        if action_icon_button(
            ui,
            ToolbarIcon::Undo,
            Color32::from_rgb(236, 238, 242),
            ICON_COLOR,
            can_undo,
        ) {
            actions.undo = true;
        }
        if action_icon_button(
            ui,
            ToolbarIcon::Redo,
            Color32::from_rgb(236, 238, 242),
            ICON_COLOR,
            can_redo,
        ) {
            actions.redo = true;
        }
    });

    ui.add_space(4.0);

    // Row 2: save / open / new / quit
    ui.horizontal(|ui| {
        if action_icon_button(
            ui,
            ToolbarIcon::Save,
            Color32::from_rgb(214, 244, 219),
            ICON_COLOR,
            true,
        ) {
            actions.save_as = true;
        }
        if action_icon_button(
            ui,
            ToolbarIcon::Open,
            Color32::from_rgb(214, 232, 255),
            ICON_COLOR,
            true,
        ) {
            actions.open = true;
        }
        if action_icon_button(
            ui,
            ToolbarIcon::New,
            Color32::from_rgb(255, 244, 213),
            ICON_COLOR,
            true,
        ) {
            actions.new_drawing = true;
        }
        if action_icon_button(
            ui,
            ToolbarIcon::Quit,
            Color32::from_rgb(255, 226, 226),
            ICON_COLOR,
            true,
        ) {
            actions.quit = true;
        }
    });
}

fn action_icon_button(
    ui: &mut egui::Ui,
    icon: ToolbarIcon,
    fill: Color32,
    icon_color: Color32,
    enabled: bool,
) -> bool {
    let button = egui::Button::new("")
        .min_size(Vec2::new(BOTTOM_ICON_SIZE + 6.0, BOTTOM_ICON_SIZE))
        .fill(fill)
        .stroke(Stroke::new(2.0, BORDER))
        .corner_radius(CornerRadius::same(10));
    let response = ui.add_enabled(enabled, button);
    let paint_color = if enabled {
        icon_color
    } else {
        Color32::from_gray(140)
    };
    ui.painter().text(
        response.rect.center(),
        egui::Align2::CENTER_CENTER,
        icons::icon_glyph(icon),
        egui::FontId::new(
            BOTTOM_ICON_SIZE * 0.68,
            egui::FontFamily::Name("lucide".into()),
        ),
        paint_color,
    );
    response.clicked()
}
