mod brush_size;
mod footer;
mod header;
mod palette;
mod tools;

use eframe::egui::{self, Color32, Frame, Panel};

use crate::brush::{BrushStyle, Tool};

pub(crate) const TOOLBAR_WIDTH: f32 = 260.0;
pub(super) const TOP_ICON_SIZE: f32 = 58.0;
pub(super) const BOTTOM_ICON_SIZE: f32 = 34.0;
pub(super) const SWATCH_COLUMNS: usize = 4;
pub(super) const SWATCH_GAP: f32 = 6.0;
pub(super) const MIN_VERTICAL_SLIDER_HEIGHT: f32 = 96.0;
pub(super) const MAX_VERTICAL_SLIDER_HEIGHT: f32 = 420.0;

pub(super) const SURFACE_BG: Color32 = Color32::from_rgb(255, 247, 234);
pub(super) const SURFACE_ELEVATED: Color32 = Color32::from_rgb(255, 255, 255);
pub(super) const ACTIVE_BG: Color32 = Color32::from_rgb(189, 227, 255);
pub(super) const BORDER: Color32 = Color32::from_rgb(172, 173, 181);
pub(super) const ICON_COLOR: Color32 = Color32::from_rgb(46, 50, 66);

pub(crate) fn toolbar_width() -> f32 {
    TOOLBAR_WIDTH
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ToolbarActions {
    pub(crate) undo: bool,
    pub(crate) redo: bool,
    pub(crate) save_as: bool,
    pub(crate) open: bool,
    pub(crate) new_drawing: bool,
    pub(crate) quit: bool,
}

pub(crate) fn show_toolbar(
    ui: &mut egui::Ui,
    active_tool: &mut Tool,
    active_style: &mut BrushStyle,
    active_color_index: &mut usize,
    brush_size_index: &mut usize,
    can_undo: bool,
    can_redo: bool,
) -> ToolbarActions {
    let mut actions = ToolbarActions::default();

    Frame::NONE
        .fill(SURFACE_BG)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            // Pin footer to the bottom using an egui panel.
            Panel::bottom("toolbar-footer")
                .resizable(false)
                .show_separator_line(false)
                .frame(Frame::NONE)
                .show_inside(ui, |ui| {
                    footer::show_footer_actions(ui, can_undo, can_redo, &mut actions);
                });

            // Everything else fills the remaining top space.
            header::show_header(ui);
            tools::show_tool_buttons(ui, active_tool);
            tools::show_style_buttons(ui, active_style, active_tool);
            palette::show_palette_grid(ui, active_color_index, active_tool);
            brush_size::show_brush_size_control(ui, brush_size_index);
        });

    actions
}
