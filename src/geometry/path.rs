use crate::{Matrix3x3, Point, Rect};

use super::{Coeff, ConicCoeff, CubicCoeff, FLOAT_ROOT2_OVER2, QuadCoeff};

/// A PathVerb describes the type of one or more points in a path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathVerb {
    /// MoveTo indicates the first point of a new subpath.
    MoveTo(Point),
    /// LineTo indicates a line from the current point to the new point.
    /// The new point becomes the current point.
    LineTo(Point),
    /// QuadTo indicates a quadratic Bézier curve from the current point to the new point.
    /// The new point becomes the current point.
    /// The first point is the Bézier control point.
    /// The second point is the end point of the Bézier curve.
    QuadTo(Point, Point),
    /// CubicTo indicates a cubic Bézier curve from the current point to the new point.
    /// The new point becomes the current point.
    /// The first point is the Bézier control point 1.
    /// The second point is the Bézier control point 2.
    /// The third point is the end point of the Bézier curve.
    CubicTo(Point, Point, Point),
    /// ConicTo indicates a conic Bézier curve from the current point to the new point.
    /// The new point becomes the current point.
    /// The first point is the Bézier control point.
    /// The second point is the end point of the Bézier curve.
    /// The third value is the weight of the curve.
    ConicTo(Point, Point, f32),
    /// ClosePath indicates a line from the current point to the first point of the current subpath.
    /// The current point becomes the first point of the current subpath.
    /// This is a path verb that can be used to close a subpath.
    /// It does not have a coordinate argument.
    ///
    /// # Note
    /// When doing stroke for the path.
    /// The ClosePath verb make the last Line joins the first point of the subpath.
    /// Otherwise, the last point and the first point will do Stroke Cap.
    ClosePath,
}

impl PathVerb {
    fn is_move_to(&self) -> bool {
        matches!(self, PathVerb::MoveTo(_))
    }

    fn is_close_path(&self) -> bool {
        matches!(self, PathVerb::ClosePath)
    }
}

/// The FillRule describes how the area to fill in the path. Based on the winding number.
/// The winding number defination is https://en.wikipedia.org/wiki/Winding_number
///
/// The default value is FillRule::Winning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FillRule {
    #[default]
    Winning,
    EvenOdd,
}

/// The Direction describes the direction of the path.
/// The default value is Direction::CW.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    CW,
    CCW,
}

struct PointIterator {
    pts: Vec<Point>,
    current: usize,
    advance: usize,
}

impl PointIterator {
    fn new_direction_start(pts: Vec<Point>, direction: Direction, start: usize) -> Self {
        let count = pts.len();

        Self {
            pts,
            current: (start % count),
            advance: if direction == Direction::CW {
                1
            } else {
                count - 1
            },
        }
    }

    fn from_rect_dir_start(rect: &Rect, direction: Direction, start: usize) -> Self {
        let mut pts = Vec::new();
        pts.push(Point {
            x: rect.left(),
            y: rect.top(),
        });
        pts.push(Point {
            x: rect.right(),
            y: rect.top(),
        });
        pts.push(Point {
            x: rect.right(),
            y: rect.bottom(),
        });
        pts.push(Point {
            x: rect.left(),
            y: rect.bottom(),
        });

        Self::new_direction_start(pts, direction, start)
    }

    fn from_oval_dir_start(rect: &Rect, direction: Direction, start: usize) -> Self {
        let mut pts = Vec::new();
        let cx = rect.center().x;
        let cy = rect.center().y;

        pts.push(Point {
            x: cx,
            y: rect.top(),
        });
        pts.push(Point {
            x: rect.right(),
            y: cy,
        });
        pts.push(Point {
            x: cx,
            y: rect.bottom(),
        });
        pts.push(Point {
            x: rect.left(),
            y: cy,
        });

        Self::new_direction_start(pts, direction, start)
    }

    pub(crate) fn current(&self) -> Point {
        assert!(self.current < self.pts.len());

        return self.pts[self.current];
    }

    fn next(&mut self) -> Point {
        let n = self.pts.len();
        self.current = (self.current + self.advance) % n;

        let pt = self.pts[self.current];

        return pt;
    }
}

/// Path describes a 2D geometry shape composed of lines or curves.
/// It can be empty or contain one or more sub-paths.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    verbs: Vec<PathVerb>,
    fill_rule: FillRule,
    last_move_to_index: Option<usize>,
}

impl Path {
    /// Create a new empty path.
    pub fn new() -> Path {
        Path {
            verbs: vec![],
            fill_rule: FillRule::Winning,
            last_move_to_index: None,
        }
    }

    /// Set the fill rule.
    ///
    /// # Arguments
    /// * `fill_rule` - The [`FillRule`] to be set for this path.
    ///
    /// # Returns
    /// The path.
    pub fn with_fill_rule(mut self, fill_rule: FillRule) -> Self {
        self.fill_rule = fill_rule;
        self
    }

    /// Get the [`FillRule`] of this path.
    ///
    /// # Returns
    /// The fill rule.
    pub fn fill_rule(&self) -> FillRule {
        self.fill_rule
    }

    /// Move the current point to the specified point.
    /// The new point becomes the current point.
    /// It also starts a new subpath.
    ///
    /// # Arguments
    /// * `point` - The new point.
    ///
    /// # Returns
    /// The path.
    pub fn move_to<T: Into<Point>>(mut self, p: T) -> Self {
        if self.verbs.last().is_some() && self.verbs.last().unwrap().is_move_to() {
            // replace the last MoveTo verb
            self.verbs[self.last_move_to_index.unwrap()] = PathVerb::MoveTo(p.into());
            return self;
        }

        self.verbs.push(PathVerb::MoveTo(p.into()));
        self.last_move_to_index = Some(self.verbs.len() - 1);
        self
    }

    /// Add a line from the current point to the specified point.
    /// The new point becomes the current point.
    /// If no previous MoveTo verb has been called, will insert a MoveTo verb from (0.0, 0.0).
    ///
    /// # Arguments
    /// * `point` - The new point.
    ///
    /// # Returns
    /// The path.
    pub fn line_to<T: Into<Point>>(mut self, point: T) -> Self {
        self.inject_move_to_if_needed();

        self.verbs.push(PathVerb::LineTo(point.into()));
        self
    }

    /// Add a quadratic Bézier curve from the current point to the specified point.
    /// The new point becomes the current point.
    /// If no previous MoveTo verb has been called, will insert a MoveTo verb from (0.0, 0.0).
    ///
    /// # Arguments
    /// * `control_point` - The Bézier control point.
    /// * `end_point` - The end point of the Bézier curve.
    ///
    /// # Returns
    /// The path.
    pub fn quad_to<T: Into<Point>>(mut self, control_point: T, end_point: T) -> Self {
        self.inject_move_to_if_needed();

        self.verbs
            .push(PathVerb::QuadTo(control_point.into(), end_point.into()));
        self
    }

    /// Add a cubic Bézier curve from the current point to the specified point.
    /// The new point becomes the current point.
    /// If no previous MoveTo verb has been called, will insert a MoveTo verb from (0.0, 0.0).
    ///
    /// # Arguments
    /// * `point_1` - The Bézier control point 1.
    /// * `point_2` - The Bézier control point 2.
    /// * `end_point` - The end point of the Bézier curve.
    ///
    /// # Returns
    /// The path.
    pub fn cubic_to(mut self, point_1: Point, point_2: Point, end_point: Point) -> Self {
        self.inject_move_to_if_needed();
        self.verbs
            .push(PathVerb::CubicTo(point_1, point_2, end_point));
        self
    }

    /// Add a conic Bézier curve from the current point to the specified point.
    /// The new point becomes the current point.
    /// If no previous MoveTo verb has been called, will insert a MoveTo verb from (0.0, 0.0).
    ///
    /// # Arguments
    /// * `control_point` - The Bézier control point.
    /// * `end_point` - The end point of the Bézier curve.
    /// * `weight` - The weight of the curve.
    ///
    /// # Returns
    /// The path.
    pub fn conic_to<T: Into<Point>>(mut self, control_point: T, end_point: T, weight: f32) -> Self {
        self.inject_move_to_if_needed();

        self.verbs.push(PathVerb::ConicTo(
            control_point.into(),
            end_point.into(),
            weight,
        ));
        self
    }

    /// Add a rectangle to the path in default [`Direction::CW`] direction.
    /// Add a MoveTo from the top-left corner of the rectangle.
    /// Add a LineTo to the top-right corner of the rectangle.
    /// Add a LineTo to the bottom-right corner of the rectangle.
    /// Add a LineTo to the bottom-left corner of the rectangle.
    /// Add a ClosePath verb to the path.
    ///
    /// # Arguments
    /// * `rect` - The rectangle.
    ///
    /// # Returns
    /// The path.
    pub fn add_rect(self, rect: &Rect) -> Self {
        self.add_rect_with_direction(rect, Default::default(), 0)
    }

    /// Add a rectangle to the path in the specified direction.
    ///
    /// # Arguments
    /// * `rect` - The rectangle.
    /// * `direction` - The direction.
    /// * `start` - The start index of the rectangle.
    ///
    /// # Returns
    /// The path.
    pub fn add_rect_with_direction(self, rect: &Rect, direction: Direction, start: usize) -> Self {
        if rect.is_empty() {
            return self;
        }

        let mut iter = PointIterator::from_rect_dir_start(rect, direction, start);

        self.move_to(iter.next())
            .line_to(iter.next())
            .line_to(iter.next())
            .line_to(iter.next())
            .close_path()
    }

    /// Add an oval to the path in direction.
    ///
    /// # Arguments
    /// * `oval` - The oval.
    /// * `direction` - The direction.
    /// * `start` - The start index of the oval.
    ///
    /// # Returns
    /// The path.
    pub fn add_oval_with_direction(self, oval: &Rect, direction: Direction, start: usize) -> Self {
        if oval.is_empty() {
            return self;
        }

        let mut oval_iter = PointIterator::from_oval_dir_start(oval, direction, start);
        let mut rect_iter = PointIterator::from_rect_dir_start(oval, direction, start);

        let weight = FLOAT_ROOT2_OVER2;

        return self
            .move_to(oval_iter.current())
            .conic_to(rect_iter.next(), oval_iter.next(), weight)
            .conic_to(rect_iter.next(), oval_iter.next(), weight)
            .conic_to(rect_iter.next(), oval_iter.next(), weight)
            .conic_to(rect_iter.next(), oval_iter.next(), weight)
            .close_path();
    }

    /// Add an oval to the path. In default [`Direction::CW`] direction.
    ///
    /// # Arguments
    /// * `oval` - The oval.
    ///
    /// # Returns
    /// The path.
    pub fn add_oval(self, oval: &Rect) -> Self {
        self.add_oval_with_direction(oval, Default::default(), 0)
    }

    /// Add a circle to the path in direction.
    ///
    /// # Arguments
    /// * `center` - The center of the circle.
    /// * `radius` - The radius of the circle.
    /// * `direction` - The direction.
    ///
    /// # Returns
    /// The path.
    pub fn add_circle_with_direction<T: Into<Point>>(
        self,
        center: T,
        radius: f32,
        direction: Direction,
    ) -> Self {
        if radius <= 0.0 {
            self
        } else {
            let center = center.into();
            let cx = center.x;
            let cy = center.y;

            self.add_oval_with_direction(
                &Rect::new_ltrb(cx - radius, cy - radius, cx + radius, cy + radius),
                direction,
                1,
            )
        }
    }

    /// Add a circle to the path. In default [`Direction::CW`] direction.
    ///
    /// # Arguments
    /// * `center` - The center of the circle.
    /// * `radius` - The radius of the circle.
    ///
    /// # Returns
    /// The path.
    pub fn add_circle<T: Into<Point>>(self, center: T, radius: f32) -> Self {
        self.add_circle_with_direction(center, radius, Default::default())
    }

    /// Add a ClosePath verb to the path.
    /// If no previous MoveTo verb has been called, nothing will be added.
    ///
    /// # Returns
    /// The path.
    pub fn close_path(mut self) -> Self {
        if self.verbs.last().is_some() && self.verbs.last().unwrap().is_close_path() {
            return self;
        }

        if self.last_move_to_index.is_some() {
            self.verbs.push(PathVerb::ClosePath);
        }

        self
    }

    fn inject_move_to_if_needed(&mut self) {
        if self.last_move_to_index.is_none() {
            self.verbs.push(PathVerb::MoveTo(Point::new(0.0, 0.0)));
            self.last_move_to_index = Some(self.verbs.len() - 1);
        }
    }
}

pub(crate) struct Contour {
    pub(crate) points: Vec<Point>,
    pub(crate) closed: bool,
}

impl Contour {
    pub(crate) fn new() -> Self {
        Self {
            points: Vec::new(),
            closed: false,
        }
    }
    pub(crate) fn add_point(&mut self, p: Point) {
        if self.points.is_empty() || self.points.last().unwrap() != &p {
            self.points.push(p);
        }
    }

    pub(crate) fn last_point(&self) -> Option<&Point> {
        self.points.last()
    }
}

pub(crate) struct PolylineBuilder<'a> {
    path: &'a Path,
    matrix: Matrix3x3,
}

impl<'a> PolylineBuilder<'a> {
    pub(crate) fn new(path: &'a Path, matrix: Matrix3x3) -> Self {
        Self { path, matrix }
    }

    fn create_contours(self) -> Vec<Contour> {
        let mut contours: Vec<Contour> = Vec::new();

        for v in &self.path.verbs {
            match v {
                PathVerb::MoveTo(p) => {
                    contours.push(Contour::new());

                    contours
                        .last_mut()
                        .expect("Not create contour")
                        .add_point(p.clone());
                }
                PathVerb::LineTo(p) => {
                    contours
                        .last_mut()
                        .expect("Not create contour")
                        .add_point(p.clone());
                }
                PathVerb::QuadTo(ctr, end) => {
                    let p1 = contours
                        .last()
                        .expect("Not create contour")
                        .last_point()
                        .expect("Not start contour");
                    let quad = QuadCoeff::from(p1, ctr, end);

                    let stops = QuadCoeff::flatten(*p1, *ctr, *end, self.matrix);

                    // TODO: flatten curve dynamic with line count
                    for step in stops {
                        contours.last_mut().unwrap().add_point(quad.eval(step));
                    }
                }
                PathVerb::ConicTo(p2, p3, weight) => {
                    let p1 = contours
                        .last()
                        .expect("Not create contour")
                        .last_point()
                        .expect("Not start contour");

                    let conic = ConicCoeff::from(p1, p2, p3, *weight);

                    let stops = ConicCoeff::flatten(*p1, *p2, *p3, *weight, self.matrix);

                    // TODO: flatten curve dynamic with line count
                    for step in stops {
                        contours.last_mut().unwrap().add_point(conic.eval(step));
                    }
                }
                PathVerb::CubicTo(p2, p3, p4) => {
                    let p1 = contours
                        .last()
                        .expect("Not create contour")
                        .last_point()
                        .expect("Not start contour");
                    let cubic = CubicCoeff::from(p1, p2, p3, p4);

                    let stops = CubicCoeff::flatten(*p1, *p2, *p3, *p4, self.matrix);

                    // TODO: flatten curve dynamic with line count
                    for step in stops {
                        contours.last_mut().unwrap().add_point(cubic.eval(step));
                    }
                }
                PathVerb::ClosePath => {
                    contours.last_mut().expect("Not start contour").closed = true;
                }
            }
        }

        return contours;
    }

    pub(crate) fn build(self) -> Vec<Contour> {
        let contours = self.create_contours();

        return contours;
    }
}
