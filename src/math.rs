#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    pub fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn size(&self) -> (f32, f32) {
        (self.width(), self.height())
    }
}

impl From<(Point, Point)> for Rect {
    fn from((x, y): (Point, Point)) -> Self {
        Self { min: x, max: y }
    }
}

impl From<((f32, f32), (f32, f32))> for Rect {
    fn from((min, max): ((f32, f32), (f32, f32))) -> Self {
        Self {
            min: min.into(),
            max: max.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}
