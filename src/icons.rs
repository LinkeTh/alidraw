use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use lucide_icons::Icon;

const LUCIDE_FONT_NAME: &str = "lucide";

#[derive(Debug, Clone, Copy)]
pub(crate) enum ToolbarIcon {
    Brush,
    Eraser,
    Pencil,
    Marker,
    Watercolor,
    Undo,
    Redo,
    Save,
    Open,
    New,
    Quit,
}

pub(crate) fn initialize(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        LUCIDE_FONT_NAME.to_owned(),
        std::sync::Arc::new(FontData::from_static(lucide_icons::LUCIDE_FONT_BYTES)),
    );

    fonts
        .families
        .entry(FontFamily::Name(LUCIDE_FONT_NAME.into()))
        .or_default()
        .insert(0, LUCIDE_FONT_NAME.to_owned());

    ctx.set_fonts(fonts);
}

/// Return just the glyph character for a toolbar icon, without styling.
pub(crate) fn icon_glyph(icon: ToolbarIcon) -> char {
    char::from(lucide_icon(icon))
}

fn lucide_icon(icon: ToolbarIcon) -> Icon {
    match icon {
        ToolbarIcon::Brush => Icon::Paintbrush,
        ToolbarIcon::Eraser => Icon::Eraser,
        ToolbarIcon::Pencil => Icon::Pencil,
        ToolbarIcon::Marker => Icon::Highlighter,
        ToolbarIcon::Watercolor => Icon::Droplets,
        ToolbarIcon::Undo => Icon::Undo2,
        ToolbarIcon::Redo => Icon::Redo2,
        ToolbarIcon::Save => Icon::Save,
        ToolbarIcon::Open => Icon::FolderOpen,
        ToolbarIcon::New => Icon::FilePlus,
        ToolbarIcon::Quit => Icon::Power,
    }
}
