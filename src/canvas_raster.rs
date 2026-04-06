use eframe::egui::{self, Rect, TextureHandle, TextureOptions};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::brush::{BrushSpec, BrushStyle, StrokeKind};
use crate::canvas::StrokeData;

#[derive(Default)]
pub struct CanvasRaster {
    committed_pixmap: Option<Pixmap>,
    frame_pixmap: Option<Pixmap>,
    texture: Option<TextureHandle>,
    width: u32,
    height: u32,
    dirty: bool,
}

impl CanvasRaster {
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        rect: Rect,
        committed_strokes: &[StrokeData],
        current_stroke: Option<&StrokeData>,
    ) {
        let width = rect.width().round().max(1.0) as u32;
        let height = rect.height().round().max(1.0) as u32;

        if self.width != width
            || self.height != height
            || self.committed_pixmap.is_none()
            || self.frame_pixmap.is_none()
        {
            self.width = width;
            self.height = height;
            self.committed_pixmap = Pixmap::new(width, height);
            self.frame_pixmap = Pixmap::new(width, height);
            self.texture = None;
            self.dirty = true;
        }

        let committed_changed = self.dirty;
        if committed_changed {
            self.redraw_committed(committed_strokes, rect.left_top());
        }

        let needs_frame_update = committed_changed || current_stroke.is_some();

        let Some(frame_pixmap) = self.frame_pixmap.as_mut() else {
            return;
        };

        if needs_frame_update || self.texture.is_none() {
            if let Some(committed) = self.committed_pixmap.as_ref() {
                frame_pixmap.data_mut().copy_from_slice(committed.data());
            }

            if let Some(stroke) = current_stroke {
                stroke.apply_tiny_skia(frame_pixmap, rect.left_top());
            }

            let size = [width as usize, height as usize];
            let image = egui::ColorImage::from_rgba_premultiplied(size, frame_pixmap.data());

            if let Some(texture) = self.texture.as_mut() {
                texture.set(image, TextureOptions::LINEAR);
            } else {
                self.texture = Some(ui.ctx().load_texture(
                    "canvas-raster",
                    image,
                    TextureOptions::LINEAR,
                ));
            }
        }

        if let Some(texture) = self.texture.as_ref() {
            ui.painter().image(
                texture.id(),
                rect,
                Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    }

    fn redraw_committed(&mut self, committed_strokes: &[StrokeData], origin: egui::Pos2) {
        let Some(committed) = self.committed_pixmap.as_mut() else {
            return;
        };

        committed.fill(Color::WHITE);
        committed_strokes
            .iter()
            .for_each(|stroke| stroke.apply_tiny_skia(committed, origin));
        self.dirty = false;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn rasterize_rgba(
        width: u32,
        height: u32,
        canvas_origin: egui::Pos2,
        committed_strokes: &[StrokeData],
        current_stroke: Option<&StrokeData>,
    ) -> Option<Vec<u8>> {
        let mut pixmap = Pixmap::new(width, height)?;
        pixmap.fill(Color::WHITE);

        committed_strokes
            .iter()
            .for_each(|stroke| stroke.apply_tiny_skia(&mut pixmap, canvas_origin));

        if let Some(stroke) = current_stroke {
            stroke.apply_tiny_skia(&mut pixmap, canvas_origin);
        }

        Some(unpremultiply_rgba(pixmap.data()))
    }
}

impl StrokeData {
    fn apply_tiny_skia(&self, pixmap: &mut Pixmap, canvas_origin: egui::Pos2) {
        if self.point_count() == 0 {
            return;
        }

        let spec = self.brush_spec();
        let points: Vec<egui::Pos2> = self.iter_points().collect();

        if points.len() == 1 {
            self.paint_dot(pixmap, canvas_origin, spec);
            return;
        }

        let mut path_builder = PathBuilder::new();
        let first = points[0] - canvas_origin.to_vec2();
        path_builder.move_to(first.x, first.y);
        points.iter().skip(1).for_each(|point| {
            let translated = *point - canvas_origin.to_vec2();
            path_builder.line_to(translated.x, translated.y);
        });

        let Some(path) = path_builder.finish() else {
            return;
        };

        let mut paint = Paint::default();
        let effective = spec.effective_color();
        let rgba = color_for_style(spec.style, spec.kind, effective);
        paint.set_color_rgba8(rgba.0, rgba.1, rgba.2, rgba.3);
        paint.anti_alias = true;

        let stroke = Stroke {
            width: width_for_style(spec.style, spec.width),
            ..Stroke::default()
        };

        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    fn paint_dot(&self, pixmap: &mut Pixmap, canvas_origin: egui::Pos2, spec: BrushSpec) {
        let Some(point) = self.first_point() else {
            return;
        };

        let point = point - canvas_origin.to_vec2();
        let radius = width_for_style(spec.style, spec.width) * 0.5;
        let Some(path) = PathBuilder::from_circle(point.x, point.y, radius) else {
            return;
        };

        let mut paint = Paint::default();
        let effective = spec.effective_color();
        let rgba = color_for_style(spec.style, spec.kind, effective);
        paint.set_color_rgba8(rgba.0, rgba.1, rgba.2, rgba.3);
        paint.anti_alias = true;

        pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

fn color_for_style(style: BrushStyle, kind: StrokeKind, color: egui::Color32) -> (u8, u8, u8, u8) {
    if matches!(kind, StrokeKind::Erase) {
        return (255, 255, 255, 255);
    }

    match style {
        BrushStyle::Pencil => (color.r(), color.g(), color.b(), 255),
        BrushStyle::Marker => (color.r(), color.g(), color.b(), 170),
        BrushStyle::Watercolor => (color.r(), color.g(), color.b(), 105),
    }
}

fn width_for_style(style: BrushStyle, width: f32) -> f32 {
    match style {
        BrushStyle::Pencil => width,
        BrushStyle::Marker => width * 1.3,
        BrushStyle::Watercolor => width * 1.8,
    }
}

fn unpremultiply_rgba(bytes: &[u8]) -> Vec<u8> {
    bytes
        .chunks_exact(4)
        .flat_map(|pixel| {
            let r = pixel[0] as u32;
            let g = pixel[1] as u32;
            let b = pixel[2] as u32;
            let a = pixel[3] as u32;

            if a == 0 {
                [0_u8, 0, 0, 0]
            } else if a == 255 {
                [pixel[0], pixel[1], pixel[2], pixel[3]]
            } else {
                let unpremul = |component: u32| ((component * 255 + a / 2) / a).min(255) as u8;
                [unpremul(r), unpremul(g), unpremul(b), pixel[3]]
            }
        })
        .collect()
}
