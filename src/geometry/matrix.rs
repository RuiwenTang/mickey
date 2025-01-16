use std::ops::Mul;

use nalgebra::{Matrix3, Matrix4, Vector2, Vector3};

use crate::{Angle, Point};

/// A 3x3 matrix.
/// The matrix is represented by a 3x3 array of floating-point values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3x3 {
    m: Matrix3<f32>,
}

impl Matrix3x3 {
    /// Create a new 3x3 matrix with the given values.
    ///
    /// # Arguments
    /// * `m00` - The value at row 0, column 0.
    /// * `m01` - The value at row 0, column 1.
    /// * `m02` - The value at row 0, column 2.
    /// * `m10` - The value at row 1, column 0.
    /// * `m11` - The value at row 1, column 1.
    /// * `m12` - The value at row 1, column 2.
    /// * `m20` - The value at row 2, column 0.
    /// * `m21` - The value at row 2, column 1.
    /// * `m22` - The value at row 2, column 2.
    pub fn new(
        m00: f32,
        m01: f32,
        m02: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m20: f32,
        m21: f32,
        m22: f32,
    ) -> Matrix3x3 {
        Matrix3x3 {
            m: Matrix3::new(m00, m01, m02, m10, m11, m12, m20, m21, m22),
        }
    }

    /// Get the value at row 0, column 0.
    pub fn m00(&self) -> f32 {
        self.m[(0, 0)]
    }

    /// Get the value at row 0, column 1.
    pub fn m01(&self) -> f32 {
        self.m[(0, 1)]
    }

    /// Get the value at row 0, column 2.
    pub fn m02(&self) -> f32 {
        self.m[(0, 2)]
    }

    /// Get the value at row 1, column 0.
    pub fn m10(&self) -> f32 {
        self.m[(1, 0)]
    }

    /// Get the value at row 1, column 1.
    pub fn m11(&self) -> f32 {
        self.m[(1, 1)]
    }

    /// Get the value at row 1, column 2.
    pub fn m12(&self) -> f32 {
        self.m[(1, 2)]
    }

    /// Get the value at row 2, column 0.
    pub fn m20(&self) -> f32 {
        self.m[(2, 0)]
    }

    /// Get the value at row 2, column 1.
    pub fn m21(&self) -> f32 {
        self.m[(2, 1)]
    }

    /// Get the value at row 2, column 2.
    pub fn m22(&self) -> f32 {
        self.m[(2, 2)]
    }

    /// Translate the matrix by the given x and y values.
    /// This equivalent to applying a translation matrix to the original matrix.
    ///
    /// # Arguments
    /// * `x` - The x value to translate by.
    /// * `y` - The y value to translate by.
    ///
    /// # Returns
    /// A new matrix that is the result of translating the original matrix by the given x and y values.
    pub fn translate(self, x: f32, y: f32) -> Self {
        Matrix3x3 {
            m: self.m * Matrix3::new_translation(&Vector2::new(x, y)),
        }
    }

    /// Scale the matrix by the given x and y values.
    /// This equivalent to applying a scaling matrix to the original matrix.
    ///
    /// # Arguments
    /// * `x` - The x value to scale by.
    /// * `y` - The y value to scale by.
    ///
    /// # Returns
    /// A new matrix that is the result of scaling the original matrix by the given x and y values.
    pub fn scale(self, x: f32, y: f32) -> Self {
        Matrix3x3 {
            m: self.m * Matrix3::new_nonuniform_scaling(&Vector2::new(x, y)),
        }
    }

    /// Rotate the matrix by the given value.
    /// This equivalent to applying a rotation matrix to the original matrix.
    ///
    /// # Arguments
    /// * `value` - The value to rotate by.
    ///
    /// # Returns
    /// A new matrix that is the result of rotating the original matrix by the given value.
    pub fn rotate<T: Angle>(self, value: T) -> Self {
        Matrix3x3 {
            m: self.m * Matrix3::new_rotation(value.radians()),
        }
    }

    pub fn rotate_at<T: Angle>(self, point: Point, value: T) -> Self {
        let m = Matrix3::new_rotation(value.radians());
        let pre = Vector2::new(-point.x, -point.y);
        let post = Vector2::new(point.x, point.y);
        let m = Matrix3::new_translation(&post) * m * Matrix3::new_translation(&pre);
        Matrix3x3 { m: self.m * m }
    }

    /// Get the inverse of the matrix.
    ///
    /// # Returns
    /// A new matrix that is the inverse of the original matrix. Or None if the matrix is not invertible.
    pub fn inverse(&self) -> Option<Self> {
        let m = self.m.try_inverse();

        match m {
            Some(m) => Some(Matrix3x3 { m }),
            None => None,
        }
    }
}

impl Default for Matrix3x3 {
    fn default() -> Self {
        Matrix3x3 {
            m: Matrix3::identity(),
        }
    }
}

impl Mul<Matrix3x3> for Matrix3x3 {
    type Output = Matrix3x3;
    fn mul(self, other: Matrix3x3) -> Matrix3x3 {
        Matrix3x3 {
            m: self.m * other.m,
        }
    }
}

impl Mul<Point> for Matrix3x3 {
    type Output = Point;
    fn mul(self, other: Point) -> Point {
        let v = Vector3::new(other.x, other.y, 1.0);
        let v = self.m * v;

        Point { x: v.x, y: v.y }
    }
}

impl Into<[f32; 16]> for Matrix3x3 {
    fn into(self) -> [f32; 16] {
        // extends 3x3 matrix to 4x4 matrix
        // [m00, m01, 0.0, m02,
        //  m10, m11, 0.0, m12,
        //  0.0, 0.0, 1.0, 0.0,
        //  m20, m21, 0.0, m22]

        [
            self.m00(),
            self.m01(),
            0.0,
            self.m02(),
            self.m10(),
            self.m11(),
            0.0,
            self.m12(),
            0.0,
            0.0,
            1.0,
            0.0,
            self.m20(),
            self.m21(),
            0.0,
            self.m22(),
        ]
    }
}
