use crate::{Draw, Point, RasterResult, Rect, StageBuffer, Style};

pub(crate) trait Raster {
    fn do_raster(&self, buffer: &mut StageBuffer) -> RasterResult;
}

/// Create the raster based on the draw.
///
/// # Arguments
///
/// * `draw` - The draw to be rastered.
///
/// # Return
///
/// Return the raster. None means not supported.
pub(crate) fn create_raster(draw: &Draw) -> Option<Box<dyn Raster>> {
    match draw {
        Draw::DrawRect(rect, paint, _) => match paint.style {
            Style::Fill => Some(Box::new(RectFillRaster { rect: *rect })),
            _ => None,
        },

        _ => None,
    }
}

struct RectFillRaster {
    rect: Rect,
}

impl Raster for RectFillRaster {
    fn do_raster(&self, buffer: &mut StageBuffer) -> RasterResult {
        let mut vertex: Vec<Point> = vec![];
        let mut index: Vec<u32> = vec![];

        let left_top = Point::new(self.rect.left(), self.rect.top());
        let left_bottom = Point::new(self.rect.left(), self.rect.bottom());
        let right_top = Point::new(self.rect.right(), self.rect.top());
        let right_bottom = Point::new(self.rect.right(), self.rect.bottom());

        vertex.push(left_top);
        vertex.push(left_bottom);
        vertex.push(right_top);
        vertex.push(right_bottom);

        index.push(0);
        index.push(1);
        index.push(2);
        index.push(2);
        index.push(1);
        index.push(3);

        let vertex_view = buffer.push(vertex.as_slice());
        let index_view = buffer.push(index.as_slice());

        RasterResult::Direct(vertex_view, index_view, index.len() as u32)
    }
}
