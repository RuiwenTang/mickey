use super::Point;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathVerb {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    CubicTo(Point, Point, Point),
    Close,
}

/// The fill type of a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathFillType {
    /// Specifies that "inside" is computed by a non-zero sum of signed edge crossings
    Winding,
    /// Specifies that "inside" is computed by an odd number of edge crossings
    EvenOdd,
}

impl Default for PathFillType {
    fn default() -> Self {
        Self::Winding
    }
}

/// Path contain geometry.Path may be empty, or contain one or more verbs that outline a figure.
/// Path always starts with a move verb to a Cartesian coordinate, and may be followed by additional verbs that add lines or curves.
/// Adding a close verb makes the geometry into a continuous loop, a closed contour.
/// A path instance may contain any number of contours, each beginning with a move verb.
#[derive(Debug, Clone)]
pub struct Path {
    pub verts: Vec<PathVerb>,

    last_move_to_index: Option<usize>,

    pub fill_type: PathFillType,
}

impl Path {
    pub fn new(fill_type: PathFillType) -> Self {
        Self {
            verts: Vec::new(),
            last_move_to_index: None,
            fill_type,
        }
    }

    fn inject_move_to_if_needed(&mut self) {
        if self.verts.is_empty() || self.last_move_to_index.is_none() {
            self.verts.push(PathVerb::MoveTo(Point { x: 0.0, y: 0.0 }));
            self.last_move_to_index = Some(self.verts.len() - 1);
        }
    }

    /// Adds beginning of contour at Point{x, y}
    pub fn move_to(mut self, x: f32, y: f32) -> Self {
        self.verts.push(PathVerb::MoveTo(Point { x, y }));
        self.last_move_to_index = Some(self.verts.len() - 1);
        self
    }

    /// Adds beginning of contour at Point{x, y}
    pub fn move_to_point(mut self, point: Point) -> Self {
        self.verts.push(PathVerb::MoveTo(point));
        self.last_move_to_index = Some(self.verts.len() - 1);
        self
    }

    /// Adds line from last point to Point{x, y}.
    /// If Path is empty, or last verb is PathVerb::Close, last point is set to (0, 0) before adding line.
    pub fn line_to(mut self, x: f32, y: f32) -> Self {
        self.inject_move_to_if_needed();

        self.verts.push(PathVerb::LineTo(Point { x, y }));
        self
    }

    /// Adds line from last point to Point{x, y}.
    /// If Path is empty, or last verb is PathVerb::Close, last point is set to (0, 0) before adding line.
    pub fn line_to_point(mut self, point: Point) -> Self {
        self.inject_move_to_if_needed();

        self.verts.push(PathVerb::LineTo(point));
        self
    }

    /// Appends PathVerb::Close to Path.
    /// A closed contour connects the first and last Point with line, forming a continuous loop.
    pub fn close(mut self) -> Self {
        self.verts.push(PathVerb::Close);
        self.last_move_to_index = None;

        self
    }
}
