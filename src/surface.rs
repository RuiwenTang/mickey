use crate::{Color, Draw, Layer, Picture, Rect, RenderContext};

/// Surface wrap a wgpu::Texture as a render target.
/// This is a one-time operation, after flush the surface is no longer usable.
/// No need to present the surface after flush.
pub struct Surface<'a> {
    texture: &'a wgpu::Texture,
    color: Color,
    cmds: Vec<Draw>,
    view_port: Rect,
}

impl<'a> Surface<'a> {
    /// Create a new surface from a wgpu::Texture.
    ///
    /// # Arguments
    ///
    /// * `texture` - The wgpu::Texture to wrap as a render target.
    pub fn new(texture: &'a wgpu::Texture) -> Self {
        Surface {
            texture,
            color: Color::transparent(),
            cmds: Vec::new(),
            view_port: Rect::new_xywh(0.0, 0.0, texture.width() as f32, texture.height() as f32),
        }
    }

    /// Set the clear color of the surface.
    /// The default clear color is transparent.
    ///
    /// # Arguments
    ///
    /// * `color` - The clear color of the surface.
    pub fn with_clear_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the view port of the surface.
    /// The default view port is the full size of the surface.
    /// This is a logical level view port, which decides the area of the content to be rendered.
    ///
    /// # Arguments
    ///
    /// * `view_port` - The view port of the surface.
    pub fn with_view_port(mut self, view_port: Rect) -> Self {
        self.view_port = view_port;
        self
    }

    /// Replay the content of the [`Picture`] on the surface.
    /// The picture will be rendered with the same transform as the picture.
    ///
    /// # Arguments
    ///
    /// * `picture` - The picture to replay.
    pub fn replay(mut self, picture: &Picture) -> Self {
        self.cmds.extend_from_slice(picture.draws.as_slice());
        self
    }

    /// Flush the content of the surface to the target [`wgpu::Texture`].
    pub fn flush(self, context: &mut RenderContext, device: &wgpu::Device, queue: &wgpu::Queue) {
        let root_layer = Layer::new(
            self.view_port,
            1,
            self.cmds.as_slice(),
            self.texture,
            self.color,
        );

        root_layer.flush(context, device, queue);
    }
}
