use std::num::NonZero;

use crate::{
    DepthStencilStateProvider, Fragment, Gradient, Group, LinearGradient, RadialGradient,
    ShaderGenerator, StageBuffer, Uniform,
};

/// Common shader and upload logical for all gradient fragment.
/// It contains the struct for common gradient info:
/// ```wgsl
///   struct GradientInfo {
///       colors : array<vec4<f32>, N>,
///       stops  : array<f32, N / 4>,
///       infos  : vec4<f32>, // [color_count, stop_count, alpha, TBD]
///   }
/// ```
///
/// If gradient is simple gradient, the stops is deleted. And the struct is:
/// ```wgsl
///   struct GradientInfo {
///       colors : array<vec4<f32>, N>,
///       infos  : vec4<f32>, // [color_count, TBD, alpha, TBD]
///   }
/// ```
pub(crate) struct CommonGradient {
    count: u32,
    is_simple: bool,
}

/// Linear gradient fragment shader.
/// The linear gradient point located at group 1 binding 1.
pub(crate) struct LinearGradientFragment<DS: DepthStencilStateProvider> {
    gradient: LinearGradient,
    common: CommonGradient,

    _ds: std::marker::PhantomData<DS>,
}

pub(crate) struct RadialGradientFragment<DS: DepthStencilStateProvider> {
    gradient: RadialGradient,
    common: CommonGradient,
    _ds: std::marker::PhantomData<DS>,
}

fn round_color_count(color_count: u32) -> u32 {
    let mut count = 1;

    while count < color_count {
        count <<= 1;
    }

    if count > 64 {
        count = 64;
    }

    count
}

impl CommonGradient {
    pub(crate) fn new(gradient: &Gradient) -> Self {
        let count = round_color_count(gradient.colors.len() as u32);
        Self {
            count,
            is_simple: gradient.is_simple(),
        }
    }

    fn generate_info_struct(&self) -> String {
        if self.is_simple {
            format!(
                r#"
                struct GradientInfo {{
                    colors : array<vec4<f32>, {}>,
                    infos  : vec4<f32>, // [color_count, stop_count, alpha, TBD]
                }}
            "#,
                self.count
            )
        } else {
            format!(
                r#"
                struct GradientInfo {{
                    colors : array<vec4<f32>, {}>,
                    stops  : array<f32, {}>,
                    infos  : vec4<f32>, // [color_count, stop_count, alpha, TBD]
                }}
            "#,
                self.count,
                (self.count + 3) / 4
            )
        }
    }

    fn generate_common_code(&self) -> String {
        let mut code = String::new();

        if self.is_simple {
            code += r#"
                fn lerp_color(current: f32, info : GradientInfo) -> vec4<f32> {
                    return mix(info.colors[0], info.colors[1], current);
                }
            "#;
        } else {
            code += r#"
                fn get_stop(info : GradientInfo, index : u32) -> f32 {
                    var batch_index = index / 4;
                    var offset = index % 4;

                    var stop = info.stops[batch_index];

                    return stop[offset];
                }

                fn lerp_color(current: f32, info : GradientInfo) -> vec4<f32> {
                    var t = current;
                    if t >= 1.0 {
                        t = 1.0;
                    }

                    var color_count = i32(info.infos[0]);
                    var stop_count = i32(info.infos[1]);

                    var start_index = 0;
                    var end_index = 1;

                    if stop_count > 0 && t <= get_stop(info, 0) {
                        return info.colors[0];
                    }

                    var step        = 1.0 / f32(stop_count - 1);
                    var i           = 0;
                    var start       = 0.0;
                    var end         = 0.0;

                    for (; i < color_count; i += 1) {
                        if stop_count > 0 {
                            start = get_stop(info, i);
                            end   = get_stop(info, i + 1);
                        } else {
                            start = f32(i) * step;
                            end   = f32(i + 1) * step;
                        }

                        if t >= start && t <= end {
                            start_index = i;
                            end_index   = i + 1;
                            break;
                        }
                    }

                    if i == color_count - 1 && color_count > 0 {
                        return info.colors[color_count - 1];
                    }

                    var total = end - start;
                    var value = t - start;

                    var mix_value = 0.5;
                    if total > 0.0 {
                        mix_value = value / total;
                    }

                    return mix(info.colors[start_index], info.colors[end_index], mix_value);
                }
            "#;
        }

        // TODO : support TileMode
        code += r#"
            fn calculate_gradient_color(t: f32, info: GradientInfo) -> vec4<f32> {
                return lerp_color(clamp(t, 0.0, 1.0), info);
            }
        "#;

        return code;
    }

    pub(crate) fn get_group_entry(&self) -> wgpu::BindGroupLayoutEntry {
        // color array size
        let mut size = 16 * self.count as wgpu::BufferAddress;

        if !self.is_simple {
            // stop array size
            size += 4 * ((self.count + 3) / 4) as wgpu::BufferAddress;
        }

        // infos vec4 size
        size += 16;

        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZero::new(size),
            },
            count: None,
        }
    }

    pub(crate) fn gen_uniform(&self, gradient: &Gradient, buffer: &mut StageBuffer) -> Uniform {
        let mut stage_buffer: Vec<u8> = Vec::new();

        {
            let mut colors: Vec<[f32; 4]> = Vec::new();
            colors.resize(self.count as usize, [0.0; 4]);

            for (i, c) in gradient.colors.iter().enumerate() {
                colors[i] = c.clone().into();
            }

            stage_buffer.extend_from_slice(bytemuck::cast_slice(colors.as_slice()));
        }

        if !self.is_simple {
            let mut stops: Vec<f32> = Vec::new();
            stops.resize((((self.count + 3) / 4) * 4) as usize, 0.0);

            for (i, s) in gradient.stops.as_ref().unwrap().iter().enumerate() {
                stops[i] = *s;
            }

            stage_buffer.extend_from_slice(bytemuck::cast_slice(stops.as_slice()));
        }

        {
            let mut infos: Vec<f32> = Vec::new();
            infos.push(self.count as f32);
            if self.is_simple {
                infos.push(0.0);
            } else {
                infos.push(gradient.stops.as_ref().unwrap().len() as f32);
            }
            infos.push(gradient.alpha);
            infos.push(0.0);
            stage_buffer.extend_from_slice(bytemuck::cast_slice(infos.as_slice()));
        }

        let view = buffer.push_align(stage_buffer.as_slice());

        return Uniform::Buffer(view, 0);
    }
}

impl<DS: DepthStencilStateProvider> LinearGradientFragment<DS> {
    pub(crate) fn new(gradient: LinearGradient) -> Self {
        let common = CommonGradient::new(&gradient.gradient);
        Self {
            gradient,
            common,
            _ds: std::marker::PhantomData,
        }
    }
}

impl<DS: DepthStencilStateProvider> Fragment for LinearGradientFragment<DS> {
    fn depth_stencil_state(&self) -> wgpu::DepthStencilState {
        DS::stencil_state()
    }

    fn blend_state(&self) -> Option<wgpu::BlendState> {
        Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING)
    }

    fn get_group_entry(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![self.common.get_group_entry(), wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZero::new(16 as wgpu::BufferAddress),
            },
            count: None,
        }]
    }

    fn gen_group(
        &self,
        buffer: &mut StageBuffer,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Option<Group> {
        let uniform = self.common.gen_uniform(&self.gradient.gradient, buffer);
        let pts = [
            self.gradient.start.x,
            self.gradient.start.y,
            self.gradient.end.x,
            self.gradient.end.y,
        ];
        let view = buffer.push_align(&pts);
        Some(Group {
            index: 1,
            bindings: vec![uniform, Uniform::Buffer(view, 1)],
        })
    }

    fn stencil_state_name(&self) -> &'static str {
        DS::name()
    }
}

impl<DS: DepthStencilStateProvider> ShaderGenerator for LinearGradientFragment<DS> {
    fn shader_name(&self) -> String {
        format!(
            "LinearGradientFragment_{}_{}",
            if self.gradient.gradient.is_simple() {
                "simple"
            } else {
                "complex"
            },
            self.common.count
        )
    }

    fn shader_code(&self) -> String {
        let mut code = self.common.generate_info_struct();

        code += &self.common.generate_common_code();

        code += r#"
            @group(1) @binding(0) var<uniform> gradient_info    : GradientInfo;
            @group(1) @binding(1) var<uniform> linear_pts       : vec4<f32>;

            @fragment
            fn fs_main(@location(0) v_pos: vec2<f32>) -> @location(0) vec4<f32> {
                var cs: vec2<f32> = v_pos - linear_pts.xy;
                var se: vec2<f32> = linear_pts.zw - linear_pts.xy;

                var t: f32 = dot(cs, se) / dot(se, se);

                var color = calculate_gradient_color(t, gradient_info);

                return vec4<f32>(color.rgb * color.a, color.a);
            }
        "#;

        return code;
    }
}

impl<DS: DepthStencilStateProvider> RadialGradientFragment<DS> {
    pub(crate) fn new(gradient: RadialGradient) -> Self {
        let common = CommonGradient::new(&gradient.gradient);
        Self {
            gradient,
            common,
            _ds: std::marker::PhantomData,
        }
    }
}

impl<DS: DepthStencilStateProvider> Fragment for RadialGradientFragment<DS> {
    fn depth_stencil_state(&self) -> wgpu::DepthStencilState {
        DS::stencil_state()
    }

    fn blend_state(&self) -> Option<wgpu::BlendState> {
        Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING)
    }

    fn get_group_entry(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![self.common.get_group_entry(), wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZero::new(16 as wgpu::BufferAddress),
            },
            count: None,
        }]
    }

    fn gen_group(
        &self,
        buffer: &mut StageBuffer,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Option<Group> {
        let uniform = self.common.gen_uniform(&self.gradient.gradient, buffer);

        let pts = [
            self.gradient.center.x,
            self.gradient.center.y,
            self.gradient.radius,
            0.0,
        ];

        let view = buffer.push_align(&pts);

        Some(Group {
            index: 1,
            bindings: vec![uniform, Uniform::Buffer(view, 1)],
        })
    }

    fn stencil_state_name(&self) -> &'static str {
        DS::name()
    }
}

impl<DS: DepthStencilStateProvider> ShaderGenerator for RadialGradientFragment<DS> {
    fn shader_name(&self) -> String {
        format!(
            "RadialGradientFragment_{}_{}",
            if self.gradient.gradient.is_simple() {
                "simple"
            } else {
                "complex"
            },
            self.common.count
        )
    }

    fn shader_code(&self) -> String {
        let mut code = self.common.generate_info_struct();
        code += &self.common.generate_common_code();
        code += r#"
            @group(1) @binding(0) var<uniform> gradient_info    : GradientInfo;
            @group(1) @binding(1) var<uniform> radial_pts       : vec4<f32>;
            @fragment
            fn fs_main(@location(0) v_pos: vec2<f32>) -> @location(0) vec4<f32> {
                var cs: vec2<f32> = v_pos - radial_pts.xy;
                var r: f32 = length(cs);
                var t: f32 = r / radial_pts.z;
                var color = calculate_gradient_color(t, gradient_info);
                return vec4<f32>(color.rgb * color.a, color.a);
            }
            "#;

        return code;
    }
}
