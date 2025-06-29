//! Mathematical primitives for 2D graphics and text layout.
//!
//! This module provides fundamental geometric types used throughout the text system,
//! including points, rectangles, and size representations.

/// A 2D rectangle defined by minimum and maximum points.
///
/// Rectangles are used to define text areas, selection bounds, and other
/// rectangular regions in the text layout system.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect {
    /// The minimum (top-left) corner of the rectangle.
    pub min: Point,
    /// The maximum (bottom-right) corner of the rectangle.
    pub max: Point,
}

impl Rect {
    /// Creates a new rectangle from minimum and maximum points.
    ///
    /// # Arguments
    /// * `min` - The minimum (top-left) corner
    /// * `max` - The maximum (bottom-right) corner
    ///
    /// # Examples
    /// ```
    /// use protextinator::math::{Rect, Point};
    /// 
    /// let rect = Rect::new(Point::new(0.0, 0.0), Point::new(100.0, 50.0));
    /// assert_eq!(rect.width(), 100.0);
    /// assert_eq!(rect.height(), 50.0);
    /// ```
    #[inline(always)]
    pub fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }

    /// Returns the height of the rectangle.
    ///
    /// # Examples
    /// ```
    /// use protextinator::math::{Rect, Point};
    /// 
    /// let rect = Rect::new(Point::new(0.0, 10.0), Point::new(100.0, 60.0));
    /// assert_eq!(rect.height(), 50.0);
    /// ```
    #[inline(always)]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Returns the width of the rectangle.
    ///
    /// # Examples
    /// ```
    /// use protextinator::math::{Rect, Point};
    /// 
    /// let rect = Rect::new(Point::new(10.0, 0.0), Point::new(110.0, 50.0));
    /// assert_eq!(rect.width(), 100.0);
    /// ```
    #[inline(always)]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Returns the size of the rectangle as a (width, height) tuple.
    ///
    /// # Examples
    /// ```
    /// use protextinator::math::{Rect, Point};
    /// 
    /// let rect = Rect::new(Point::new(0.0, 0.0), Point::new(100.0, 50.0));
    /// assert_eq!(rect.size(), (100.0, 50.0));
    /// ```
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

/// A 2D point representing a position in 2D space.
///
/// Points are used for positions, sizes, offsets, and other 2D coordinate
/// representations throughout the text system. The coordinate system typically
/// has the origin (0,0) at the top-left, with positive X going right and
/// positive Y going down.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Point {
    /// The X coordinate (horizontal position).
    pub x: f32,
    /// The Y coordinate (vertical position).
    pub y: f32,
}

impl Point {
    /// A point at the origin (0, 0).
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Creates a new point with the specified coordinates.
    ///
    /// # Arguments
    /// * `x` - The X coordinate
    /// * `y` - The Y coordinate
    ///
    /// # Examples
    /// ```
    /// use protextinator::math::Point;
    /// 
    /// let point = Point::new(10.0, 20.0);
    /// assert_eq!(point.x, 10.0);
    /// assert_eq!(point.y, 20.0);
    /// ```
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Converts the point to a tuple of (x, y) coordinates.
    ///
    /// # Examples
    /// ```
    /// use protextinator::math::Point;
    /// 
    /// let point = Point::new(10.0, 20.0);
    /// assert_eq!(point.to_tuple(), (10.0, 20.0));
    /// ```
    #[inline(always)]
    pub fn to_tuple(self) -> (f32, f32) {
        (self.x, self.y)
    }

    /// Checks if this point is approximately equal to another point within a given epsilon.
    ///
    /// This is useful for floating-point comparisons where exact equality may not be reliable.
    ///
    /// # Arguments
    /// * `other` - The other point to compare against
    /// * `epsilon` - The maximum allowed difference for each coordinate
    ///
    /// # Examples
    /// ```
    /// use protextinator::math::Point;
    /// 
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(10.001, 19.999);
    /// 
    /// assert!(p1.approx_eq(&p2, 0.01));
    /// assert!(!p1.approx_eq(&p2, 0.0001));
    /// ```
    #[inline(always)]
    pub fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        (self.x - other.x).abs() <= epsilon && (self.y - other.y).abs() <= epsilon
    }
}

/// Type alias for [`Point`] when used to represent dimensions (width, height).
///
/// While functionally identical to `Point`, this type alias provides semantic clarity
/// when a point is being used to represent a size rather than a position.
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
