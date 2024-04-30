use nalgebra::{Matrix4, Vector4};

use crate::render::{fragment::SolidColorFragment, raster::PathFillRaster, PathRenderer, Renderer};

use super::Path;

pub(crate) enum DrawCommand {
    DrawPath(Path),
}

pub(crate) struct Draw {
    pub(crate) depth: u32,
    pub(crate) command: DrawCommand,
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
            DrawCommand::DrawPath(path) => Box::new(PathRenderer::new(
                target_format,
                anti_alias,
                PathFillRaster::new(path.clone()),
                SolidColorFragment::new(
                    Vector4::new(1.0, 0.0, 0.0, 0.5),
                    vw,
                    vh,
                    Matrix4::identity(),
                ),
                (self.depth + depth_offset) as f32,
            )),
        }
    }
}

/// Picture records drawing commands. The command stream can be played back to a Surface.
/// A picture can be played back multiple times.
pub struct Picture {
    pub(crate) draws: Vec<Draw>,
    pub(crate) current_depth: u32,
}

impl Picture {
    pub fn new() -> Self {
        Self {
            draws: Vec::new(),
            current_depth: 0,
        }
    }

    pub fn draw_path(&mut self, path: Path) {
        self.current_depth += 1;
        self.draws.push(Draw {
            depth: self.current_depth,
            command: DrawCommand::DrawPath(path),
        });
    }

    pub fn clear(&mut self) {
        self.draws.clear();
        self.current_depth = 0;
    }
}
