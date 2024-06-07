use std::ops::Mul;

use nalgebra::{Matrix4, Vector3, Vector4};

use crate::{geometry::degree_to_radian, Point, Rect};

/// Holds the matrix information which can be used to transform the Point, Rect or other geometries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix {
    pub(crate) matrix: Matrix4<f32>,
}

impl Matrix {
    /// Creates a new identity matrix.
    pub fn new() -> Self {
        Self {
            matrix: Matrix4::identity(),
        }
    }

    /// Check if the matrix is identity.
    pub fn is_identity(&self) -> bool {
        self.matrix.is_identity(f32::EPSILON)
    }

    pub fn is_invertible(&self) -> bool {
        self.matrix.is_invertible()
    }

    /// Check if the matrix has rotation.
    pub fn has_rotation(&self) -> bool {
        // check if the matrix has rotation
        // If m12 or m21 is not zero, then the matrix has rotation
        self.matrix.m12 != 0.0 || self.matrix.m21 != 0.0
    }

    /// Append translate to this matrix.
    ///
    /// # Arguments
    ///
    /// * `x` - Translate distance at x-axis.
    /// * `y` - Translate distance at y-axis.
    pub fn translate(&mut self, x: f32, y: f32) {
        self.matrix = Matrix4::new_translation(&Vector3::new(x, y, 0.0)) * self.matrix;
    }

    /// Append scale to this matrix.
    ///
    /// # Arguments
    ///
    /// * `sx` - Scale factor at x-axis.
    /// * `sy` - Scale factor at y-axis.
    pub fn scale(&mut self, sx: f32, sy: f32) {
        let s: Matrix4<f32> = Matrix4::new(
            sx, 0.0, 0.0, 0.0, 0.0, sy, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        );

        self.matrix = s * self.matrix;
    }

    /// Append rotate to this matrix.
    ///
    /// # Arguments
    ///
    /// * `degree` - Rotate in degree.
    pub fn rotate(&mut self, degree: f32) {
        let rotate = Matrix4::new_rotation(Vector3::new(0.0, 0.0, degree_to_radian(degree)));

        self.matrix = rotate * self.matrix;
    }

    /// Append rotate around at point to this matrix.
    ///
    /// # Arguments
    ///
    /// * `degree` - Rotate in degree.
    /// * `px` - x coordinate of the point.
    /// * `py` - y coordinate of the point.
    pub fn rotate_at(&mut self, degree: f32, px: f32, py: f32) {
        let rotate = Matrix4::new_rotation(Vector3::new(0.0, 0.0, degree_to_radian(degree)));
        let pre = Matrix4::new_translation(&Vector3::new(-px, -py, 0.0));
        let post = Matrix4::new_translation(&Vector3::new(px, py, 0.0));

        self.matrix = post * rotate * pre * self.matrix;
    }

    /// Invert this matrix.
    ///
    /// # Returns
    ///
    /// If the matrix is invertible, return the inverted matrix.
    /// Otherwise, return None.
    pub fn try_invert(&self) -> Option<Self> {
        self.matrix.try_inverse().map(|m| Self { matrix: m })
    }

    /// Apply this matrix to the point.
    ///
    /// # Arguments
    ///
    /// * `point` - The point to be transformed.
    pub fn map_point(&self, point: &Point) -> Point {
        if self.is_identity() {
            return point.clone();
        }

        let vector = self.matrix * Vector4::new(point.x, point.y, 0.0, 1.0);

        return Point::from(vector.x, vector.y);
    }

    /// Apply this matrix to the rect.
    ///
    /// # Arguments
    ///
    /// * `rect` - The rect to be transformed.
    pub fn map_rect(&self, rect: &Rect) -> Rect {
        if self.is_identity() {
            return rect.clone();
        }

        let top_left = self.map_point(&Point::from(rect.left, rect.top));
        let top_right = self.map_point(&Point::from(rect.right, rect.top));
        let bottom_left = self.map_point(&Point::from(rect.left, rect.bottom));
        let bottom_right = self.map_point(&Point::from(rect.right, rect.bottom));

        // If the rect is not a rectangle, we need to find the minimum and maximum x and y values
        // and create a new rect from them.
        let min_x = f32::min(
            f32::min(top_left.x, top_right.x),
            f32::min(bottom_left.x, bottom_right.x),
        );

        let min_y = f32::min(
            f32::min(top_left.y, top_right.y),
            f32::min(bottom_left.y, bottom_right.y),
        );

        let max_x = f32::max(
            f32::max(top_left.x, top_right.x),
            f32::max(bottom_left.x, bottom_right.x),
        );

        let max_y = f32::max(
            f32::max(top_left.y, top_right.y),
            f32::max(bottom_left.y, bottom_right.y),
        );

        return Rect::from_ltrb(min_x, min_y, max_x, max_y);
    }
}

impl Mul for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            matrix: self.matrix * rhs.matrix,
        }
    }
}

impl Mul<Point> for Matrix {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        self.map_point(&rhs)
    }
}

impl Mul<Rect> for Matrix {
    type Output = Rect;

    fn mul(self, rhs: Rect) -> Self::Output {
        self.map_rect(&rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_transform() {
        let mut matrix = Matrix::new();

        assert!(matrix.is_identity());

        matrix.translate(10.0, 20.0);
        matrix.scale(2.0, 3.0);

        assert!(!matrix.is_identity());
        assert!(!matrix.has_rotation());

        matrix.rotate(45.0);

        assert!(matrix.has_rotation());
    }

    #[test]
    fn test_matrix_map_point() {
        let mut matrix = Matrix::new();
        matrix.translate(10.0, 20.0);
        matrix.scale(2.0, 3.0);

        let point = Point::from(10.0, 20.0);

        let transformed_point = matrix * point;

        assert_eq!(transformed_point.x, 40.0);
        assert_eq!(transformed_point.y, 120.0);
    }
}
