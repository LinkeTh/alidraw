use eframe::egui::Color32;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Tool {
    #[default]
    Brush,
    Eraser,
}

impl Tool {
    pub(crate) fn is_eraser(self) -> bool {
        matches!(self, Self::Eraser)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StrokeKind {
    Draw,
    Erase,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum BrushStyle {
    #[default]
    Pencil,
    Marker,
    Watercolor,
}

impl BrushStyle {
    /// Width multiplier applied on top of the base brush size.
    pub(crate) fn width_multiplier(self) -> f32 {
        match self {
            Self::Pencil => 1.0,
            Self::Marker => 1.3,
            Self::Watercolor => 1.8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct BrushSpec {
    pub(crate) color: [u8; 4],
    pub(crate) width: f32,
    pub(crate) style: BrushStyle,
    pub(crate) kind: StrokeKind,
}

impl BrushSpec {
    /// Return the stored color as an `egui::Color32`.
    ///
    /// Erase-to-white mapping is handled downstream in the rasterizer
    /// (`color_for_style`), not here — this returns the raw colour
    /// so callers outside the raster pipeline get a faithful value.
    pub(crate) fn stored_color(self) -> Color32 {
        Color32::from_rgba_premultiplied(self.color[0], self.color[1], self.color[2], self.color[3])
    }
}
