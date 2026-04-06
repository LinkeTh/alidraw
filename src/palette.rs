use eframe::egui::Color32;

use crate::brush::BrushStyle;

pub(crate) const SWATCH_SIZE: f32 = 48.0;
pub(crate) const DEFAULT_COLOR_INDEX: usize = 20;
pub(crate) const DEFAULT_BRUSH_SIZE_INDEX: usize = 9;
pub(crate) const DEFAULT_BRUSH_STYLE_INDEX: usize = 0;

pub(crate) const BRUSH_SIZES: [f32; 20] = [
    2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 32.0, 36.0, 40.0,
    48.0, 56.0, 64.0,
];
pub(crate) const BRUSH_STYLES: [BrushStyle; 3] = [
    BrushStyle::Pencil,
    BrushStyle::Marker,
    BrushStyle::Watercolor,
];

pub(crate) const COLORS: [Color32; 24] = [
    Color32::from_rgb(0xFF, 0x3B, 0x30),
    Color32::from_rgb(0xFF, 0x6A, 0x2A),
    Color32::from_rgb(0xFF, 0x95, 0x00),
    Color32::from_rgb(0xFF, 0xB0, 0x20),
    Color32::from_rgb(0xFF, 0xCC, 0x00),
    Color32::from_rgb(0xE5, 0xE5, 0x00),
    Color32::from_rgb(0x9B, 0xD4, 0x00),
    Color32::from_rgb(0x34, 0xC7, 0x59),
    Color32::from_rgb(0x00, 0xC7, 0xBE),
    Color32::from_rgb(0x00, 0xA7, 0xA0),
    Color32::from_rgb(0x32, 0xAD, 0xE6),
    Color32::from_rgb(0x00, 0x7A, 0xFF),
    Color32::from_rgb(0x3B, 0x4B, 0xFF),
    Color32::from_rgb(0x6C, 0x5C, 0xE7),
    Color32::from_rgb(0xAF, 0x52, 0xDE),
    Color32::from_rgb(0xE0, 0x56, 0xFD),
    Color32::from_rgb(0xFF, 0x2D, 0x55),
    Color32::from_rgb(0xFF, 0x5E, 0x9A),
    Color32::from_rgb(0xA2, 0x84, 0x5E),
    Color32::from_rgb(0x7F, 0x5A, 0x3A),
    Color32::from_rgb(0x1C, 0x1C, 0x1E),
    Color32::from_rgb(0x4E, 0x4E, 0x52),
    Color32::from_rgb(0xCF, 0xCF, 0xD4),
    Color32::from_rgb(0xFF, 0xFF, 0xFF),
];
