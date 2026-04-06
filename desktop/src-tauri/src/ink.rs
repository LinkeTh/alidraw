//! Rasterize brush strokes and flood fills on the drawing bitmap.
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub(crate) fn draw(
    paper: &mut Pixmap,
    mut points: impl Iterator<Item = [f32; 2]>,
    width: f32,
    rgba: [u8; 4],
) {
    let Some([x, y]) = points.next() else { return };
    let mut paint = Paint::default();
    paint.set_color_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
    paint.anti_alias = true;

    let Some(second) = points.next() else {
        if let Some(dot) = PathBuilder::from_circle(x, y, width * 0.5) {
            paper.fill_path(&dot, &paint, FillRule::Winding, Transform::identity(), None);
        }
        return;
    };
    let mut path = PathBuilder::new();
    path.move_to(x, y);
    path.line_to(second[0], second[1]);
    for [x, y] in points {
        path.line_to(x, y);
    }
    if let Some(path) = path.finish() {
        let stroke = Stroke {
            width,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };
        paper.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

/// Replace the exact four-connected region and return its affected rectangle.
pub(crate) fn flood(
    paper: &mut Pixmap,
    [x, y]: [f32; 2],
    rgb: [u8; 3],
) -> Option<tiny_skia::IntRect> {
    use std::collections::VecDeque;
    let (w, h) = (paper.width(), paper.height());
    if !(0.0..w as f32).contains(&x) || !(0.0..h as f32).contains(&y) {
        return None;
    }
    let (x, y) = (x as u32, y as u32);
    let target = paper.pixel(x, y)?;
    let replacement = tiny_skia::PremultipliedColorU8::from_rgba(rgb[0], rgb[1], rgb[2], 255)?;
    if target == replacement {
        return None;
    }
    let pixels = paper.pixels_mut();
    let index = |x: u32, y: u32| y as usize * w as usize + x as usize;
    pixels[index(x, y)] = replacement;
    let mut frontier = VecDeque::from([(x, y)]);
    let (mut left, mut top, mut right, mut bottom) = (x, y, x, y);
    while let Some((x, y)) = frontier.pop_front() {
        left = left.min(x);
        top = top.min(y);
        right = right.max(x);
        bottom = bottom.max(y);
        for (nx, ny) in [
            (x.saturating_sub(1), y),
            ((x + 1).min(w - 1), y),
            (x, y.saturating_sub(1)),
            (x, (y + 1).min(h - 1)),
        ] {
            let pixel = &mut pixels[index(nx, ny)];
            if *pixel == target {
                *pixel = replacement;
                frontier.push_back((nx, ny));
            }
        }
    }
    tiny_skia::IntRect::from_xywh(left as i32, top as i32, right - left + 1, bottom - top + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::{Color, IntRect};

    #[test]
    fn fill_stops_at_a_one_shade_boundary() {
        let mut paper = Pixmap::new(4, 3).expect("test paper");
        paper.fill(Color::WHITE);
        let boundary = [254, 255, 255, 255];
        for row in paper.data_mut().chunks_exact_mut(4 * 4) {
            row[4..8].copy_from_slice(&boundary);
        }

        assert_eq!(
            flood(&mut paper, [0.0, 0.0], [255, 0, 0]),
            IntRect::from_xywh(0, 0, 1, 3)
        );
        for row in paper.data().chunks_exact(4 * 4) {
            assert_eq!(&row[..4], &[255, 0, 0, 255]);
            assert_eq!(&row[4..8], &boundary);
            assert_eq!(&row[8..], &[255; 8]);
        }
    }
}
