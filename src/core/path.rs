use super::{
    geometry::{ConicCoeff, CubicCoeff, QuadCoeff, FLOAT_ROOT2_OVER2},
    Point, RRect, Rect,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathVerb {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    ConicTo(Point, Point, f32),
    CubicTo(Point, Point, Point),
    Close,
}

/// The fill type of a path.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PathFillType {
    /// Specifies that "inside" is computed by a non-zero sum of signed edge crossings
    #[default]
    Winding,
    /// Specifies that "inside" is computed by an odd number of edge crossings
    EvenOdd,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PathDirection {
    /// The path is drawn in the clockwise direction.
    #[default]
    Clockwise,
    /// The path is drawn in the counter-clockwise direction.
    CounterClockwise,
}

struct PointIterator {
    pts: Vec<Point>,
    current: usize,
    advance: usize,
}

impl PointIterator {
    fn new_direction_start(pts: Vec<Point>, direction: PathDirection, start: usize) -> Self {
        let count = pts.len();

        Self {
            pts,
            current: (start % count),
            advance: if direction == PathDirection::Clockwise {
                1
            } else {
                count - 1
            },
        }
    }

    fn from_rect_dir_start(rect: &Rect, direction: PathDirection, start: usize) -> Self {
        let mut pts = Vec::new();
        pts.push(Point {
            x: rect.left,
            y: rect.top,
        });
        pts.push(Point {
            x: rect.right,
            y: rect.top,
        });
        pts.push(Point {
            x: rect.right,
            y: rect.bottom,
        });
        pts.push(Point {
            x: rect.left,
            y: rect.bottom,
        });

        Self::new_direction_start(pts, direction, start)
    }

    fn from_oval_dir_start(rect: &Rect, direction: PathDirection, start: usize) -> Self {
        let mut pts = Vec::new();
        let cx = rect.center().x;
        let cy = rect.center().y;

        pts.push(Point { x: cx, y: rect.top });
        pts.push(Point {
            x: rect.right,
            y: cy,
        });
        pts.push(Point {
            x: cx,
            y: rect.bottom,
        });
        pts.push(Point {
            x: rect.left,
            y: cy,
        });

        Self::new_direction_start(pts, direction, start)
    }

    pub fn from_rrect_dir_start(rrect: &RRect, direction: PathDirection, start: usize) -> Self {
        let bounds = rrect.bounds();
        let l = bounds.left;
        let t = bounds.top;
        let r = bounds.right;
        let b = bounds.bottom;

        let mut pts = Vec::new();

        pts.push(Point::from(l + rrect.radii[0].x, t));
        pts.push(Point::from(r - rrect.radii[1].x, t));
        pts.push(Point::from(r, t + rrect.radii[1].y));
        pts.push(Point::from(r, b - rrect.radii[2].y));
        pts.push(Point::from(r - rrect.radii[2].x, b));
        pts.push(Point::from(l + rrect.radii[3].x, b));
        pts.push(Point::from(l, b - rrect.radii[3].y));
        pts.push(Point::from(l, t + rrect.radii[0].y));

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
    /// Create a new empty path instance. With default fill type Winding.
    pub fn new() -> Self {
        Self {
            verts: Vec::new(),
            last_move_to_index: None,
            fill_type: PathFillType::Winding,
        }
    }

    /// Create a new empty path instance.
    ///
    /// # Arguments
    ///
    /// * `fill_type` the fill type of the path
    pub fn with_fill_type(fill_type: PathFillType) -> Self {
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

    /// Adds quad from last point towards (x1, y1), to (x2, y2).
    /// If Path is empty or last verb is PathVerb::Close, last point is set to (0, 0) before adding quad.
    pub fn quad_to(mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        self.inject_move_to_if_needed();

        self.verts.push(PathVerb::QuadTo(
            Point { x: x1, y: y1 },
            Point { x: x2, y: y2 },
        ));
        self
    }

    /// Adds quad from last point towards point `ctr`, to point `end`.
    /// If Path is empty or last verb is PathVerb::Close, last point is set to (0, 0) before adding quad.
    pub fn quad_to_point(mut self, ctr: Point, end: Point) -> Self {
        self.inject_move_to_if_needed();

        self.verts.push(PathVerb::QuadTo(ctr, end));
        self
    }

    /// Adds conic from last point towards (x1, y1), to (x2, y2), with weight w.
    /// If Path is empty or last verb is PathVerb::Close, last point is set to (0, 0) before adding conic.
    pub fn conic_to(mut self, x1: f32, y1: f32, x2: f32, y2: f32, w: f32) -> Self {
        self.inject_move_to_if_needed();

        self.verts.push(PathVerb::ConicTo(
            Point { x: x1, y: y1 },
            Point { x: x2, y: y2 },
            w,
        ));
        self
    }

    /// Adds conic from last point towards `ctr`, to `end`, with weight `w`.
    /// If Path is empty or last verb is PathVerb::Close, last point is set to (0, 0) before adding conic.
    pub fn conic_to_point(mut self, ctr: Point, end: Point, w: f32) -> Self {
        self.inject_move_to_if_needed();

        self.verts.push(PathVerb::ConicTo(ctr, end, w));
        self
    }

    /// Adds cubic from last point towards (x1, y1), then towards (x2, y2), ending at (x3, y3)
    /// If Path is empty or last verb is PathVerb::Close, last point is set to (0, 0) before adding cubic.
    pub fn cubic_to(mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> Self {
        self.inject_move_to_if_needed();

        self.verts.push(PathVerb::CubicTo(
            Point { x: x1, y: y1 },
            Point { x: x2, y: y2 },
            Point { x: x3, y: y3 },
        ));
        self
    }

    /// Adds cubic from last point towards `ctr1`, then towards `ctr2`, ending at `end`
    /// If Path is empty or last verb is PathVerb::Close, last point is set to (0, 0) before adding cubic.
    pub fn cubic_to_point(mut self, ctr1: Point, ctr2: Point, end: Point) -> Self {
        self.inject_move_to_if_needed();

        self.verts.push(PathVerb::CubicTo(ctr1, ctr2, end));
        self
    }

    /// Adds a new contour to the path, defined by the Rect. Start at index 0 and with default direction Clockwise.
    /// The verbs added to the path will be:
    /// MoveTo, LineTo, LineTo, LineTo, Close
    pub fn add_rect(self, rect: &Rect) -> Self {
        self.add_rect_dir_start(rect, Default::default(), 0)
    }

    /// Adds a new contour to the path, defined by the Rect.
    /// The verbs added to the path will be:
    /// MoveTo, LineTo, LineTo, LineTo, Close
    ///
    /// # Arguments
    ///
    /// * `rect` the bounds of rectangle added
    /// * `dir` the direction to wind rectangle
    /// * `start` the initial point of rectangle
    pub fn add_rect_dir_start(self, rect: &Rect, dir: PathDirection, start: usize) -> Self {
        if rect.is_empty() {
            return self;
        }

        let mut iter = PointIterator::from_rect_dir_start(rect, dir, start);

        self.move_to_point(iter.next())
            .line_to_point(iter.next())
            .line_to_point(iter.next())
            .line_to_point(iter.next())
            .close()
    }

    /// Adds oval to Path with default PathDirection::Clockwise and start at index 1.
    /// The verbs added to the path will be:
    /// MoveTo, CubicTo, CubicTo, CubicTo, CubicTo, Close.
    ///
    /// # Arguments
    ///
    /// * `oval` the bounds of ellipse added
    pub fn add_oval(self, oval: &Rect) -> Self {
        self.add_oval_dir_start(oval, Default::default(), 1)
    }

    /// Adds RRect to Path.
    ///
    /// # Arguments
    ///
    /// * `oval` the bounds of RRect added
    /// * `dir` the direction to wind ellipse
    /// * `start` the initial point of ellipse
    pub fn add_oval_dir_start(self, oval: &Rect, dir: PathDirection, start: usize) -> Self {
        if oval.is_empty() {
            return self;
        }

        let mut oval_iter = PointIterator::from_oval_dir_start(oval, dir, start);
        let mut rect_iter = PointIterator::from_rect_dir_start(oval, dir, start);

        let weight = FLOAT_ROOT2_OVER2;

        return self
            .move_to_point(oval_iter.current())
            .conic_to_point(rect_iter.next(), oval_iter.next(), weight)
            .conic_to_point(rect_iter.next(), oval_iter.next(), weight)
            .conic_to_point(rect_iter.next(), oval_iter.next(), weight)
            .conic_to_point(rect_iter.next(), oval_iter.next(), weight)
            .close();
    }

    /// Adds RRect to Path. With given direction and start point.
    ///
    /// # Arguments
    ///
    /// * `rrect` the round rect to add which defines the bounds and radii
    /// * `dir` the direction to wind ellipse
    /// * `start` the initial point of ellipse
    pub fn add_rrect_dir_start(mut self, rrect: &RRect, dir: PathDirection, start: usize) -> Self {
        if rrect.is_empty() {
            return self;
        }

        if rrect.is_rect() {
            return self.add_rect_dir_start(&rrect.bounds(), dir, (start + 1) / 2);
        }

        if rrect.is_oval() {
            return self.add_oval_dir_start(&rrect.bounds(), dir, start / 2);
        }

        let start_with_conic = ((start & 1) == 1) == (dir == PathDirection::Clockwise);
        let weight = FLOAT_ROOT2_OVER2;

        let mut rrect_iter = PointIterator::from_rrect_dir_start(rrect, dir, start);
        let rect_start_index = start / 2
            + if dir == PathDirection::Clockwise {
                0
            } else {
                1
            };
        let mut rect_iter =
            PointIterator::from_rect_dir_start(&rrect.bounds(), dir, rect_start_index);

        self = self.move_to_point(rrect_iter.current());

        if start_with_conic {
            for _ in 0..3 {
                self = self
                    .conic_to_point(rect_iter.next(), rrect_iter.next(), weight)
                    .line_to_point(rrect_iter.next());
            }

            self = self.conic_to_point(rect_iter.next(), rrect_iter.next(), weight);
        } else {
            for _ in 0..4 {
                self = self.line_to_point(rrect_iter.next()).conic_to_point(
                    rect_iter.next(),
                    rrect_iter.next(),
                    weight,
                );
            }
        }

        self.close()
    }

    /// Adds RRect to Path. With given direction.
    /// The start index is 6 if PathDirection::Clockwise, 7 otherwise.
    /// # Arguments
    ///
    /// * `rrect` the round rect to add which defines the bounds and radii
    /// * `dir` the direction to wind the round rect
    pub fn add_rrect_dir(self, rrect: &RRect, dir: PathDirection) -> Self {
        self.add_rrect_dir_start(
            rrect,
            dir,
            if dir == PathDirection::Clockwise {
                6
            } else {
                7
            },
        )
    }

    /// Adds RRect to Path. With default PathDirection::Clockwise.
    /// The start index is 6 if PathDirection::Clockwise, 7 otherwise.
    /// # Arguments
    ///
    /// * `rrect` the round rect to add which defines the bounds and radii
    pub fn add_rrect(self, rrect: &RRect) -> Self {
        self.add_rrect_dir(rrect, Default::default())
    }

    /// Appends PathVerb::Close to Path.
    /// A closed contour connects the first and last Point with line, forming a continuous loop.
    pub fn close(mut self) -> Self {
        self.verts.push(PathVerb::Close);
        self.last_move_to_index = None;

        self
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
        self.points.push(p);
    }

    pub(crate) fn last_point(&self) -> Option<&Point> {
        self.points.last()
    }
}

pub(crate) struct Polyline {
    pub(crate) contours: Vec<Contour>,
}

pub(crate) struct PolylineBuilder<'a> {
    path: &'a Path,
    verbs: Vec<PathVerb>,
}

const CURVE_STEP: f32 = 32.0;

impl<'a> PolylineBuilder<'a> {
    pub(crate) fn from(path: &'a Path) -> Self {
        Self {
            path: path,
            verbs: Vec::new(),
        }
    }

    /// Simplefy verbs
    ///  Remove PathVerb::Move if it not continue with line_to or other curve verbs
    fn simple_verbs(mut self) -> Self {
        let verb_count = self.path.verts.len();
        let mut prev_is_move = false;
        for (i, e) in self.path.verts.iter().enumerate() {
            match e {
                PathVerb::MoveTo(p) => {
                    if i == verb_count - 1 {
                        continue;
                    }
                    prev_is_move = true;
                    match self.path.verts[i + 1] {
                        PathVerb::Close => continue,
                        PathVerb::MoveTo(_) => continue,
                        _ => {
                            self.verbs.push(PathVerb::MoveTo(p.clone()));
                        }
                    }
                }
                PathVerb::Close => {
                    if prev_is_move {
                        continue;
                    }
                    if self.verbs.is_empty() {
                        continue;
                    }

                    match self.verbs.last().as_ref().unwrap() {
                        PathVerb::Close => continue,
                        PathVerb::MoveTo(_) => {
                            self.verbs.pop();
                            continue;
                        }
                        _ => {}
                    }

                    self.verbs.push(PathVerb::Close);
                }
                _ => {
                    prev_is_move = false;
                    self.verbs.push(e.clone());
                }
            }
        }

        self
    }

    fn create_contours(self) -> Vec<Contour> {
        let mut contours: Vec<Contour> = Vec::new();

        for v in &self.verbs {
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
                    let quad = QuadCoeff::from(
                        contours
                            .last()
                            .expect("Not create contour")
                            .last_point()
                            .expect("Contour not start"),
                        ctr,
                        end,
                    );

                    // TODO: flatten curve dynamic with line count
                    for step in 0..(CURVE_STEP as i32) {
                        let t = (step as f32 + 1.0) / CURVE_STEP;
                        contours.last_mut().unwrap().add_point(quad.eval(t));
                    }
                }
                PathVerb::ConicTo(p2, p3, weight) => {
                    let conic = ConicCoeff::from(
                        contours
                            .last()
                            .expect("Not create contour")
                            .last_point()
                            .expect("Not start contour"),
                        p2,
                        p3,
                        *weight,
                    );

                    // TODO: flatten curve dynamic with line count
                    for step in 0..(CURVE_STEP as i32) {
                        let t = (step as f32 + 1.0) / CURVE_STEP;
                        contours.last_mut().unwrap().add_point(conic.eval(t));
                    }
                }
                PathVerb::CubicTo(p2, p3, p4) => {
                    let cubic = CubicCoeff::from(
                        contours
                            .last()
                            .expect("Not create contour")
                            .last_point()
                            .expect("Not start contour"),
                        p2,
                        p3,
                        p4,
                    );

                    // TODO: flatten curve dynamic with line count
                    for step in 0..(CURVE_STEP as i32) {
                        let t = (step as f32 + 1.0) / CURVE_STEP;
                        contours.last_mut().unwrap().add_point(cubic.eval(t));
                    }
                }
                PathVerb::Close => {
                    contours.last_mut().expect("Not start contour").closed = true;
                }
            }
        }

        return contours;
    }

    pub(crate) fn build(self) -> Polyline {
        let contours = self.simple_verbs().create_contours();

        return Polyline { contours };
    }
}
