use crate::{Color, Matrix3x3, PaintColor, Point};

/// Common gradient info contains colors, optional stops and the local matrix
#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    /// colors of the gradient
    pub colors: Vec<Color>,
    /// optional stops of the gradient, if set the len must equal to colors len
    pub stops: Option<Vec<f32>>,
    /// local matrix of the gradient, default is identity matrix
    pub local_matrix: Matrix3x3,
}

/// Represent a linear gradient between two specified points
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    pub start: Point,
    pub end: Point,
    pub gradient: Gradient,
}

/// Represent a radial gradient with a specified center and radius
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    pub center: Point,
    pub radius: f32,
    pub gradient: Gradient,
}

impl Gradient {
    pub fn new(colors: Vec<Color>, stops: Option<Vec<f32>>, local_matrix: Matrix3x3) -> Self {
        if colors.len() < 2 {
            panic!("colors len must greater than 2");
        }

        if stops.is_some() {
            if stops.as_ref().unwrap().len() != colors.len() {
                panic!("stops is set, the len must equal to colors len");
            }
        }

        Self {
            colors,
            stops,
            local_matrix,
        }
    }

    pub(crate) fn is_simple(&self) -> bool {
        if self.colors.len() > 2 {
            return false;
        }

        if self.stops.is_some() {
            let stops = self.stops.as_ref().unwrap();

            if stops.len() == 2 && stops[0] == 0.0 && stops[1] == 1.0 {
                return true;
            }

            return false;
        }

        true
    }
}

impl Into<Gradient> for Vec<Color> {
    fn into(self) -> Gradient {
        Gradient::new(self, None, Matrix3x3::default())
    }
}

impl Into<Gradient> for (Vec<Color>, Vec<f32>) {
    fn into(self) -> Gradient {
        Gradient::new(self.0, Some(self.1), Matrix3x3::default())
    }
}

impl Into<Gradient> for (Vec<Color>, Matrix3x3) {
    fn into(self) -> Gradient {
        Gradient::new(self.0, None, self.1)
    }
}

impl Into<Gradient> for (Vec<Color>, Vec<f32>, Matrix3x3) {
    fn into(self) -> Gradient {
        Gradient::new(self.0, Some(self.1), self.2)
    }
}

impl LinearGradient {
    pub fn new<T: Into<Gradient>>(start: Point, end: Point, gradient: T) -> Self {
        Self {
            start,
            end,
            gradient: gradient.into(),
        }
    }
}

impl<T: Into<Gradient>> Into<LinearGradient> for (Point, Point, T) {
    fn into(self) -> LinearGradient {
        LinearGradient::new(self.0, self.1, self.2)
    }
}

impl Into<PaintColor> for LinearGradient {
    fn into(self) -> PaintColor {
        PaintColor::LinearGradient(self)
    }
}

impl RadialGradient {
    pub fn new<T: Into<Gradient>>(center: Point, radius: f32, gradient: T) -> Self {
        Self {
            center,
            radius,
            gradient: gradient.into(),
        }
    }
}

impl<T: Into<Gradient>> Into<RadialGradient> for (Point, f32, T) {
    fn into(self) -> RadialGradient {
        RadialGradient::new(self.0, self.1, self.2)
    }
}

impl Into<PaintColor> for RadialGradient {
    fn into(self) -> PaintColor {
        PaintColor::RadialGradient(self)
    }
}
