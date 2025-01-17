mod buffer;
mod command;
mod content;
mod context;
mod fragment;
mod geometry;
mod pipeline;
mod raster;
mod shader;

pub(crate) use buffer::*;
pub(crate) use command::*;
pub(crate) use content::*;
pub use context::*;
pub(crate) use fragment::*;
pub(crate) use geometry::*;
pub(crate) use pipeline::*;
pub(crate) use shader::*;

/// The raster result. Which indicates if needs to do stencil then fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MeshType {
    /// The raster result can be rendered directly.
    Direct,
    /// Stroke mesh
    Stroke,
    /// The raster result needs to do stencil test then fill. with even-odd winding.
    EvenOdd,
    /// The raster result needs to do stencil test then fill. with non-zero winding.
    NonZero,
}

/// Mesh just collect the vertex and index data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Mesh {
    pub(crate) vertex_buffer: StageBufferView,
    pub(crate) index_buffer: StageBufferView,
    pub(crate) draw_count: u32,
    pub(crate) mesh_type: MeshType,
}

/// The renderer responsible for the rendering the geometry shape.
/// First it handled to convert the shape into vertex and index.
/// Then it handled to pick the pipeline for the shape.
/// Finally it generated the Command to do the actual rendering.
pub(crate) struct Renderer<G: Geometry + ShaderGenerator, F: Fragment + ShaderGenerator> {
    format: wgpu::TextureFormat,
    sample_count: u32,
    geometry: G,
    fragment: F,
}

impl<G: Geometry + ShaderGenerator, F: Fragment + ShaderGenerator> Renderer<G, F> {
    pub(crate) fn new(
        format: wgpu::TextureFormat,
        sample_count: u32,
        geometry: G,
        fragment: F,
    ) -> Self {
        Renderer {
            format,
            sample_count,
            geometry,
            fragment,
        }
    }

    pub(crate) fn render(
        &self,
        mesh: Mesh,
        buffer: &mut StageBuffer,
        context: &mut RenderContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Command {
        let (attributes, attr_stride) = self.geometry.get_vertex_attributes();

        let pipeline_query = PipelineQuery {
            label: format!(
                "{}-{}",
                self.geometry.shader_name(),
                self.fragment.shader_name()
            ),
            attributes,
            attr_stride,
            vs_generator: &self.geometry,
            fs_generator: &self.fragment,
            layout_generator: self,
        };

        let pipeline = context.query_pipeline(device, &pipeline_query);

        let key = PipelineKey {
            label: format!(
                "{}-{}-{}",
                self.geometry.shader_name(),
                self.fragment.shader_name(),
                self.fragment.stencil_state_name(),
            ),
            format: self.format,
            sample_count: self.sample_count,
            ds: self.fragment.depth_stencil_state(),
            blend: self.fragment.blend_state(),
        };
        let render_pipeline = pipeline
            .borrow_mut()
            .get_varient(&key, device)
            .expect("Pipeline created failed");

        let mut groups = vec![self.geometry.gen_group(buffer)];

        match self.fragment.gen_group(buffer, device, queue) {
            Some(fg) => groups.push(fg),
            None => (),
        }

        Command {
            pipeline: render_pipeline,
            vertex_buffer: mesh.vertex_buffer,
            index_buffer: mesh.index_buffer,
            groups,
            draw_count: mesh.draw_count,
        }
    }
}

impl<G: Geometry + ShaderGenerator, F: Fragment + ShaderGenerator> LayoutGenerator
    for Renderer<G, F>
{
    fn generate(&self, device: &wgpu::Device) -> wgpu::PipelineLayout {
        let vertex_group_entry = self.geometry.get_group_entry();
        let fragment_group_entry = self.fragment.get_group_entry();

        let vertex_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(format!("{}_group", self.geometry.shader_name()).as_str()),
                entries: &vertex_group_entry,
            });
        let fragment_group_layout = if fragment_group_entry.is_empty() {
            None
        } else {
            Some(
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(format!("{}_group", self.fragment.shader_name()).as_str()),
                    entries: &fragment_group_entry,
                }),
            )
        };

        let mut layouts = vec![&vertex_group_layout];
        if fragment_group_layout.is_some() {
            layouts.push(fragment_group_layout.as_ref().unwrap());
        }

        return device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        });
    }
}
