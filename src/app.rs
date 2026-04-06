use eframe::egui::{self, Color32, Sense, UiBuilder};
use std::time::{Duration, Instant};

use crate::brush::{BrushStyle, Tool};
use crate::canvas::StrokeData;
use crate::canvas_raster::{self, CanvasRaster};
use crate::export;
use crate::history::History;
use crate::palette::{
    self, DEFAULT_BRUSH_SIZE_INDEX, DEFAULT_BRUSH_STYLE_INDEX, DEFAULT_COLOR_INDEX,
};
use crate::toolbar;

const STATUS_MESSAGE_DURATION: Duration = Duration::from_secs(2);
const TOOLBAR_SIDE_PADDING: f32 = 16.0;

// -- Status overlay geometry --
const STATUS_OVERLAY_WIDTH: f32 = 520.0;
const STATUS_OVERLAY_HEIGHT: f32 = 56.0;
const STATUS_OVERLAY_TOP_OFFSET: f32 = 52.0;

// -- Confirmation dialog geometry --
const DIALOG_WIDTH: f32 = 540.0;
const DIALOG_HEIGHT: f32 = 230.0;
const DIALOG_HEADING_SIZE: f32 = 32.0;
const DIALOG_SUBTITLE_SIZE: f32 = 22.0;
const DIALOG_BUTTON_TEXT_SIZE: f32 = 24.0;
const DIALOG_CANCEL_BUTTON_WIDTH: f32 = 170.0;
const DIALOG_CONFIRM_BUTTON_WIDTH: f32 = 190.0;
const DIALOG_BUTTON_HEIGHT: f32 = 62.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum PendingDialog {
    #[default]
    None,
    ConfirmClose,
    ConfirmNewDrawing,
}

pub(crate) struct AlidrawApp {
    strokes: Vec<StrokeData>,
    current_stroke: Option<StrokeData>,
    active_tool: Tool,
    active_style: BrushStyle,
    active_color_index: usize,
    brush_size_index: usize,
    history: History,
    raster: CanvasRaster,
    status_message: Option<String>,
    status_until: Option<Instant>,
    allow_native_close: bool,
    pending_dialog: PendingDialog,
}

impl Default for AlidrawApp {
    fn default() -> Self {
        Self {
            strokes: Vec::new(),
            current_stroke: None,
            active_tool: Tool::Brush,
            active_style: palette::active_style(DEFAULT_BRUSH_STYLE_INDEX),
            active_color_index: DEFAULT_COLOR_INDEX,
            brush_size_index: DEFAULT_BRUSH_SIZE_INDEX,
            history: History::default(),
            raster: CanvasRaster::default(),
            status_message: None,
            status_until: None,
            allow_native_close: false,
            pending_dialog: PendingDialog::None,
        }
    }
}

impl AlidrawApp {
    fn begin_stroke_if_needed(&mut self) {
        if self.current_stroke.is_none() {
            self.current_stroke = Some(StrokeData::new(
                palette::active_color(self.active_tool, self.active_color_index),
                palette::active_width(self.brush_size_index),
                self.active_tool,
                self.active_style,
            ));
        }
    }

    fn append_point_to_stroke(&mut self, point: egui::Pos2) {
        self.begin_stroke_if_needed();
        if let Some(stroke) = self.current_stroke.as_mut() {
            stroke.push_point(point);
        }
    }

    fn finish_current_stroke(&mut self) {
        if let Some(stroke) = self.current_stroke.take()
            && stroke.point_count() > 0
        {
            self.history.snapshot(&self.strokes);
            self.strokes.push(stroke);
            self.raster.mark_dirty();
        }
    }

    fn undo(&mut self) {
        self.finish_current_stroke();
        self.history.undo(&mut self.strokes);
        self.raster.mark_dirty();
        self.set_status("Undid one step");
    }

    fn redo(&mut self) {
        self.finish_current_stroke();
        self.history.redo(&mut self.strokes);
        self.raster.mark_dirty();
        self.set_status("Redid one step");
    }

    fn save_as(&mut self, canvas_rect: egui::Rect) {
        self.finish_current_stroke();

        let Some((path, format)) = export::choose_export_path() else {
            return;
        };

        let width = canvas_raster::safe_dimension(canvas_rect.width());
        let height = canvas_raster::safe_dimension(canvas_rect.height());

        let Some(rgba) = CanvasRaster::rasterize_rgba(
            width,
            height,
            canvas_rect.left_top(),
            &self.strokes,
            None,
        ) else {
            self.set_status("Could not prepare image");
            return;
        };

        match export::save_rgba_image(width, height, &rgba, &path, format) {
            Ok(saved_path) => {
                let label = saved_path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| "drawing".to_owned());
                self.set_status(&format!("Saved {label}"));
            }
            Err(error) => {
                eprintln!("Save failed: {error}");
                self.set_status("Could not save drawing");
            }
        }
    }

    fn set_status(&mut self, message: &str) {
        self.status_message = Some(message.to_owned());
        self.status_until = Some(Instant::now() + STATUS_MESSAGE_DURATION);
    }

    fn prune_status(&mut self) {
        if let Some(until) = self.status_until
            && Instant::now() > until
        {
            self.status_until = None;
            self.status_message = None;
        }
    }

    fn paint_status_overlay(&self, ui: &mut egui::Ui, canvas_rect: egui::Rect) {
        if let Some(message) = self.status_message.as_deref() {
            let rect = egui::Rect::from_center_size(
                egui::pos2(
                    canvas_rect.center().x,
                    canvas_rect.top() + STATUS_OVERLAY_TOP_OFFSET,
                ),
                egui::vec2(STATUS_OVERLAY_WIDTH, STATUS_OVERLAY_HEIGHT),
            );

            ui.painter().rect_filled(
                rect,
                12.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 230),
            );
            ui.painter().rect_stroke(
                rect,
                12.0,
                egui::Stroke::new(2.0, Color32::from_rgb(255, 105, 180)),
                egui::StrokeKind::Outside,
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                message,
                egui::FontId::proportional(24.0),
                Color32::from_rgb(55, 55, 80),
            );
        }
    }

    fn clear_drawing(&mut self) {
        self.finish_current_stroke();
        if self.strokes.is_empty() {
            return;
        }

        self.history.set_baseline(&[]);
        self.strokes.clear();
        self.current_stroke = None;
        self.raster.mark_dirty();
        self.set_status("Started a fresh drawing");
    }

    fn show_close_confirmation(&mut self, ui: &mut egui::Ui) {
        if self.pending_dialog != PendingDialog::ConfirmClose {
            return;
        }

        let response = show_confirm_dialog(
            ui,
            &ConfirmDialogConfig {
                window_title: "Leave AliDraw?",
                heading: "Quit AliDraw?",
                subtitle: "Your drawing will be discarded",
                cancel_label: "Stay",
                cancel_fill: Color32::from_rgb(188, 230, 255),
                confirm_label: "Quit",
                confirm_fill: Color32::from_rgb(255, 203, 203),
            },
        );

        if response.cancel_clicked {
            self.allow_native_close = false;
            self.pending_dialog = PendingDialog::None;
        }
        if response.confirm_clicked {
            self.finish_current_stroke();
            self.allow_native_close = true;
            self.pending_dialog = PendingDialog::None;
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn show_new_drawing_confirmation(&mut self, ui: &mut egui::Ui) {
        if self.pending_dialog != PendingDialog::ConfirmNewDrawing {
            return;
        }

        let response = show_confirm_dialog(
            ui,
            &ConfirmDialogConfig {
                window_title: "Start new drawing?",
                heading: "Start a new drawing?",
                subtitle: "This clears the current canvas",
                cancel_label: "Keep",
                cancel_fill: Color32::from_rgb(186, 230, 255),
                confirm_label: "Start New",
                confirm_fill: Color32::from_rgb(255, 236, 186),
            },
        );

        if response.cancel_clicked {
            self.pending_dialog = PendingDialog::None;
        }
        if response.confirm_clicked {
            self.pending_dialog = PendingDialog::None;
            self.clear_drawing();
        }
    }
}

/// Buttons clicked on a confirmation dialog.
struct ConfirmResponse {
    cancel_clicked: bool,
    confirm_clicked: bool,
}

/// Configuration for a confirmation dialog.
struct ConfirmDialogConfig<'a> {
    window_title: &'a str,
    heading: &'a str,
    subtitle: &'a str,
    cancel_label: &'a str,
    cancel_fill: Color32,
    confirm_label: &'a str,
    confirm_fill: Color32,
}

/// Shared helper for the two confirmation modals.
fn show_confirm_dialog(ui: &mut egui::Ui, config: &ConfirmDialogConfig<'_>) -> ConfirmResponse {
    let mut response = ConfirmResponse {
        cancel_clicked: false,
        confirm_clicked: false,
    };

    egui::Window::new(config.window_title)
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(DIALOG_WIDTH, DIALOG_HEIGHT))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(config.heading)
                        .size(DIALOG_HEADING_SIZE)
                        .color(Color32::from_rgb(60, 60, 90)),
                );
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(config.subtitle)
                        .size(DIALOG_SUBTITLE_SIZE)
                        .color(Color32::from_rgb(86, 86, 105)),
                );
                ui.add_space(18.0);

                ui.horizontal(|ui| {
                    let cancel = egui::Button::new(
                        egui::RichText::new(config.cancel_label).size(DIALOG_BUTTON_TEXT_SIZE),
                    )
                    .min_size(egui::vec2(DIALOG_CANCEL_BUTTON_WIDTH, DIALOG_BUTTON_HEIGHT))
                    .fill(config.cancel_fill);
                    if ui.add(cancel).clicked() {
                        response.cancel_clicked = true;
                    }

                    ui.add_space(20.0);

                    let confirm = egui::Button::new(
                        egui::RichText::new(config.confirm_label).size(DIALOG_BUTTON_TEXT_SIZE),
                    )
                    .min_size(egui::vec2(
                        DIALOG_CONFIRM_BUTTON_WIDTH,
                        DIALOG_BUTTON_HEIGHT,
                    ))
                    .fill(config.confirm_fill);
                    if ui.add(confirm).clicked() {
                        response.confirm_clicked = true;
                    }
                });
            });
        });

    response
}

impl eframe::App for AlidrawApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.ctx().input(|input| input.viewport().close_requested()) && !self.allow_native_close {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending_dialog = PendingDialog::ConfirmClose;
        }

        let full_rect = ui.available_rect_before_wrap();
        let toolbar_width = toolbar::toolbar_width();
        let toolbar_right = (full_rect.right() - TOOLBAR_SIDE_PADDING).max(full_rect.left());
        let toolbar_left = (toolbar_right - toolbar_width).max(full_rect.left());
        let canvas_rect =
            egui::Rect::from_min_max(full_rect.min, egui::pos2(toolbar_left, full_rect.bottom()));
        let toolbar_rect = egui::Rect::from_min_max(
            egui::pos2(toolbar_left, full_rect.top()),
            egui::pos2(toolbar_right, full_rect.bottom()),
        );

        let toolbar_actions = ui
            .scope_builder(
                UiBuilder::new()
                    .id_salt("toolbar-region")
                    .max_rect(toolbar_rect),
                |ui| {
                    toolbar::show_toolbar(
                        ui,
                        &mut self.active_tool,
                        &mut self.active_style,
                        &mut self.active_color_index,
                        &mut self.brush_size_index,
                        self.history.can_undo(),
                        self.history.can_redo(),
                    )
                },
            )
            .inner;

        if toolbar_actions.undo {
            self.undo();
        }
        if toolbar_actions.redo {
            self.redo();
        }

        self.prune_status();

        let interaction_blocked = self.pending_dialog != PendingDialog::None;
        ui.scope_builder(
            UiBuilder::new()
                .id_salt("canvas-region")
                .max_rect(canvas_rect),
            |ui| {
                let canvas_area = ui.max_rect();
                let response = ui.allocate_rect(canvas_area, Sense::click_and_drag());
                let painter = ui.painter_at(canvas_area);

                painter.rect_filled(canvas_area, 0.0, Color32::WHITE);

                if !interaction_blocked {
                    if response.is_pointer_button_down_on() {
                        if let Some(point) = response.interact_pointer_pos() {
                            self.append_point_to_stroke(point);
                        }
                    } else {
                        self.finish_current_stroke();
                    }
                }

                self.raster
                    .render(ui, canvas_area, &self.strokes, self.current_stroke.as_ref());

                ui.painter().rect_stroke(
                    canvas_area.shrink(1.0),
                    6.0,
                    egui::Stroke::new(2.0, Color32::from_rgb(255, 149, 0)),
                    egui::StrokeKind::Outside,
                );

                self.paint_status_overlay(ui, canvas_area);
            },
        );

        if toolbar_actions.save_as {
            self.save_as(canvas_rect);
        }
        if toolbar_actions.new_drawing {
            self.pending_dialog = PendingDialog::ConfirmNewDrawing;
        }
        if toolbar_actions.quit {
            self.allow_native_close = false;
            self.pending_dialog = PendingDialog::ConfirmClose;
        }

        self.show_close_confirmation(ui);
        self.show_new_drawing_confirmation(ui);

        ui.expand_to_include_rect(full_rect);
    }
}
