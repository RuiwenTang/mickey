use super::{Raster, VertexMode};
use crate::core::{
    path::{Path, PathFillType, PolylineBuilder},
    Point,
};
use nalgebra::Point2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    CW,
    CCW,
    LINEAR,
}

impl Orientation {
    pub(crate) fn from(a: &Point, b: &Point, c: &Point) -> Self {
        let aa = Point2::<f64>::new(a.x as f64, a.y as f64);
        let bb = Point2::<f64>::new(b.x as f64, b.y as f64);
        let cc = Point2::<f64>::new(c.x as f64, c.y as f64);

        let v1 = bb - aa;
        let v2 = cc - aa;

        let cross = v1.x * v2.y - v1.y * v2.x;

        if cross > 0.0 {
            return Self::CW;
        } else if cross < 0.0 {
            return Self::CCW;
        } else {
            return Self::LINEAR;
        }
    }
}

pub(crate) struct PathFillRaster {
    path: Path,
}

impl PathFillRaster {
    pub(crate) fn new(path: Path) -> Self {
        Self { path }
    }

    fn do_raster(&self) -> (Vec<Point>, Vec<u32>, VertexMode) {
        let mut points: Vec<Point> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut front_count = 0;
        let mut back_count = 0;

        let polyline = PolylineBuilder::from(&self.path).build();

        for contour in &polyline.contours {
            if contour.points.len() < 3 {
                // can not fill contour with less than 3 points
                continue;
            }

            let first_pt = &contour.points[0];
            let first_index = points.len() as u32;
            points.push(first_pt.clone());

            let mut prev_pt = &contour.points[1];
            let mut prev_index = points.len() as u32;
            points.push(prev_pt.clone());

            for i in 2..contour.points.len() {
                let curr_pt = &contour.points[i];
                match Orientation::from(first_pt, prev_pt, curr_pt) {
                    Orientation::LINEAR => {
                        points.last_mut().unwrap().x = curr_pt.x;
                        points.last_mut().unwrap().y = curr_pt.y;
                        continue;
                    }
                    Orientation::CW => front_count += 1,
                    Orientation::CCW => back_count += 1,
                }

                let curr_index = points.len() as u32;
                points.push(curr_pt.clone());

                indices.push(first_index);
                indices.push(prev_index);
                indices.push(curr_index);

                prev_pt = curr_pt;
                prev_index = curr_index;
            }
        }

        let mode = if self.path.fill_type == PathFillType::EvenOdd {
            VertexMode::EvenOddFill
        } else if front_count == 0 || back_count == 0 {
            VertexMode::Convex
        } else {
            VertexMode::Complex
        };

        return (points, indices, mode);
    }
}

impl Raster for PathFillRaster {
    fn rasterize(
        &self,
        buffer: &mut crate::gpu::buffer::StageBuffer,
    ) -> (
        std::ops::Range<wgpu::BufferAddress>,
        std::ops::Range<wgpu::BufferAddress>,
        super::VertexMode,
        u32,
    ) {
        let (points, indices, mode) = self.do_raster();

        let vertex_range = buffer.push_data(bytemuck::cast_slice(&points));
        let index_range = buffer.push_data(bytemuck::cast_slice(&indices));

        (vertex_range, index_range, mode, indices.len() as u32)
    }
}
