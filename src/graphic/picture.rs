use crate::{Angle, Matrix3x3, Paint, Path, Point, Rect};

/// The command describes the rendering operation on the canvas.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Draw {
    /// Draw a path with a paint.
    /// The path will be transformed by the matrix.
    DrawPath(Path, Paint, Matrix3x3),
    /// Draw a rect with a paint.
    /// The rect will be transformed by the matrix.
    DrawRect(Rect, Paint, Matrix3x3),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct State {
    pub(crate) transform: Matrix3x3,
}

impl Default for State {
    fn default() -> Self {
        State {
            transform: Matrix3x3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
        }
    }
}

impl State {
    pub(crate) fn translate(&mut self, x: f32, y: f32) {
        self.transform.translate(x, y);
    }

    pub(crate) fn scale(&mut self, x: f32, y: f32) {
        self.transform.scale(x, y);
    }

    pub(crate) fn rotate<T: Angle>(&mut self, value: T) {
        self.transform.rotate(value);
    }

    pub(crate) fn rotate_at<T: Angle>(&mut self, point: Point, value: T) {
        self.transform.rotate_at(point, value);
    }
}

/// The picture recorder is used to record the drawing commands.
/// After the drawing commands are recorded, it can be persisted into a Picture object.
pub struct PictureRecorder {
    pub(crate) state: Vec<State>,
    pub(crate) draws: Vec<Draw>,
}

impl PictureRecorder {
    pub fn new() -> Self {
        PictureRecorder {
            state: vec![State::default()],
            draws: vec![],
        }
    }

    /// Draw a path with a paint.
    /// The path will be transformed by the current matrix. And clipped by the current clip state.
    ///
    /// # Arguments
    /// * `path` - The path to draw.
    /// * `paint` - The paint to draw the path.
    pub fn draw_path(&mut self, path: Path, paint: Paint) {
        self.draws
            .push(Draw::DrawPath(path, paint, self.current_transform()));
    }

    /// Draw a rect with a paint.
    /// The rect will be transformed by the current matrix. And clipped by the current clip state.
    ///
    /// # Arguments
    /// * `rect` - The rect to draw.
    /// * `paint` - The paint to draw the rect.
    pub fn draw_rect(&mut self, rect: Rect, paint: Paint) {
        self.draws
            .push(Draw::DrawRect(rect, paint, self.current_transform()));
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
                Draw::DrawPath(path, paint, m) => {
                    self.draws.push(Draw::DrawPath(
                        path.clone(),
                        paint.clone(),
                        current_transform * m.clone(),
                    ));
                }
                Draw::DrawRect(rect, paint, m) => {
                    self.draws.push(Draw::DrawRect(
                        rect.clone(),
                        paint.clone(),
                        current_transform * m.clone(),
                    ));
                }
            }
        }
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

        self.state.pop();
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
    /// recorder.rotate(45.0); // rotate in 45 degree
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
    pub fn finish_recorder(self) -> Picture {
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
