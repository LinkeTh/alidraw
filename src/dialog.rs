use eframe::egui::{self, Color32};
use std::path::PathBuf;

// -- Confirmation dialog geometry --
const DIALOG_WIDTH: f32 = 540.0;
const DIALOG_HEIGHT: f32 = 230.0;
const DIALOG_HEADING_SIZE: f32 = 32.0;
const DIALOG_SUBTITLE_SIZE: f32 = 22.0;
const DIALOG_BUTTON_TEXT_SIZE: f32 = 24.0;
const DIALOG_CANCEL_BUTTON_WIDTH: f32 = 170.0;
const DIALOG_CONFIRM_BUTTON_WIDTH: f32 = 190.0;
const DIALOG_BUTTON_HEIGHT: f32 = 62.0;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum PendingDialog {
    #[default]
    None,
    ConfirmClose,
    ConfirmNewDrawing,
    ConfirmOpen(PathBuf),
}

/// Buttons clicked on a confirmation dialog.
pub(crate) struct ConfirmResponse {
    pub(crate) cancel_clicked: bool,
    pub(crate) confirm_clicked: bool,
}

/// Configuration for a confirmation dialog.
pub(crate) struct ConfirmDialogConfig<'a> {
    pub(crate) window_title: &'a str,
    pub(crate) heading: &'a str,
    pub(crate) subtitle: &'a str,
    pub(crate) cancel_label: &'a str,
    pub(crate) cancel_fill: Color32,
    pub(crate) confirm_label: &'a str,
    pub(crate) confirm_fill: Color32,
}

/// Shared helper for confirmation modals.
pub(crate) fn show_confirm_dialog(
    ui: &mut egui::Ui,
    config: &ConfirmDialogConfig<'_>,
) -> ConfirmResponse {
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
                        .color(Color32::from_rgb(200, 210, 240)),
                );
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(config.subtitle)
                        .size(DIALOG_SUBTITLE_SIZE)
                        .color(Color32::from_rgb(170, 175, 195)),
                );
                ui.add_space(18.0);

                ui.horizontal(|ui| {
                    let cancel = egui::Button::new(
                        egui::RichText::new(config.cancel_label)
                            .size(DIALOG_BUTTON_TEXT_SIZE)
                            .color(Color32::from_rgb(20, 20, 24)),
                    )
                    .min_size(egui::vec2(DIALOG_CANCEL_BUTTON_WIDTH, DIALOG_BUTTON_HEIGHT))
                    .fill(config.cancel_fill);
                    if ui.add(cancel).clicked() {
                        response.cancel_clicked = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let confirm = egui::Button::new(
                            egui::RichText::new(config.confirm_label)
                                .size(DIALOG_BUTTON_TEXT_SIZE)
                                .color(Color32::from_rgb(20, 20, 24)),
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
        });

    response
}
