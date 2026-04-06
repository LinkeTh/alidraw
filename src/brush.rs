use eframe::egui::Color32;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Brush,
    Eraser,
}

impl Tool {
    pub fn is_eraser(self) -> bool {
        matches!(self, Self::Eraser)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeKind {
    Draw,
    Erase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushStyle {
    Pencil,
    Marker,
    Watercolor,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrushSpec {
    pub color: [u8; 4],
    pub width: f32,
    pub style: BrushStyle,
    pub kind: StrokeKind,
}

impl BrushSpec {
    pub fn effective_color(self) -> Color32 {
        let color = Color32::from_rgba_premultiplied(
            self.color[0],
            self.color[1],
            self.color[2],
            self.color[3],
        );

        match self.kind {
            StrokeKind::Draw => color,
            StrokeKind::Erase => Color32::WHITE,
        }
    }
}
