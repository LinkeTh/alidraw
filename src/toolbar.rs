use eframe::egui::{
    self, Color32, CornerRadius, Frame, Rect, RichText, Stroke, StrokeKind, UiBuilder, Vec2,
};

use crate::brush::{BrushStyle, Tool};
use crate::icons::{self, ToolbarIcon};
use crate::palette::{self, BRUSH_SIZES, BRUSH_STYLES, COLORS, SWATCH_SIZE};

const TOOLBAR_WIDTH: f32 = 260.0;
const TOP_ICON_SIZE: f32 = 58.0;
const BOTTOM_ICON_SIZE: f32 = 34.0;
const SWATCH_COLUMNS: usize = 4;
const SWATCH_GAP: f32 = 6.0;
const MAX_VERTICAL_SLIDER_HEIGHT: f32 = 1600.0;

const SURFACE_BG: Color32 = Color32::from_rgb(255, 247, 234);
const SURFACE_ELEVATED: Color32 = Color32::from_rgb(255, 255, 255);
const ACTIVE_BG: Color32 = Color32::from_rgb(189, 227, 255);
const BORDER: Color32 = Color32::from_rgb(172, 173, 181);
const ICON_COLOR: Color32 = Color32::from_rgb(46, 50, 66);

pub(crate) fn toolbar_width() -> f32 {
    TOOLBAR_WIDTH
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ToolbarActions {
    pub(crate) undo: bool,
    pub(crate) redo: bool,
    pub(crate) save_as: bool,
    pub(crate) new_drawing: bool,
    pub(crate) quit: bool,
}

fn rainbow(index: usize) -> Color32 {
    const RAINBOW_STRIPE: [Color32; 7] = [
        Color32::from_rgb(255, 59, 48),
        Color32::from_rgb(255, 149, 0),
        Color32::from_rgb(255, 204, 0),
        Color32::from_rgb(76, 217, 100),
        Color32::from_rgb(90, 200, 250),
        Color32::from_rgb(0, 122, 255),
        Color32::from_rgb(175, 82, 222),
    ];
    RAINBOW_STRIPE[index % RAINBOW_STRIPE.len()]
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
            let full_rect = ui.max_rect();
            let actions_row_h = BOTTOM_ICON_SIZE + 12.0;
            let top_rect = egui::Rect::from_min_max(
                full_rect.min,
                egui::pos2(full_rect.max.x, full_rect.max.y - actions_row_h),
            );
            let bottom_rect = egui::Rect::from_min_max(
                egui::pos2(full_rect.min.x, full_rect.max.y - actions_row_h),
                full_rect.max,
            );

            ui.scope_builder(
                UiBuilder::new().id_salt("toolbar-top").max_rect(top_rect),
                |ui| {
                    show_header(ui);
                    show_tool_buttons(ui, active_tool);
                    show_style_buttons(ui, active_style, active_tool);
                    show_palette_grid(ui, active_color_index, active_tool);
                    show_brush_size_control(
                        ui,
                        brush_size_index,
                        active_tool,
                        active_style,
                        *active_color_index,
                    );
                },
            );

            show_footer_actions(ui, bottom_rect, can_undo, can_redo, &mut actions);
        });

    actions
}

// ---------------------------------------------------------------------------
// Toolbar sections
// ---------------------------------------------------------------------------

fn show_header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("AliDraw")
                .size(34.0)
                .color(Color32::from_rgb(21, 124, 236))
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

fn show_tool_buttons(ui: &mut egui::Ui, active_tool: &mut Tool) {
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

fn show_style_buttons(ui: &mut egui::Ui, active_style: &mut BrushStyle, active_tool: &Tool) {
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

fn show_palette_grid(ui: &mut egui::Ui, active_color_index: &mut usize, active_tool: &Tool) {
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

fn show_brush_size_control(
    ui: &mut egui::Ui,
    brush_size_index: &mut usize,
    active_tool: &Tool,
    active_style: &BrushStyle,
    active_color_index: usize,
) {
    let preview_color = palette::active_color(*active_tool, active_color_index);

    let preview_width = palette::active_width(*brush_size_index);
    let preview_radius = (preview_width * active_style.width_multiplier() * 0.45).clamp(4.0, 26.0);

    let remaining_for_slider =
        (ui.available_height() - 2.0).clamp(180.0, MAX_VERTICAL_SLIDER_HEIGHT);
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
        let step_count = max_index.max(1);
        (0..=step_count).for_each(|step| {
            let t = step as f32 / step_count as f32;
            let y = rail_rect.bottom() - t * rail_rect.height();
            let tick_half = if step % 2 == 0 { 10.0 } else { 7.0 };
            ui.painter().line_segment(
                [
                    egui::pos2(rail_rect.center().x - tick_half, y),
                    egui::pos2(rail_rect.center().x + tick_half, y),
                ],
                Stroke::new(1.0, Color32::from_rgb(140, 140, 152)),
            );
        });

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
                    .circle_filled(preview_rect.center(), preview_radius, preview_color);

                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!(
                        "{} px",
                        palette::active_width(*brush_size_index) as i32
                    ))
                    .size(36.0)
                    .strong()
                    .color(Color32::from_rgb(52, 59, 76)),
                );
            },
        );
    });
}

fn show_footer_actions(
    ui: &mut egui::Ui,
    bottom_rect: egui::Rect,
    can_undo: bool,
    can_redo: bool,
    actions: &mut ToolbarActions,
) {
    ui.scope_builder(
        UiBuilder::new()
            .id_salt("toolbar-bottom")
            .max_rect(bottom_rect),
        |ui| {
            ui.separator();
            ui.add_space(4.0);
            ui.horizontal_centered(|ui| {
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
        },
    );
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
