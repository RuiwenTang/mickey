use std::{borrow::BorrowMut, ops::Range, rc::Rc};

use ab_glyph::ScaleFont;
use nalgebra::{Matrix4, Vector4};

use crate::{
    core::{Color, Point},
    gpu::{buffer::StageBuffer, pipeline::Pipeline},
    text::TextBlob,
};

use super::{
    command::Command,
    fragment::{TransformGroup, SOLID_TEXT_PIPELINE_NAME},
    Renderer,
};

struct GlyphRunDrawable {
    texture: Rc<wgpu::Texture>,
    transform: TransformGroup,
    vertices: Vec<f32>,
    indices: Vec<u32>,

    color_range: Range<wgpu::BufferAddress>,
    vertex_range: Range<wgpu::BufferAddress>,
    index_range: Range<wgpu::BufferAddress>,
}

impl GlyphRunDrawable {
    fn new(
        texture: Rc<wgpu::Texture>,
        transform: TransformGroup,
        color_range: Range<wgpu::BufferAddress>,
    ) -> Self {
        Self {
            texture,
            transform,
            vertices: Vec::new(),
            indices: Vec::new(),
            color_range,
            vertex_range: 0..0,
            index_range: 0..0,
        }
    }

    fn add_vertex(&mut self, x: f32, y: f32, u: f32, v: f32) -> u32 {
        let index = self.vertices.len() as u32 / 4;

        self.vertices.extend_from_slice(&[x, y, u, v]);

        return index;
    }

    fn add_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.extend_from_slice(&[a, b, c]);
    }

    fn prepare(&mut self, buffer: &mut StageBuffer) {
        self.vertex_range = buffer.push_data(bytemuck::cast_slice(&self.vertices));

        self.index_range = buffer.push_data(bytemuck::cast_slice(&self.indices));
    }

    fn gen_command<'a>(
        &self,
        buffer: &'a wgpu::Buffer,
        pipeline: &'a Pipeline,
        context: &'a crate::gpu::GPUContext,
        device: &wgpu::Device,
    ) -> Command<'a> {
        let group0_layout = pipeline
            .get_group_layout(0)
            .expect("common group at slot 0 can not be get!");

        let group1_layout = pipeline
            .get_group_layout(1)
            .expect("text color group at slot 1 can not be get!");

        let groups = vec![
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Common Group"),
                layout: &group0_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: buffer,
                        offset: self.transform.get_buffer_range().start,
                        size: wgpu::BufferSize::new(
                            self.transform.get_buffer_range().end
                                - self.transform.get_buffer_range().start,
                        ),
                    }),
                }],
            }),
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Text Color Group"),
                layout: &group1_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: buffer,
                            offset: self.color_range.start,
                            size: wgpu::BufferSize::new(
                                self.color_range.end - self.color_range.start,
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &self
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(context.get_linear_sampler()),
                    },
                ],
            }),
        ];

        let state = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                read_mask: 0xff,
                write_mask: 0xff,
            },
            bias: Default::default(),
        };

        let raw_pipeline = pipeline.get_pipeline(&state).unwrap();

        return Command::new(
            raw_pipeline,
            buffer.slice(self.vertex_range.clone()),
            buffer.slice(self.index_range.clone()),
            self.indices.len() as u32,
            groups,
        );
    }
}

pub(crate) struct TextBlobRender {
    format: wgpu::TextureFormat,
    anti_alias: bool,
    blob: Rc<TextBlob>,
    color: Color,
    pos: Point,
    depth: f32,
    transform: TransformGroup,

    drawables: Vec<GlyphRunDrawable>,
}

impl TextBlobRender {
    pub(crate) fn new(
        format: wgpu::TextureFormat,
        anti_alias: bool,
        blob: Rc<TextBlob>,
        color: Color,
        pos: Point,
        depth: f32,
        vw: f32,
        vh: f32,
        transform: Matrix4<f32>,
    ) -> Self {
        Self {
            format,
            anti_alias,
            blob,
            color,
            pos,
            depth,
            transform: TransformGroup::new(
                Matrix4::new_orthographic(0.0, vw, vh, 0.0, -1000.0, 1000.0),
                transform,
                Vector4::new(0.0, 0.0, 0.0, 0.0),
            ),
            drawables: Vec::new(),
        }
    }
}

impl Renderer for TextBlobRender {
    fn pipeline_label(&self) -> &'static str {
        SOLID_TEXT_PIPELINE_NAME
    }

    fn prepare(
        &mut self,
        total_depth: f32,
        buffer: &mut StageBuffer,
        context: &crate::gpu::GPUContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.transform.prepare(self.depth / total_depth, buffer);

        let color_range = buffer.push_data_align(bytemuck::cast_slice(&[self.color]));

        let mut drawable: Option<GlyphRunDrawable> = None;

        for run in &self.blob.runs {
            // end previous drawable
            if drawable.is_some() {
                self.drawables.push(drawable.take().unwrap());
            }

            let font = run.font.clone();

            let fs = font.get_scaled_font(run.px_size);

            for glyph in run.glyphs.iter() {
                let mut am = context.get_atlas_manager();

                let mut region = am.query_atlas_region(font.as_ref(), glyph, run.px_size);

                if region.is_none() {
                    if drawable.is_some() {
                        self.drawables.push(drawable.take().unwrap());
                    }

                    region = am.borrow_mut().alloc_atlas_region(
                        font.as_ref(),
                        glyph,
                        run.px_size,
                        device,
                        queue,
                    );
                }

                let region = region.unwrap();

                if drawable.is_none() || drawable.as_ref().unwrap().texture != region.texture {
                    if drawable.is_some() {
                        self.drawables.push(drawable.take().unwrap());
                    }

                    drawable = Some(GlyphRunDrawable::new(
                        region.texture.clone(),
                        self.transform.clone(),
                        color_range.clone(),
                    ));
                }

                let mut g = glyph.clone();
                g.position.x += self.pos.x;
                // replace y with baseline position
                g.position.y = self.pos.y;

                let og = fs.outline_glyph(g);
                if let Some(outlined) = og {
                    let bounds = outlined.px_bounds();

                    let pa = Point::from(bounds.min.x, bounds.min.y);
                    let pb = Point::from(bounds.max.x, bounds.min.y);
                    let pc = Point::from(bounds.max.x, bounds.max.y);
                    let pd = Point::from(bounds.min.x, bounds.max.y);

                    let ua = Point::from(region.rect.left, region.rect.top);
                    let ub = Point::from(region.rect.right, region.rect.top);
                    let uc = Point::from(region.rect.right, region.rect.bottom);
                    let ud = Point::from(region.rect.left, region.rect.bottom);

                    let a = drawable
                        .as_mut()
                        .unwrap()
                        .add_vertex(pa.x, pa.y, ua.x, ua.y);
                    let b = drawable
                        .as_mut()
                        .unwrap()
                        .add_vertex(pb.x, pb.y, ub.x, ub.y);
                    let c = drawable
                        .as_mut()
                        .unwrap()
                        .add_vertex(pc.x, pc.y, uc.x, uc.y);
                    let d = drawable
                        .as_mut()
                        .unwrap()
                        .add_vertex(pd.x, pd.y, ud.x, ud.y);

                    drawable.as_mut().unwrap().add_triangle(a, b, c);
                    drawable.as_mut().unwrap().add_triangle(a, c, d);
                }
            }
        }

        if drawable.is_some() {
            self.drawables.push(drawable.take().unwrap());
        }

        for drawable in &mut self.drawables {
            drawable.prepare(buffer);
        }
    }

    fn render<'a>(
        &self,
        buffer: &'a wgpu::Buffer,
        context: &'a crate::gpu::GPUContext,
        device: &wgpu::Device,
    ) -> Vec<Command<'a>> {
        let pipeline = context.get_pipeline(self.pipeline_label(), self.format, self.anti_alias);

        if self.drawables.is_empty() || pipeline.is_none() {
            return vec![];
        }

        let pipeline = pipeline.unwrap();

        let mut commands = Vec::new();

        for drawable in &self.drawables {
            commands.push(drawable.gen_command(buffer, &pipeline, context, device));
        }

        return commands;
    }
}
