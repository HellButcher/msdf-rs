use approx::RelativeEq;
use core::{cell::Cell, cmp::Ordering};
use num_traits::{real::Real, FloatConst};
use smallvec::SmallVec;

use crate::shape::Shape;

pub struct Scanline<S> {
    intersections: SmallVec<[S; 4]>,
    index: Cell<usize>,
}

impl<S> Scanline<S>
where
    S: Real + FloatConst,
{
    #[inline]
    pub fn reset(&mut self) -> &mut Self {
        self.index.set(0);
        self
    }

    fn move_to(&self, x: S) -> usize {
        let mut index = self.index.get();
        while index > 0 && (index >= self.intersections.len() || x < self.intersections[index]) {
            index -= 1;
        }
        while index < self.intersections.len() && x >= self.intersections[index] {
            index += 1;
        }
        self.index.set(index);
        index
    }

    #[inline]
    pub fn is_filled(&self, x: S) -> bool {
        self.move_to(x) & 1 != 0
    }
}

impl<S> Shape<S>
where
    S: Real + FloatConst + RelativeEq + From<u16>,
{
    pub fn scanline(&self, y: S) -> Scanline<S> {
        let mut intersections = self.scanline_intersections(y);
        intersections.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        Scanline {
            intersections,
            index: Cell::new(0),
        }
    }
}
