use crate::{Color, Draw, Layer, Picture, Rect, RenderContext};

/// Surface wrap a wgpu::Texture as a render target.
/// This is a one-time operation, after flush the surface is no longer usable.
/// No need to present the surface after flush.
pub struct Surface<'a> {
    texture: &'a wgpu::Texture,
    color: Color,
    cmds: Vec<Draw>,
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
            Rect::new_xywh(
                0.0,
                0.0,
                self.texture.width() as f32,
                self.texture.height() as f32,
            ),
            1,
            self.cmds.as_slice(),
            self.texture,
            self.color,
        );

        root_layer.flush(context, device, queue);
    }
}
