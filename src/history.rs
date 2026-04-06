use std::collections::VecDeque;
use std::sync::Arc;

use crate::canvas::StrokeData;

const MAX_UNDO_STEPS: usize = 10;

#[derive(Debug, Clone, Default)]
pub(crate) struct History {
    undo: VecDeque<Vec<Arc<StrokeData>>>,
    redo: Vec<Vec<Arc<StrokeData>>>,
    baseline: Vec<Arc<StrokeData>>,
    modified_since_baseline: bool,
}

impl History {
    pub(crate) fn set_baseline(&mut self, strokes: &[Arc<StrokeData>]) {
        self.baseline = strokes.to_vec();
        self.undo.clear();
        self.redo.clear();
        self.modified_since_baseline = false;
    }

    pub(crate) fn snapshot(&mut self, strokes: &[Arc<StrokeData>]) {
        self.push_undo(strokes.to_vec());
        self.redo.clear();
        self.modified_since_baseline = true;
    }

    pub(crate) fn undo(&mut self, strokes: &mut Vec<Arc<StrokeData>>) {
        if let Some(previous) = self.undo.pop_back() {
            self.redo.push(strokes.clone());
            *strokes = previous;
            self.modified_since_baseline = !self.undo.is_empty();
        } else if self.modified_since_baseline {
            self.redo.push(strokes.clone());
            *strokes = self.baseline.clone();
            self.modified_since_baseline = false;
        }
    }

    pub(crate) fn redo(&mut self, strokes: &mut Vec<Arc<StrokeData>>) {
        if let Some(next) = self.redo.pop() {
            self.push_undo(strokes.clone());
            *strokes = next;
            self.modified_since_baseline = true;
        }
    }

    pub(crate) fn can_undo(&self) -> bool {
        !self.undo.is_empty() || self.modified_since_baseline
    }

    pub(crate) fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Push an entry onto the undo ring, evicting the oldest entry
    /// into `baseline` when the ring exceeds `MAX_UNDO_STEPS`.
    fn push_undo(&mut self, entry: Vec<Arc<StrokeData>>) {
        self.undo.push_back(entry);
        if self.undo.len() > MAX_UNDO_STEPS
            && let Some(evicted) = self.undo.pop_front()
        {
            self.baseline = evicted;
        }
    }
}
