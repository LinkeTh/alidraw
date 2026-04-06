use std::collections::VecDeque;
use tiny_skia::{Color, IntRect, Pixmap, PixmapPaint, Transform};

#[derive(Default)]
pub(crate) struct Paper {
    image: Option<Pixmap>,
    undo: VecDeque<HistoryPatch>,
    redo: Vec<HistoryPatch>,
}

// Edits keep only the pixels they can change. These coordinates remain valid when paper grows.
struct HistoryPatch {
    rect: IntRect,
    pixels: Pixmap,
}
impl HistoryPatch {
    fn swap(&mut self, paper: &mut Pixmap) {
        let stride = paper.width() as usize * 4;
        let row_len = self.rect.width() as usize * 4;
        let start = self.rect.top() as usize * stride + self.rect.left() as usize * 4;
        for (row, saved) in self.pixels.data_mut().chunks_exact_mut(row_len).enumerate() {
            let offset = start + row * stride;
            saved.swap_with_slice(&mut paper.data_mut()[offset..offset + row_len]);
        }
    }
}

type EditResult = Result<Vec<u8>, String>;

impl Paper {
    pub(crate) fn image(&self) -> Result<&Pixmap, String> {
        self.image
            .as_ref()
            .ok_or_else(|| "Paper is not ready".into())
    }

    pub(crate) fn grow(&mut self, width: u32, height: u32) -> Result<(), String> {
        let Some(old) = &self.image else {
            self.image = Some(white(width, height)?);
            return Ok(());
        };
        let (width, height) = (width.max(old.width()), height.max(old.height()));
        if (width, height) != (old.width(), old.height()) {
            self.image = Some(expanded(old, width, height)?);
        }
        Ok(())
    }

    fn remember(&mut self, patch: HistoryPatch) {
        if self.undo.len() == 10 {
            self.undo.pop_front();
        }
        self.undo.push_back(patch);
        self.redo.clear();
    }

    pub(crate) fn commit(
        &mut self,
        mut points: Vec<[f32; 2]>,
        width: f32,
        rgba: [u8; 4],
    ) -> EditResult {
        if !(1.0..=120.0).contains(&width) {
            return Err("Invalid brush size".into());
        }
        let paper = self.image()?;
        points.dedup();
        let rect =
            stroke_bounds(&points, width, paper.width(), paper.height()).ok_or("Invalid stroke")?;
        let pixels = paper.clone_rect(rect).ok_or("Could not snapshot stroke")?;
        self.remember(HistoryPatch { rect, pixels });
        let paper = self.image.as_mut().ok_or("Paper is not ready")?;
        crate::ink::draw(paper, points.into_iter(), width, rgba);
        self.patch(Some(rect))
    }

    pub(crate) fn fill(&mut self, point: [f32; 2], color: [u8; 3]) -> EditResult {
        let paper = self.image.as_mut().ok_or("Paper is not ready")?;
        let before = paper.clone();
        if let Some(rect) = crate::ink::flood(paper, point, color) {
            // Keep the pre-fill image as an undo patch, including when later window growth exposes new paper.
            let full =
                IntRect::from_xywh(0, 0, before.width(), before.height()).ok_or("Invalid paper")?;
            self.remember(HistoryPatch {
                rect: full,
                pixels: before,
            });
            self.patch(Some(rect))
        } else {
            self.patch(None)
        }
    }

    pub(crate) fn history(&mut self, redo: bool) -> EditResult {
        let paper = self.image.as_mut().ok_or("Paper is not ready")?;
        let entry = if redo {
            self.redo.pop()
        } else {
            self.undo.pop_back()
        };
        let Some(mut entry) = entry else {
            return self.patch(None);
        };
        entry.swap(paper);
        let rect = entry.rect;
        if redo {
            self.undo.push_back(entry)
        } else {
            self.redo.push(entry)
        }
        self.patch(Some(rect))
    }

    pub(crate) fn replace(
        &mut self,
        width: u32,
        height: u32,
        imported: Option<&Pixmap>,
    ) -> EditResult {
        let (width, height) = imported.map_or((width, height), |img| {
            (width.max(img.width()), height.max(img.height()))
        });
        let mut next = white(width, height)?;
        if let Some(img) = imported {
            next.draw_pixmap(
                0,
                0,
                img.as_ref(),
                &PixmapPaint::default(),
                Transform::identity(),
                None,
            );
        }
        self.image = Some(next);
        self.undo.clear();
        self.redo.clear();
        self.full()
    }

    pub(crate) fn needs_confirmation(&self) -> bool {
        !self.undo.is_empty()
            || self
                .image
                .as_ref()
                .is_some_and(|img| img.data().iter().any(|&b| b != 255))
    }

    fn full(&self) -> EditResult {
        let paper = self.image()?;
        self.patch(IntRect::from_xywh(0, 0, paper.width(), paper.height()))
    }

    fn patch(&self, rect: Option<IntRect>) -> EditResult {
        let paper = self.image()?;
        let (x, y, w, h) = rect
            .map(|r| (r.x() as u32, r.y() as u32, r.width(), r.height()))
            .unwrap_or_default();
        let flags = u32::from(!self.undo.is_empty()) | (u32::from(!self.redo.is_empty()) << 1);
        let mut bytes = Vec::with_capacity(28 + w as usize * h as usize * 4);
        for n in [paper.width(), paper.height(), x, y, w, h, flags] {
            bytes.extend(n.to_le_bytes());
        }
        for row in y..y + h {
            let start = (row as usize * paper.width() as usize + x as usize) * 4;
            bytes.extend_from_slice(&paper.data()[start..start + w as usize * 4]);
        }
        Ok(bytes)
    }
}

fn white(w: u32, h: u32) -> Result<Pixmap, String> {
    let mut image = Pixmap::new(w, h).ok_or("Could not allocate paper")?;
    image.fill(Color::WHITE);
    Ok(image)
}
fn expanded(old: &Pixmap, w: u32, h: u32) -> Result<Pixmap, String> {
    let mut next = white(w, h)?;
    for (from, to) in old
        .data()
        .chunks_exact(old.width() as usize * 4)
        .zip(next.data_mut().chunks_exact_mut(w as usize * 4))
    {
        to[..from.len()].copy_from_slice(from);
    }
    Ok(next)
}

fn stroke_bounds(
    points: &[[f32; 2]],
    brush_width: f32,
    width: u32,
    height: u32,
) -> Option<IntRect> {
    let [mut left, mut top] = *points.first()?;
    let (mut right, mut bottom) = (left, top);
    for &[x, y] in points {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        left = left.min(x);
        right = right.max(x);
        top = top.min(y);
        bottom = bottom.max(y);
    }
    let pad = brush_width / 2.0 + 1.0;
    IntRect::from_ltrb(
        (left - pad).floor().max(0.0) as i32,
        (top - pad).floor().max(0.0) as i32,
        (right + pad).ceil().min(width as f32) as i32,
        (bottom + pad).ceil().min(height as f32) as i32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_brush_strokes_match_reference_pixels() {
        // FNV-1a digests of the reference dot and polyline from commit 19d7884.
        let cases = [
            (8.0, [255, 59, 48, 255], 0x8e95a71074a1ef48),
            (8.0 * 1.3, [255, 59, 48, 170], 0x7f8ff1abb0700cb9),
            (8.0 * 1.8, [255, 59, 48, 105], 0xa9625edcf9b138fb),
            (8.0, [255, 255, 255, 255], 0xfb262bbca48cd98b),
        ];
        for (width, color, expected) in cases {
            let mut paper = Paper::default();
            paper.grow(80, 60).expect("initial paper");
            paper.fill([0.0, 0.0], [100, 120, 140]).expect("fill");
            for points in [
                vec![[14.5, 10.25]],
                vec![[0.0, 3.0], [22.25, 20.75], [32.0, 3.5], [79.0, 59.0]],
            ] {
                paper.commit(points, width, color).expect("stroke");
            }
            let hash = paper
                .image()
                .expect("paper")
                .data()
                .iter()
                .fold(0xcbf29ce484222325u64, |h, b| {
                    (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
                });
            assert_eq!(hash, expected);
        }
    }

    #[test]
    fn small_patch_undo_and_redo_survive_growth_and_shrink() {
        let mut paper = Paper::default();
        paper.grow(50, 60).expect("initial paper");
        let patch = paper
            .commit(vec![[45.0, 54.0]], 8.0, [1, 2, 3, 255])
            .expect("dot");
        assert_eq!(patch.len(), 28 + 10 * 10 * 4);
        paper.grow(80, 90).expect("grow");
        paper.grow(10, 10).expect("hide edges");
        paper.history(false).expect("undo");
        let img = paper.image().expect("paper");
        assert_eq!((img.width(), img.height()), (80, 90));
        assert!(img.data().iter().all(|&b| b == 255));
        paper.history(true).expect("redo");
        assert_eq!(
            paper
                .image()
                .expect("paper")
                .pixel(45, 54)
                .expect("pixel")
                .red(),
            1
        );
        paper.history(false).expect("undo");
        paper.fill([75.0, 85.0], [10, 30, 50]).expect("new edit");
        assert!(paper.redo.is_empty());
    }

    #[test]
    fn ten_edits_and_noop_fill() {
        let mut paper = Paper::default();
        paper.grow(5, 4).expect("initial paper");
        for r in 0..11 {
            paper.fill([0.0, 0.0], [r, 0, 0]).expect("fill");
        }
        for _ in 0..10 {
            paper.history(false).expect("undo");
        }
        assert!(paper.undo.is_empty());
        let img = paper.image().expect("paper");
        assert_eq!(img.pixel(0, 0).expect("first fill").red(), 0);
        paper.fill([0.0, 0.0], [0, 0, 0]).expect("noop");
        assert_eq!(paper.redo.len(), 10);
    }

    #[test]
    fn open_keeps_hidden_edges_and_composites_transparency_over_white() {
        let mut paper = Paper::default();
        let mut imported = Pixmap::new(12, 2).expect("image");
        imported.fill(Color::from_rgba8(255, 0, 0, 128));
        paper.replace(8, 8, Some(&imported)).expect("open");
        let img = paper.image().expect("paper");
        assert_eq!((img.width(), img.height()), (12, 8));
        assert_eq!(img.pixel(11, 0).expect("hidden edge").green(), 127);
        assert_eq!(img.pixel(0, 2).expect("white paper").green(), 255);
        paper
            .fill([0.0, 0.0], [0, 0, 255])
            .expect("fill imported pixels");
        assert_eq!(
            paper
                .image()
                .expect("paper")
                .pixel(11, 1)
                .expect("filled edge")
                .red(),
            0
        );
        paper.replace(6, 5, None).expect("new");
        assert!(paper.undo.is_empty() && paper.redo.is_empty());
        assert!(!paper.needs_confirmation());
    }
}
