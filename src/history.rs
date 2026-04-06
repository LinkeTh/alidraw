use std::collections::VecDeque;

use crate::canvas::StrokeData;

const MAX_UNDO_STEPS: usize = 10;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct History {
    undo: VecDeque<Vec<StrokeData>>,
    redo: Vec<Vec<StrokeData>>,
    baseline: Vec<StrokeData>,
}

impl History {
    pub fn set_baseline(&mut self, strokes: &[StrokeData]) {
        self.baseline = strokes.to_vec();
        self.undo.clear();
        self.redo.clear();
    }

    pub fn snapshot(&mut self, strokes: &[StrokeData]) {
        self.undo.push_back(strokes.to_vec());
        if self.undo.len() > MAX_UNDO_STEPS
            && let Some(evicted) = self.undo.pop_front()
        {
            self.baseline = evicted;
        }
        self.redo.clear();
    }

    pub fn undo(&mut self, strokes: &mut Vec<StrokeData>) {
        if let Some(previous) = self.undo.pop_back() {
            self.redo.push(strokes.clone());
            *strokes = previous;
        } else if strokes != &self.baseline {
            self.redo.push(strokes.clone());
            *strokes = self.baseline.clone();
        }
    }

    pub fn redo(&mut self, strokes: &mut Vec<StrokeData>) {
        if let Some(next) = self.redo.pop() {
            self.undo.push_back(strokes.clone());
            if self.undo.len() > MAX_UNDO_STEPS
                && let Some(evicted) = self.undo.pop_front()
            {
                self.baseline = evicted;
            }
            *strokes = next;
        }
    }

    pub fn can_undo(&self, strokes: &[StrokeData]) -> bool {
        !self.undo.is_empty() || strokes != self.baseline.as_slice()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}
