mod common;

use bytemuck::{Pod, Zeroable};
use common::App;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

struct Pipeline {
    groups: Vec<wgpu::BindGroupLayout>,
    _layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
}

struct HelloTriangle {
    pipeline: Option<Pipeline>,
}

impl HelloTriangle {
    pub fn new() -> Self {
        HelloTriangle { pipeline: None }
    }

    fn create_pipeline(&self, format: wgpu::TextureFormat, device: &wgpu::Device) -> Pipeline {
        let groups = vec![
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("transform"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
        ];

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Hello world"),
            bind_group_layouts: &[&groups[0]],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        return Pipeline {
            groups: groups,
            _layout: layout,
            pipeline,
        };
    }

    fn create_buffer(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (wgpu::Buffer, wgpu::BufferAddress) {
        let vertex_data = [
            Vertex {
                position: [0.0, 0.5],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5],
                color: [0.0, 0.0, 1.0],
            },
        ];

        let matrix = nalgebra::Matrix4::<f32>::new_rotation(nalgebra::Vector3::new(0.0, 0.0, 0.2));
        let alighment = device.limits().min_uniform_buffer_offset_alignment as usize;
        let mut size = std::mem::size_of::<Vertex>() * vertex_data.len();
        if size < alighment {
            size = alighment;
        } else {
            let offset = alighment - (size % alighment);

            size += offset;
        }

        let matrix_offset = size as u64;

        size += std::mem::size_of::<nalgebra::Matrix4<f32>>();

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_stage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        let matrix_stage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(matrix.as_slice()),
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.copy_buffer_to_buffer(
            &vertex_stage_buffer,
            0,
            &buffer,
            0,
            (std::mem::size_of::<Vertex>() * vertex_data.len()) as wgpu::BufferAddress,
        );

        encoder.copy_buffer_to_buffer(
            &matrix_stage_buffer,
            0,
            &buffer,
            matrix_offset as wgpu::BufferAddress,
            std::mem::size_of::<nalgebra::Matrix4<f32>>() as wgpu::BufferAddress,
        );

        queue.submit(Some(encoder.finish()));

        return (buffer, matrix_offset);
    }

    fn create_bind_group(
        &self,
        matrix_offset: u64,
        buffer: &wgpu::Buffer,
        device: &wgpu::Device,
    ) -> Vec<wgpu::BindGroup> {
        buffer.as_entire_binding();
        vec![device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.as_ref().unwrap().groups[0],
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer,
                    offset: matrix_offset,
                    size: wgpu::BufferSize::new(
                        std::mem::size_of::<nalgebra::Matrix4<f32>>() as wgpu::BufferAddress
                    ),
                }),
            }],
        })]
    }
}

impl common::Renderer for HelloTriangle {
    fn on_init(
        &mut self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.pipeline = Some(self.create_pipeline(format, device));
    }

    fn on_render(&mut self, surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue) {
        let (buffer, matrix_offset) = self.create_buffer(device, queue);
        let groups = self.create_bind_group(matrix_offset, &buffer, device);

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let texture = surface.get_current_texture().unwrap();

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("OnScreen render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline.as_ref().unwrap().pipeline);

            render_pass.set_vertex_buffer(0, buffer.slice(0..matrix_offset));

            render_pass.set_bind_group(0, &groups[0], &[]);

            render_pass.draw(0..3, 0..1);
        }

        queue.submit(vec![encoder.finish()]);

        texture.present();
    }
}

fn main() {
    let app = App::new("Hello World", 800, 800);

    app.run(HelloTriangle::new());
}
