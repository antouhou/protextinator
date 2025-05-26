#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    pub fn new(min: Point, max: Point) -> Self {
        Self {
            min, max
        }
    }
    
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
    
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }
    
    pub fn size(&self) -> Point {
        Point {
            x: self.width(),
            y: self.height(),
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