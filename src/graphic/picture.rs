use crate::{Angle, ClipOp, Matrix3x3, Paint, Path, Point, Rect};

/// The command describes the rendering operation on the canvas.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Draw {
    /// Draw a path with a paint.
    /// The path will be transformed by the matrix.
    DrawPath(Path, Paint, Matrix3x3, u32),
    /// Draw a rect with a paint.
    /// The rect will be transformed by the matrix.
    DrawRect(Rect, Paint, Matrix3x3, u32),
    /// Clip a path.
    /// The path will be transformed by the matrix.
    /// The clip operation will be applied to the current clip state.
    ClipPath(Path, ClipOp, Matrix3x3, u32),
}

impl Draw {
    pub(crate) fn transform(&self) -> Matrix3x3 {
        match self {
            Draw::DrawPath(_, _, m, _) => m.clone(),
            Draw::DrawRect(_, _, m, _) => m.clone(),
            Draw::ClipPath(_, _, m, _) => m.clone(),
        }
    }

    pub(crate) fn paint(&self) -> Paint {
        match self {
            Draw::DrawPath(_, paint, _, _) => paint.clone(),
            Draw::DrawRect(_, paint, _, _) => paint.clone(),
            Draw::ClipPath(_, _, _, _) => Paint::default(),
        }
    }

    pub(crate) fn depth(&self) -> u32 {
        match self {
            Draw::DrawPath(_, _, _, depth) => *depth,
            Draw::DrawRect(_, _, _, depth) => *depth,
            Draw::ClipPath(_, _, _, depth) => *depth,
        }
    }

    pub(crate) fn set_depth(&mut self, depth: u32) {
        match self {
            Draw::DrawPath(_, _, _, d) => *d = depth,
            Draw::DrawRect(_, _, _, d) => *d = depth,
            Draw::ClipPath(_, _, _, d) => *d = depth,
        }
    }

    pub(crate) fn clip_op(&self) -> Option<ClipOp> {
        match self {
            Draw::ClipPath(_, op, _, _) => Some(*op),
            _ => None,
        }
    }

    pub(crate) fn get_bounds(&self) -> Rect {
        match self {
            Draw::DrawPath(path, _, _, _) => path.get_bounds(),
            Draw::DrawRect(rect, _, _, _) => rect.clone(),
            Draw::ClipPath(path, _, _, _) => path.get_bounds(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct State {
    pub(crate) transform: Matrix3x3,
    pub(crate) clip_op: Vec<usize>,
}

impl Default for State {
    fn default() -> Self {
        State {
            transform: Matrix3x3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            clip_op: vec![],
        }
    }
}

impl State {
    pub(crate) fn translate(&mut self, x: f32, y: f32) {
        self.transform = self.transform.translate(x, y);
    }

    pub(crate) fn scale(&mut self, x: f32, y: f32) {
        self.transform = self.transform.scale(x, y);
    }

    pub(crate) fn rotate<T: Angle>(&mut self, value: T) {
        self.transform = self.transform.rotate(value);
    }

    pub(crate) fn rotate_at<T: Angle>(&mut self, point: Point, value: T) {
        self.transform = self.transform.rotate_at(point, value);
    }

    pub(crate) fn save_clip(&mut self, index: usize) {
        self.clip_op.push(index);
    }
}

/// The picture recorder is used to record the drawing commands.
/// After the drawing commands are recorded, it can be persisted into a [`Picture`]` object.
pub struct PictureRecorder {
    pub(crate) state: Vec<State>,
    pub(crate) draws: Vec<Draw>,
    pub(crate) current_depth: u32,
}

impl PictureRecorder {
    pub fn new() -> Self {
        PictureRecorder {
            state: vec![State::default()],
            draws: vec![],
            current_depth: 0,
        }
    }

    /// Draw a path with a paint.
    /// The path will be transformed by the current matrix. And clipped by the current clip state.
    ///
    /// # Arguments
    /// * `path` - The path to draw.
    /// * `paint` - The paint to draw the path.
    pub fn draw_path(&mut self, path: Path, paint: &Paint) {
        self.current_depth += 1;
        self.draws.push(Draw::DrawPath(
            path,
            paint.clone(),
            self.current_transform(),
            self.current_depth,
        ));
    }

    /// Draw a rect with a paint.
    /// The rect will be transformed by the current matrix. And clipped by the current clip state.
    ///
    /// # Arguments
    /// * `rect` - The rect to draw.
    /// * `paint` - The paint to draw the rect.
    pub fn draw_rect(&mut self, rect: Rect, paint: &Paint) {
        self.current_depth += 1;
        self.draws.push(Draw::DrawRect(
            rect,
            paint.clone(),
            self.current_transform(),
            self.current_depth,
        ));
    }

    /// Draw a circle with a paint.
    /// The circle will be transformed by the current matrix. And clipped by the current clip state.
    ///
    /// # Arguments
    /// * `center` - The center of the circle.
    /// * `radius` - The radius of the circle.
    /// * `paint` - The paint to draw the circle.
    pub fn draw_circle<T: Into<Point>>(&mut self, center: T, radius: f32, paint: &Paint) {
        let path = Path::new().add_circle(center.into(), radius);
        self.draw_path(path, paint);
    }

    /// Draw a picture content to the current canvas.
    /// The picture content will be transformed by the current matrix. And clipped by the current clip state.
    ///
    /// # Arguments
    /// * `picture` - The picture to draw.
    pub fn draw_picture(&mut self, picture: &Picture) {
        let current_transform = self.current_transform();

        for draw in &picture.draws {
            match draw {
                Draw::DrawPath(path, paint, m, depth) => {
                    self.draws.push(Draw::DrawPath(
                        path.clone(),
                        paint.clone(),
                        current_transform * m.clone(),
                        self.current_depth + depth,
                    ));
                }
                Draw::DrawRect(rect, paint, m, depth) => {
                    self.draws.push(Draw::DrawRect(
                        rect.clone(),
                        paint.clone(),
                        current_transform * m.clone(),
                        self.current_depth + depth,
                    ));
                }
                Draw::ClipPath(path, op, m, depth) => {
                    self.draws.push(Draw::ClipPath(
                        path.clone(),
                        *op,
                        current_transform * m.clone(),
                        self.current_depth + depth,
                    ));
                }
            }
        }

        self.current_depth += picture.draws.len() as u32;
    }

    /// Clip the current canvas.
    /// The clip operation will be applied to the current clip state.
    ///
    /// # Arguments
    /// * `path` - The path to clip.
    /// * `op` - The clip operation.
    pub fn clip<T: Into<Path>>(&mut self, path: T, op: ClipOp) {
        self.draws.push(Draw::ClipPath(
            path.into(),
            op,
            self.current_transform(),
            0, // The value will be set when restore or finish recording
        ));

        let index = self.draws.len() - 1;

        self.state
            .last_mut()
            .expect("The state stack is empty")
            .save_clip(index);
    }

    /// Save the current state of the canvas.
    /// The state will be restored when the canvas is restored.
    pub fn save(&mut self) {
        let state = self.state.last().expect("The state stack is empty").clone();
        self.state.push(state);
    }

    /// Restore the previous state of the canvas.
    pub fn restore(&mut self) {
        if self.state.len() == 1 {
            return;
        }

        let state = self.state.pop().expect("The state stack is empty");

        for i in state.clip_op.iter().rev() {
            self.current_depth += 1;
            self.draws[*i].set_depth(self.current_depth);
        }
    }

    /// Translate the current matrix. By x and y.
    ///
    /// # Arguments
    /// * `x` - The offset at x coordinate.
    /// * `y` - The offset at y coordinate.
    pub fn translate(&mut self, x: f32, y: f32) {
        self.state
            .last_mut()
            .expect("The state stack is empty")
            .translate(x, y);
    }

    /// Scale the current matrix. By x and y. The function does not check the scale value.
    /// If the scale value is 0, the rendering result may be unexpected.
    ///
    /// # Arguments
    /// * `x` - The scale at x coordinate.
    /// * `y` - The scale at y coordinate.
    pub fn scale(&mut self, x: f32, y: f32) {
        self.state
            .last_mut()
            .expect("The state stack is empty")
            .scale(x, y);
    }

    /// Rotate the current matrix. By angle.
    ///
    /// # Arguments
    /// * `value` - The angle to rotate. Pass raw f32 value means degree.
    ///
    /// # Example
    /// ```
    /// use mickey::*;
    /// let mut recorder = PictureRecorder::new();
    /// recorder.rotate(45.0.degree()); // rotate in 45 degree
    /// recorder.rotate(Radian(0.3)); // rotate in radians
    /// recorder.rotate(Degree(45.0)); // rotate in degree
    /// ```
    pub fn rotate<T: Angle>(&mut self, value: T) {
        self.state
            .last_mut()
            .expect("The state stack is empty")
            .rotate(value);
    }

    /// Rotate the current matrix at the given point. By angle.
    ///
    /// # Arguments
    /// * `point` - The point to rotate at.
    /// * `value` - The angle to rotate.
    pub fn rotate_at<T: Angle>(&mut self, point: Point, value: T) {
        self.state
            .last_mut()
            .expect("The state stack is empty")
            .rotate_at(point, value);
    }

    /// Finish the recording and return the picture.
    pub fn finish_recorder(mut self) -> Picture {
        while self.state.len() > 1 {
            self.restore();
        }

        let state = self.state.pop().expect("The state stack is empty");
        for i in state.clip_op.iter().rev() {
            self.draws[*i].set_depth(self.current_depth);
        }

        Picture { draws: self.draws }
    }

    pub fn current_transform(&self) -> Matrix3x3 {
        self.state
            .last()
            .expect("The state stack is empty")
            .transform
            .clone()
    }
}

/// The picture is a collection of drawing commands.
/// It can be used to draw a picture on the canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct Picture {
    pub(crate) draws: Vec<Draw>,
}
