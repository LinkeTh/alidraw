use eframe::egui::{self, Color32, CornerRadius, Stroke, Vec2};

use crate::brush::{BrushStyle, Tool};
use crate::icons::{self, ToolbarIcon};
use crate::palette::BRUSH_STYLES;

use super::{ACTIVE_BG, BORDER, ICON_COLOR, SURFACE_ELEVATED, TOP_ICON_SIZE};

pub(super) fn show_tool_buttons(ui: &mut egui::Ui, active_tool: &mut Tool) {
    ui.horizontal(|ui| {
        if tool_icon_button(
            ui,
            ToolbarIcon::Brush,
            !active_tool.is_eraser(),
            true,
            TOP_ICON_SIZE,
        ) {
            *active_tool = Tool::Brush;
        }
        if tool_icon_button(
            ui,
            ToolbarIcon::Eraser,
            active_tool.is_eraser(),
            true,
            TOP_ICON_SIZE,
        ) {
            *active_tool = Tool::Eraser;
        }
    });

    ui.add_space(8.0);
}

pub(super) fn show_style_buttons(
    ui: &mut egui::Ui,
    active_style: &mut BrushStyle,
    active_tool: &Tool,
) {
    ui.horizontal(|ui| {
        BRUSH_STYLES.iter().copied().for_each(|style| {
            let icon = match style {
                BrushStyle::Pencil => ToolbarIcon::Pencil,
                BrushStyle::Marker => ToolbarIcon::Marker,
                BrushStyle::Watercolor => ToolbarIcon::Watercolor,
            };
            if tool_icon_button(
                ui,
                icon,
                *active_style == style,
                !active_tool.is_eraser(),
                TOP_ICON_SIZE,
            ) {
                *active_style = style;
            }
        });
    });

    ui.add_space(10.0);
}

fn tool_icon_button(
    ui: &mut egui::Ui,
    icon: ToolbarIcon,
    is_active: bool,
    enabled: bool,
    size: f32,
) -> bool {
    let base_fill = if is_active {
        ACTIVE_BG
    } else {
        SURFACE_ELEVATED
    };
    let button = egui::Button::new("")
        .min_size(Vec2::splat(size))
        .fill(base_fill)
        .stroke(if is_active {
            Stroke::new(3.0, Color32::from_rgb(27, 103, 193))
        } else {
            Stroke::new(2.0, BORDER)
        })
        .corner_radius(CornerRadius::same(12));

    let response = ui.add_enabled(enabled, button);
    let icon_color = if enabled {
        ICON_COLOR
    } else {
        Color32::from_gray(145)
    };
    ui.painter().text(
        response.rect.center(),
        egui::Align2::CENTER_CENTER,
        icons::icon_glyph(icon),
        egui::FontId::new(size * 0.58, egui::FontFamily::Name("lucide".into())),
        icon_color,
    );
    response.clicked()
}
