use nalgebra::{Matrix4, Vector4};

use crate::render::{fragment::SolidColorFragment, raster::PathFillRaster, PathRenderer, Renderer};

use super::{state::State, Paint, Path};

pub(crate) enum DrawCommand {
    DrawPath(Path, Paint),
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
            DrawCommand::DrawPath(path, paint) => Box::new(PathRenderer::new(
                target_format,
                anti_alias,
                PathFillRaster::new(path.clone()),
                SolidColorFragment::new(paint.color, vw, vh, self.transform.clone()),
                (self.depth + depth_offset) as f32,
            )),
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

    pub fn draw_path(&mut self, path: Path, paint: Paint) {
        self.current_depth += 1;
        self.draws.push(Draw {
            depth: self.current_depth,
            command: DrawCommand::DrawPath(path, paint),
            transform: self.state.current_transform(),
        });
    }

    /// Save current transform matrix and clip state
    pub fn save(&mut self) {
        self.state.save();
    }

    /// Restore the transform matrix and clip to the last saved state
    pub fn restore(&mut self) {
        self.state.restore();
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

    /// Finish record and generate a Picture instance with recorded drawing commands
    pub fn finish_record(self) -> Picture {
        Picture { draws: self.draws }
    }
}
