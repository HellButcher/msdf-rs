use approx::RelativeEq;
use num_traits::{real::Real, FloatConst, NumCast};
use vek::Vec2;

use crate::{
    math::{max, min},
    shape::{Point2, Shape},
};

pub struct Rasterizer<S> {
    pub scale: Vec2<S>,
    pub translate: Vec2<S>,
}

impl<S> Rasterizer<S>
where
    S: Real + FloatConst + RelativeEq + From<u16>,
{
    pub fn new() -> Self {
        Rasterizer {
            scale: Vec2::new(S::one(), S::one()),
            translate: Vec2::new(S::zero(), S::zero()),
        }
    }

    pub fn with_scale(mut self, scale: S) -> Self {
        self.scale = Vec2::new(scale, scale);
        self
    }
    pub fn with_scale2(mut self, scale_x: S, scale_y: S) -> Self {
        self.scale = Vec2::new(scale_x, scale_y);
        self
    }
    pub fn with_translate(mut self, x: S, y: S) -> Self {
        self.translate = Vec2::new(x, y);
        self
    }

    pub fn rasterize_bitmap<F>(
        &self,
        shape: &Shape<S>,
        width: usize,
        height: usize,
        mut draw_pixel: F,
    ) where
        F: FnMut(usize, usize, bool),
    {
        let half = S::one() / (S::one() + S::one());
        for y in 0..height {
            let f_y: S = NumCast::from(y).unwrap();
            let p_y = (f_y + half) / self.scale.y - self.translate.y;
            let scanline = shape.scanline(p_y);
            for x in 0..width {
                let f_x: S = NumCast::from(x).unwrap();
                let p_x = (f_x + half) / self.scale.x - self.translate.x;
                let filled = scanline.is_filled(p_x);
                draw_pixel(x, height - y - 1, filled);
            }
        }
    }

    pub fn rasterize_sdf<F>(
        &self,
        shape: &Shape<S>,
        width: usize,
        height: usize,
        offset: u8,
        mut draw_pixel: F,
    ) where
        F: FnMut(usize, usize, u8),
    {
        let half = S::one() / (S::one() + S::one());
        let min_scale = min(self.scale.x, self.scale.y);
        let epsilon = min_scale / <S as From<u16>>::from(offset as u16);
        let scaled_offset: S = <S as From<u16>>::from(offset as u16) / min_scale;
        for y in 0..height {
            let f_y: S = NumCast::from(y).unwrap();
            let p_y = (f_y + half) / self.scale.y - self.translate.y;
            let scanline = shape.scanline(p_y);
            for x in 0..width {
                let f_x: S = NumCast::from(x).unwrap();
                let p_x = (f_x + half) / self.scale.x - self.translate.x;
                let filled = scanline.is_filled(p_x);
                let value = if let Some((distance, _)) =
                    shape.closest_point(Point2::new(p_x, p_y), scaled_offset, epsilon)
                {
                    let distance = (distance * From::from(128) / scaled_offset)
                        .to_u8()
                        .unwrap();
                    if filled {
                        128 + distance
                    } else {
                        128 - distance
                    }
                } else {
                    0
                };
                draw_pixel(x, height - y - 1, value);
            }
        }
    }
}
