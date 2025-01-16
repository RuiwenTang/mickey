use nalgebra::Matrix4;

use crate::{ColorFragment, Draw};

use super::{
    ColorGeometry, Command, Fragment, Geometry, LayoutGenerator, NoneStencil, PipelineKey,
    PipelineQuery, PositionChunk, RenderContext, Renderer, ShaderGenerator, StageBuffer,
    StageBufferView,
};

fn mvp_to_buffer(mvp: &Matrix4<f32>) -> [f32; 16] {
    mvp.as_slice()
        .iter()
        .map(|v| *v)
        .collect::<Vec<f32>>()
        .try_into()
        .unwrap()
}

pub(crate) fn create_render_direct(draw: &Draw, mvp: &Matrix4<f32>) -> Box<dyn Renderer> {
    match draw {
        Draw::DrawRect(_, paint, transform) => {
            let geometry = ColorGeometry::new(PositionChunk::new(
                mvp_to_buffer(mvp),
                transform.clone().into(),
                [0.5, 0.0, 0.0, 0.0],
            ));

            let fragment = ColorFragment::<NoneStencil>::new(paint.color);

            return Box::new(ShapeRender::new(geometry, fragment));
        }
        _ => {
            panic!("Unsupported draw");
        }
    }
}

struct ShapeRender<G: Geometry + ShaderGenerator, F: Fragment + ShaderGenerator> {
    geometry: G,
    fragment: F,
}

impl<G: Geometry + ShaderGenerator, F: Fragment + ShaderGenerator> ShapeRender<G, F> {
    fn new(geometry: G, fragment: F) -> Self {
        ShapeRender { geometry, fragment }
    }
}

impl<G: Geometry + ShaderGenerator, F: Fragment + ShaderGenerator> LayoutGenerator
    for ShapeRender<G, F>
{
    fn generate(&self, device: &wgpu::Device) -> wgpu::PipelineLayout {
        let vertex_group_entry = self.geometry.get_group_entry();
        let fragment_group_entry = self.fragment.get_group_entry();

        let vertex_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(format!("{}_group", self.geometry.shader_name()).as_str()),
                entries: &vertex_group_entry,
            });
        let fragment_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(format!("{}_group", self.fragment.shader_name()).as_str()),
                entries: &fragment_group_entry,
            });

        return device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&vertex_group_layout, &fragment_group_layout],
            push_constant_ranges: &[],
        });
    }
}

impl<G: Geometry + ShaderGenerator, F: Fragment + ShaderGenerator> Renderer for ShapeRender<G, F> {
    fn render(
        &self,
        raster_result: (StageBufferView, StageBufferView, u32),
        buffer: &mut StageBuffer,
        context: &mut RenderContext,
        format: wgpu::TextureFormat,
        sample_count: u32,
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
            format,
            sample_count,
            ds: self.fragment.depth_stencil_state(),
            blend: self.fragment.blend_state(),
        };
        let render_pipeline = pipeline
            .borrow_mut()
            .get_varient(&key, device)
            .expect("Pipeline created failed");

        Command {
            pipeline: render_pipeline,
            vertex_buffer: raster_result.0,
            index_buffer: raster_result.1,
            groups: vec![
                self.geometry.gen_group(buffer),
                self.fragment.gen_group(buffer, device, queue),
            ],
            draw_count: raster_result.2,
        }
    }
}
