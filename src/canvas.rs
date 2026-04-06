use eframe::egui::{Color32, Pos2};

use crate::brush::{BrushSpec, BrushStyle, StrokeKind, Tool};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StrokeData {
    points: Vec<[f32; 2]>,
    brush: BrushSpec,
}

impl StrokeData {
    pub(crate) fn new(color: Color32, width: f32, tool: Tool, style: BrushStyle) -> Self {
        let kind = if tool.is_eraser() {
            StrokeKind::Erase
        } else {
            StrokeKind::Draw
        };

        let effective_style = if tool.is_eraser() {
            BrushStyle::Pencil
        } else {
            style
        };

        Self {
            points: Vec::new(),
            brush: BrushSpec {
                color: [color.r(), color.g(), color.b(), color.a()],
                width,
                kind,
                style: effective_style,
            },
        }
    }

    pub(crate) fn push_point(&mut self, point: Pos2) {
        if self
            .points
            .last()
            .is_some_and(|last| last[0] == point.x && last[1] == point.y)
        {
            return;
        }
        self.points.push([point.x, point.y]);
    }

    pub(crate) fn point_count(&self) -> usize {
        self.points.len()
    }

    pub(crate) fn brush_spec(&self) -> BrushSpec {
        self.brush
    }

    pub(crate) fn iter_points(&self) -> impl Iterator<Item = Pos2> + '_ {
        self.points
            .iter()
            .map(|point| Pos2::new(point[0], point[1]))
    }

    pub(crate) fn first_point(&self) -> Option<Pos2> {
        self.points
            .first()
            .map(|point| Pos2::new(point[0], point[1]))
    }
}
