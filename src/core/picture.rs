use nalgebra::Matrix4;

use crate::render::{
    fragment::{ClipMaskFragment, SolidColorFragment},
    raster::{PathFill, PathStroke},
    PathCliper, PathRenderer, Raster, Renderer,
};

use super::{state::State, Paint, Path, RRect, Rect, Style};

/// Defines the type of operation performed by a clip operation.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum ClipOp {
    /// The clip area is the intersection of the current clip area and the specified path.
    #[default]
    Intersect,
    /// The clip area is the difference of the current clip area and the specified path.
    Difference,
}

pub(crate) enum DrawCommand {
    DrawPath(Path, Paint),
    ClipPath(Path, ClipOp),
}

pub(crate) struct Draw {
    pub(crate) depth: u32,
    pub(crate) command: DrawCommand,
    pub(crate) transform: Matrix4<f32>,
}

impl Draw {
    pub(crate) fn gen_render(
        &self,
        vw: f32,
        vh: f32,
        target_format: wgpu::TextureFormat,
        anti_alias: bool,
        depth_offset: u32,
    ) -> Box<dyn Renderer> {
        match &self.command {
            DrawCommand::DrawPath(path, paint) => {
                let raster: Box<dyn Raster> = match paint.style {
                    Style::Fill => Box::new(PathFill::new(path.clone(), self.transform.clone())),
                    Style::Stroke(stroke) => Box::new(PathStroke::new(
                        path.clone(),
                        self.transform.clone(),
                        stroke.width,
                        stroke.miter_limit,
                        stroke.cap,
                        stroke.join,
                    )),
                };

                Box::new(PathRenderer::new(
                    target_format,
                    anti_alias,
                    raster,
                    Box::new(SolidColorFragment::new(
                        paint.color,
                        vw,
                        vh,
                        self.transform.clone(),
                    )),
                    (self.depth + depth_offset) as f32,
                ))
            }
            DrawCommand::ClipPath(path, op) => {
                let raster = PathFill::new(path.clone(), self.transform.clone());
                let fragment = ClipMaskFragment::new(vw, vh, self.transform.clone());

                Box::new(PathCliper::new(
                    target_format,
                    anti_alias,
                    raster,
                    fragment,
                    *op,
                    (self.depth + depth_offset) as f32,
                ))
            }
        }
    }
}

/// Picture holds drawing commands. The command stream can be played back to a Surface.
/// A picture can be played back multiple times.
pub struct Picture {
    pub(crate) draws: Vec<Draw>,
}

/// Recorder drawing commands and can generate a Picture.
pub struct PictureRecorder {
    pub(crate) state: State,
    pub(crate) draws: Vec<Draw>,
    pub(crate) current_depth: u32,
}

impl PictureRecorder {
    pub fn new() -> Self {
        Self {
            state: State::new(),
            draws: Vec::new(),
            current_depth: 0,
        }
    }

    /// Draws path with current clip and transform.
    ///
    /// # Arguments
    ///
    /// * `path` the path to draw
    /// * `paint` the paint controls the styling when drawing the path
    pub fn draw_path(&mut self, path: Path, paint: Paint) {
        self.current_depth += 1;
        self.draws.push(Draw {
            depth: self.current_depth,
            command: DrawCommand::DrawPath(path, paint),
            transform: self.state.current_transform(),
        });
    }

    /// Draws rect with current clip and transform.
    ///
    /// # Arguments
    ///
    /// * `rect` the rect to draw
    /// * `paint` the paint controls the styling when drawing the rect
    pub fn draw_rect(&mut self, rect: &Rect, paint: Paint) {
        self.draw_path(Path::new().add_rect(rect), paint);
    }

    /// Draws oval with current clip and transform.
    ///
    /// # Arguments
    ///
    /// * `rect` the RoundRect to draw
    /// * `paint` the paint controls the styling when drawing the oval
    pub fn draw_rrect(&mut self, rect: &RRect, paint: Paint) {
        self.draw_path(Path::new().add_rrect(rect), paint);
    }

    /// Draws oval with current clip and transform.
    ///
    /// # Arguments
    ///
    /// * `rect` the bounds of ellipse to draw
    /// * `paint` the paint controls the styling when drawing the oval
    pub fn draw_oval(&mut self, rect: &Rect, paint: Paint) {
        self.draw_path(Path::new().add_oval(rect), paint);
    }

    /// Draws circle with current clip and transform.
    ///
    /// # Arguments
    ///
    /// * `cx` the x coordinate of the center of the circle
    /// * `cy` the y coordinate of the center of the circle
    /// * `radius` the radius of the circle
    pub fn draw_circle(&mut self, cx: f32, cy: f32, radius: f32, paint: Paint) {
        if radius <= 0.0 {
            return;
        }

        let oval = Rect::from_xywh(cx - radius, cy - radius, radius * 2.0, radius * 2.0);
        self.draw_oval(&oval, paint);
    }

    /// Clips the current context with the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` the path to clip
    /// * `op` the type of operation performed by the clip
    pub fn clip_path(&mut self, path: Path, op: ClipOp) {
        self.draws.push(Draw {
            depth: 0,
            command: DrawCommand::ClipPath(path, op),
            transform: self.state.current_transform(),
        });

        let last_index = self.draws.len() - 1;

        self.state.save_clip(last_index);
    }

    /// Clips the current context with the specified rect.
    ///
    /// # Arguments
    ///
    /// * `rect` the rect to clip
    /// * `op` the type of operation performed by the clip
    pub fn clip_rect(&mut self, rect: &Rect, op: ClipOp) {
        self.clip_path(Path::new().add_rect(rect), op);
    }

    /// Save current transform matrix and clip state
    pub fn save(&mut self) {
        self.state.save();
    }

    /// Restore the transform matrix and clip to the last saved state
    pub fn restore(&mut self) {
        let clip_state = self.state.restore();

        if clip_state.is_none() {
            return;
        }

        let clip_state = clip_state.unwrap();

        for i in clip_state.clip_op.iter().rev() {
            self.current_depth += 1;
            self.draws[*i].depth = self.current_depth;
        }
    }

    /// Translates transform matrix by dx alone the x-axis and dy along the y-axis
    ///
    /// # Arguments
    ///
    /// * `dx` distance to translate on x-axis
    /// * `dy` distance to translate on y-axis
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.state.translate(dx, dy);
    }

    /// Rotate transform matrix by degree at point (px, py)
    ///
    /// # Arguments
    ///
    /// * `degree` degree to rotate on z-axis
    /// * `px` x position of rotation center
    /// * `py` y position of rotation center
    pub fn rotate_at_xy(&mut self, degree: f32, px: f32, py: f32) {
        self.state.rotate_at(degree, px, py);
    }

    /// Rotate transform matrix by degree at point (0.0, 0.0)
    ///
    /// # Arguments
    ///
    /// * `degree` degree to rotate on z-axis
    pub fn rotate(&mut self, degree: f32) {
        self.state.rotate(degree);
    }

    /// Scale transform matrix
    ///
    /// # Arguments
    ///
    /// * `sx` scale at x-axis must be positive
    /// * `sy` scale at y-axis must be positive
    pub fn scale(&mut self, sx: f32, sy: f32) {
        if sx <= 0.0 || sy <= 0.0 {
            // scale must be positive value
            return;
        }

        self.state.scale(sx, sy);
    }

    /// Finish record and generate a Picture instance with recorded drawing commands
    pub fn finish_record(mut self) -> Picture {
        loop {
            let clip_state = self.state.pop_clip_stack();

            if clip_state.is_none() {
                break;
            }

            let clip_state = clip_state.unwrap();

            for i in clip_state.clip_op.iter().rev() {
                self.current_depth += 1;
                self.draws[*i].depth = self.current_depth;
            }
        }

        Picture { draws: self.draws }
    }
}
