mod app;
mod brush;
mod canvas;
mod canvas_raster;
mod error;
mod export;
mod history;
mod icons;
mod import;
mod palette;
mod toolbar;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        #[cfg(target_os = "linux")]
        event_loop_builder: Some(Box::new(|builder| {
            use winit::platform::x11::EventLoopBuilderExtX11;
            builder.with_x11();
        })),
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Alidraw")
            .with_inner_size([1366.0, 900.0])
            .with_decorations(true),
        ..Default::default()
    };

    eframe::run_native(
        "Alidraw",
        options,
        Box::new(|cc| {
            icons::initialize(&cc.egui_ctx);
            Ok(Box::new(app::AlidrawApp::default()))
        }),
    )
}
