use alloc::vec::Vec;
use approx::RelativeEq;
use num_traits::{real::Real, FloatConst};
use smallvec::SmallVec;

pub use vek::{Aabr, CubicBezier2, LineSegment2, QuadraticBezier2, Vec2};

use crate::math::{mix, solve_cubic, solve_linear, solve_quadratic};

pub type Point2<S> = Vec2<S>;

pub struct Shape<S> {
    edges: Vec<Edge<S>>,
    aabr: Aabr<S>,
}

fn aabr_potentialli_contains_circle<S: Real>(
    aabr: Aabr<S>,
    point: Point2<S>,
    max_distance: S,
) -> bool {
    aabr.min.x - max_distance <= point.x
        && aabr.max.x + max_distance >= point.x
        && aabr.min.y - max_distance <= point.y
        && aabr.max.y + max_distance >= point.y
}

impl<S> Shape<S>
where
    S: Real + FloatConst + RelativeEq + From<u16>,
{
    #[inline]
    pub fn builder() -> ShapeBuilder<S> {
        ShapeBuilder::new()
    }

    #[inline]
    pub fn aabr(&self) -> Aabr<S> {
        self.aabr
    }

    pub fn scanline_intersections(&self, y: S) -> SmallVec<[S; 4]> {
        let mut intersections = SmallVec::new();
        if self.aabr.min.y > y || y > self.aabr.max.y {
            return intersections;
        }
        for edge in &self.edges {
            if edge.aabr.min.y <= y && edge.aabr.max.y >= y {
                intersections.append(&mut edge.scanline_intersections(y))
            }
        }
        intersections
    }

    pub fn closest_point(
        &self,
        point: Point2<S>,
        max_distance: S,
        epsilon: S,
    ) -> Option<(S, Point2<S>)> {
        if !aabr_potentialli_contains_circle(self.aabr, point, max_distance) {
            return None;
        }
        let mut best = None;
        for edge in &self.edges {
            if aabr_potentialli_contains_circle(edge.aabr, point, max_distance) {
                let (distance, p) = edge.closest_point(point, epsilon);
                if distance <= max_distance {
                    if let Some((ref mut best_distance, ref mut best_point)) = best {
                        if *best_distance > distance {
                            *best_distance = distance;
                            *best_point = p;
                        }
                    } else {
                        best = Some((distance, p));
                    }
                }
            }
        }
        best
    }
}

pub struct Edge<S> {
    segment: EdgeSegment<S>,
    color: EdgeColor,
    aabr: Aabr<S>,
    is_new_contour: bool,
}

impl<S> core::ops::Deref for Edge<S> {
    type Target = EdgeSegment<S>;
    #[inline]
    fn deref(&self) -> &EdgeSegment<S> {
        &self.segment
    }
}

pub enum EdgeSegment<S> {
    Linear(LineSegment2<S>),
    Quadratic(QuadraticBezier2<S>),
    Cubic(CubicBezier2<S>),
}

impl<S> From<LineSegment2<S>> for EdgeSegment<S> {
    #[inline]
    fn from(edge: LineSegment2<S>) -> EdgeSegment<S> {
        EdgeSegment::Linear(edge)
    }
}
impl<S> From<QuadraticBezier2<S>> for EdgeSegment<S> {
    #[inline]
    fn from(edge: QuadraticBezier2<S>) -> EdgeSegment<S> {
        EdgeSegment::Quadratic(edge)
    }
}
impl<S> From<CubicBezier2<S>> for EdgeSegment<S> {
    #[inline]
    fn from(edge: CubicBezier2<S>) -> EdgeSegment<S> {
        EdgeSegment::Cubic(edge)
    }
}

trait Segment {
    type Scalar;
    fn aabr(&self) -> Aabr<Self::Scalar>;
    fn evaluate(&self, value: Self::Scalar) -> Point2<Self::Scalar>;
    fn scanline_intersections(&self, y: Self::Scalar) -> SmallVec<[Self::Scalar; 3]>;
    fn closest_point(
        &self,
        point: Point2<Self::Scalar>,
        epsilon: Self::Scalar,
    ) -> (Self::Scalar, Point2<Self::Scalar>);
}

impl<S> Segment for LineSegment2<S>
where
    S: Real + RelativeEq,
{
    type Scalar = S;
    #[inline]
    fn aabr(&self) -> Aabr<Self::Scalar> {
        let mut aabr = Aabr::new_empty(self.start);
        aabr.expand_to_contain_point(self.end);
        aabr
    }
    #[inline]
    fn evaluate(&self, value: Self::Scalar) -> Point2<Self::Scalar> {
        mix(self.start, self.end, value)
    }
    #[inline]
    fn scanline_intersections(&self, y: Self::Scalar) -> SmallVec<[Self::Scalar; 3]> {
        let mut solution = SmallVec::new();
        if self.start.y <= y && y < self.end.y || self.end.y <= y && y < self.start.y {
            if let Some(value) = solve_linear(self.end.y - self.start.y, self.start.y - y) {
                let x = mix(self.start.x, self.end.x, value);
                solution.push(x);
            }
        }
        solution
    }
    #[inline]
    fn closest_point(
        &self,
        point: Point2<Self::Scalar>,
        epsilon: Self::Scalar,
    ) -> (Self::Scalar, Point2<Self::Scalar>) {
        let p = self.projected_point(point);
        (p.distance(point), p)
    }
}

impl<S> Segment for QuadraticBezier2<S>
where
    S: Real + From<u16>,
{
    type Scalar = S;
    #[inline]
    fn aabr(&self) -> Aabr<Self::Scalar> {
        let mut aabr = Aabr::new_empty(self.start);
        aabr.expand_to_contain_point(self.end);
        let bot = (self.ctrl - self.start) - (self.end - self.ctrl);
        if bot.x.abs() < S::epsilon() {
            let value = (self.ctrl.x - self.start.x) / bot.x;
            if S::zero() <= value && value <= S::one() {
                aabr.expand_to_contain_point(self.evaluate(value));
            }
        }
        if bot.y.abs() < S::epsilon() {
            let value = (self.ctrl.y - self.start.y) / bot.y;
            if S::zero() <= value && value <= S::one() {
                aabr.expand_to_contain_point(self.evaluate(value));
            }
        }
        aabr
    }
    #[inline]
    fn evaluate(&self, value: Self::Scalar) -> Point2<Self::Scalar> {
        QuadraticBezier2::evaluate(*self, value)
    }

    #[inline]
    fn scanline_intersections(&self, y: Self::Scalar) -> SmallVec<[Self::Scalar; 3]> {
        let _2 = S::one() + S::one();
        let ba = self.ctrl - self.start;
        let cb2a = self.end - self.ctrl - ba;
        let b2a2 = ba * _2;
        let solutions_values = solve_quadratic(cb2a.y, b2a2.y, self.start.y - y);
        let mut solutions = SmallVec::new();
        for value in solutions_values {
            if value >= S::zero() && value < S::one() {
                let x = cb2a.x * value * value + b2a2.x * value + self.start.x;
                solutions.push(x);
            }
        }
        solutions
    }

    #[inline]
    fn closest_point(
        &self,
        point: Point2<Self::Scalar>,
        epsilon: Self::Scalar,
    ) -> (Self::Scalar, Point2<Self::Scalar>) {
        let (v, mut p) = self.binary_search_point_by_steps(point, 1, epsilon);
        if v < S::zero() {
            p = self.start;
        } else if v > S::one() {
            p = self.end;
        }
        (p.distance(point), p)
    }
}

impl<S> Segment for CubicBezier2<S>
where
    S: Real + FloatConst + From<u16>,
{
    type Scalar = S;
    #[inline]
    fn aabr(&self) -> Aabr<Self::Scalar> {
        let _3 = S::one() + S::one() + S::one();
        let mut aabr = Aabr::new_empty(self.start);
        aabr.expand_to_contain_point(self.end);
        let a = self.ctrl0 - self.start;
        let b = self.ctrl1 - self.ctrl0 - a;
        let b = b + b;
        let c = self.end - self.ctrl1 * _3 + self.ctrl0 * _3 - self.start;
        for solution in solve_quadratic(c.x, b.x, a.x) {
            if solution >= S::zero() && solution <= S::one() {
                aabr.expand_to_contain_point(self.evaluate(solution));
            }
        }
        for solution in solve_quadratic(c.y, b.y, a.y) {
            if solution >= S::zero() && solution <= S::one() {
                aabr.expand_to_contain_point(self.evaluate(solution));
            }
        }
        aabr
    }
    #[inline]
    fn evaluate(&self, value: Self::Scalar) -> Point2<Self::Scalar> {
        CubicBezier2::evaluate(*self, value)
    }

    #[inline]
    fn scanline_intersections(&self, y: Self::Scalar) -> SmallVec<[Self::Scalar; 3]> {
        let _3 = S::one() + S::one() + S::one();
        let ba = self.ctrl0 - self.start;
        let cb2a = self.ctrl1 - self.ctrl0 - ba;
        let b3a3 = ba * _3;
        let c3b6a3 = cb2a * _3;
        let dc3b3a = self.end - self.start + (self.ctrl0 - self.ctrl1) * _3;
        let solutions_values = solve_cubic(dc3b3a.y, c3b6a3.y, b3a3.y, self.start.y - y);
        let mut solutions = SmallVec::new();
        for value in solutions_values {
            if value >= S::zero() && value < S::one() {
                let value_sq = value * value;
                let x = dc3b3a.x * value_sq * value
                    + c3b6a3.x * value_sq
                    + b3a3.x * value
                    + self.start.x;
                solutions.push(x);
            }
        }
        solutions
    }

    #[inline]
    fn closest_point(
        &self,
        point: Point2<Self::Scalar>,
        epsilon: Self::Scalar,
    ) -> (Self::Scalar, Point2<Self::Scalar>) {
        let (v, mut p) = self.binary_search_point_by_steps(point, 2, epsilon);
        if v < S::zero() {
            p = self.start;
        } else if v > S::one() {
            p = self.end;
        }
        (p.distance(point), p)
    }
}

impl<S> EdgeSegment<S>
where
    S: Real + FloatConst + RelativeEq + From<u16>,
{
    #[inline]
    fn aabr(&self) -> Aabr<S> {
        match self {
            EdgeSegment::Linear(e) => Segment::aabr(e),
            EdgeSegment::Quadratic(e) => Segment::aabr(e),
            EdgeSegment::Cubic(e) => Segment::aabr(e),
        }
    }

    #[inline]
    pub fn evaluate(&self, value: S) -> Point2<S> {
        match self {
            EdgeSegment::Linear(e) => Segment::evaluate(e, value),
            EdgeSegment::Quadratic(e) => Segment::evaluate(e, value),
            EdgeSegment::Cubic(e) => Segment::evaluate(e, value),
        }
    }

    #[inline]
    pub fn end(&self) -> Point2<S> {
        match self {
            EdgeSegment::Linear(e) => e.end,
            EdgeSegment::Quadratic(e) => e.end,
            EdgeSegment::Cubic(e) => e.end,
        }
    }

    #[inline]
    fn scanline_intersections(&self, y: S) -> SmallVec<[S; 3]> {
        match self {
            EdgeSegment::Linear(e) => e.scanline_intersections(y),
            EdgeSegment::Quadratic(e) => e.scanline_intersections(y),
            EdgeSegment::Cubic(e) => e.scanline_intersections(y),
        }
    }
    #[inline]
    pub fn closest_point(&self, point: Point2<S>, epsilon: S) -> (S, Point2<S>) {
        match self {
            EdgeSegment::Linear(e) => e.closest_point(point, epsilon),
            EdgeSegment::Quadratic(e) => e.closest_point(point, epsilon),
            EdgeSegment::Cubic(e) => e.closest_point(point, epsilon),
        }
    }
}

pub enum EdgeColor {
    BLACK = 0,
    RED = 1,
    GREEN = 2,
    YELLOW = 3,
    BLUE = 4,
    MAGENTA = 5,
    CYAN = 6,
    WHITE = 7,
}

pub struct ShapeBuilder<S> {
    edges: Vec<Edge<S>>,
    contour_start: Option<Point2<S>>,
    contour_previous: Option<Point2<S>>,
}

impl<S> ShapeBuilder<S>
where
    S: Real + RelativeEq + FloatConst + From<u16>,
{
    #[inline]
    pub fn new() -> ShapeBuilder<S> {
        ShapeBuilder {
            edges: Vec::new(),
            contour_start: None,
            contour_previous: None,
        }
    }
    #[inline]
    pub fn finish(mut self) -> Option<Shape<S>> {
        self.close();
        let mut edges = self.edges.iter();
        if let Some(edge) = edges.next() {
            let mut aabr = edge.aabr;
            while let Some(edge) = edges.next() {
                aabr.expand_to_contain(edge.aabr());
            }
            Some(Shape {
                edges: self.edges,
                aabr,
            })
        } else {
            None
        }
    }
    #[inline]
    pub fn close(&mut self) -> &mut Self {
        if let Some(contour_previous) = self.contour_previous.take() {
            let contour_start = self.contour_start.unwrap();
            if contour_previous != contour_start {
                // close line
                self.push_edge(LineSegment2 {
                    start: contour_previous,
                    end: contour_start,
                });
            }
        }
        self
    }
    #[inline]
    pub fn move_to(&mut self, x: S, y: S) -> &mut Self {
        self.close();
        self.contour_start = Some(Point2::new(x, y));
        self
    }

    #[inline]
    fn push_edge(&mut self, edge: impl Into<EdgeSegment<S>>) -> &mut Self {
        let segment = edge.into();
        let aabr = segment.aabr();
        let is_new_contour = self.contour_previous.is_none();
        let end = segment.end();
        self.edges.push(Edge {
            segment: segment,
            color: EdgeColor::WHITE,
            aabr,
            is_new_contour,
        });
        self.contour_previous = Some(end);
        self
    }

    #[inline]
    fn next_start(&self) -> Point2<S> {
        self.contour_previous
            .or(self.contour_start)
            .expect("move_to not called")
    }

    #[inline]
    pub fn line_to(&mut self, x: S, y: S) -> &mut Self {
        let start = self.next_start();
        let end = Point2::new(x, y);
        if start != end {
            self.push_edge(LineSegment2 { start, end });
        }
        self
    }

    #[inline]
    pub fn quadratic_to(&mut self, cx: S, cy: S, x: S, y: S) -> &mut Self {
        let start = self.next_start();
        let ctrl = Point2::new(cx, cy);
        let end = Point2::new(x, y);
        // TODO: check is line?
        self.push_edge(QuadraticBezier2 { start, ctrl, end })
    }

    #[inline]
    pub fn cubic_to(&mut self, cx0: S, cy0: S, cx1: S, cy1: S, x: S, y: S) -> &mut Self {
        // TODO: check is quadratic?
        self.push_edge(CubicBezier2 {
            start: self.next_start(),
            ctrl0: Point2::new(cx0, cy0),
            ctrl1: Point2::new(cx1, cy1),
            end: Point2::new(x, y),
        })
    }
}
