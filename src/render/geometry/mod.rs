mod image_geometry;
mod shape_geometry;

pub(crate) use image_geometry::*;
pub(crate) use shape_geometry::*;

use crate::{Group, StageBuffer};

/// The geometry handled the shape raster.
/// Also provide the vertex buffer layout information.
pub(crate) trait Geometry {
    /// Get the vertex buffer layout information.
    fn get_vertex_attributes(&self) -> (Vec<wgpu::VertexAttribute>, wgpu::BufferAddress);

    /// Get the bind group entry. for create pipeline layout
    /// All vertex shader use group 0.
    fn get_group_entry(&self) -> Vec<wgpu::BindGroupLayoutEntry>;

    /// Generate the group for the geometry.
    fn gen_group(&self, buffer: &mut StageBuffer) -> Group;
}
