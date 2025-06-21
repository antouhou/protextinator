#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    #[inline(always)]
    pub fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }

    #[inline(always)]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    #[inline(always)]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline(always)]
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

impl Point {
    #[inline(always)]
    pub fn to_tuple(self) -> (f32, f32) {
        (self.x, self.y)
    }

    #[inline(always)]
    pub fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        (self.x - other.x).abs() <= epsilon && (self.y - other.y).abs() <= epsilon
    }
}

pub type Size = Point;

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<(i32, i32)> for Point {
    fn from((x, y): (i32, i32)) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
        }
    }
}
