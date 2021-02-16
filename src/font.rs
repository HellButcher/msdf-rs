use crate::shape::ShapeBuilder;

impl ttf_parser::OutlineBuilder for ShapeBuilder<f32> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.quadratic_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.close();
    }
}

impl ttf_parser::OutlineBuilder for ShapeBuilder<f64> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.move_to(x as f64, y as f64);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.line_to(x as f64, y as f64);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.quadratic_to(x1 as f64, y1 as f64, x as f64, y as f64);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.cubic_to(x1 as f64, y1 as f64, x2 as f64, y2 as f64, x as f64, y as f64);
    }

    fn close(&mut self) {
        self.close();
    }
}
