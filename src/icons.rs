use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId, RichText};
use lucide_icons::Icon;

const LUCIDE_FONT_NAME: &str = "lucide";

#[derive(Debug, Clone, Copy)]
pub enum ToolbarIcon {
    Brush,
    Eraser,
    Pencil,
    Marker,
    Watercolor,
    Undo,
    Redo,
    Save,
    New,
    Quit,
}

pub fn initialize(ctx: &egui::Context) {
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

pub fn icon_text(icon: ToolbarIcon, size: f32, color: egui::Color32) -> RichText {
    let glyph = char::from(lucide_icon(icon)).to_string();
    RichText::new(glyph)
        .font(FontId::new(size, FontFamily::Name(LUCIDE_FONT_NAME.into())))
        .color(color)
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
        ToolbarIcon::New => Icon::FilePlus,
        ToolbarIcon::Quit => Icon::Power,
    }
}
