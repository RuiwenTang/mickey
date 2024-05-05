use super::{Raster, VertexMode};
use crate::core::{
    paint::{StrokeCap, StrokeJoin},
    path::{Contour, Path, PathFillType, PolylineBuilder},
    Point,
};
use nalgebra::{Point2, Vector2};

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

pub(crate) struct PathFill {
    path: Path,
}

impl PathFill {
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

impl Raster for PathFill {
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

pub(crate) struct PathStroke {
    path: Path,
    stroke_width: f32,
    miter_limit: f32,
    cap: StrokeCap,
    join: StrokeJoin,
}

impl PathStroke {
    pub(crate) fn new(
        path: Path,
        stroke_width: f32,
        miter_limit: f32,
        cap: StrokeCap,
        join: StrokeJoin,
    ) -> Self {
        Self {
            path,
            stroke_width,
            miter_limit,
            cap,
            join,
        }
    }

    pub(crate) fn stroke_contour(
        &self,
        contour: &Contour,
        mut points: Vec<Point>,
        mut indices: Vec<u32>,
    ) -> (Vec<Point>, Vec<u32>) {
        for i in 0..contour.points.len() - 1 {
            let p1 = &contour.points[i];
            let p2 = &contour.points[i + 1];

            if p1 == p2 {
                continue;
            }

            let (a, b, c, d) = self.expend_line(p1, p2);

            let a_index = points.len() as u32;
            points.push(a.clone());

            let b_index = points.len() as u32;
            points.push(b.clone());

            let c_index = points.len() as u32;
            points.push(c.clone());

            let d_index = points.len() as u32;
            points.push(d.clone());

            // a --------- c
            // |           |
            // |           |
            // b-----------d
            indices.push(a_index);
            indices.push(b_index);
            indices.push(c_index);

            indices.push(b_index);
            indices.push(d_index);
            indices.push(c_index);
        }

        return (points, indices);
    }

    fn expend_line(&self, p1: &Point, p2: &Point) -> (Point, Point, Point, Point) {
        let stroke_radius = (self.stroke_width as f64) * 0.5;

        let p1 = Vector2::new(p1.x as f64, p1.y as f64);
        let p2 = Vector2::new(p2.x as f64, p2.y as f64);

        let dir = (p2 - p1).normalize();
        let normal = Vector2::new(-dir.y, dir.x);

        let a = p1 + normal * stroke_radius;
        let b = p1 - normal * stroke_radius;
        let c = p2 + normal * stroke_radius;
        let d = p2 - normal * stroke_radius;

        return (
            Point::from_highp(a.x, a.y),
            Point::from_highp(b.x, b.y),
            Point::from_highp(c.x, c.y),
            Point::from_highp(d.x, d.y),
        );
    }
}

impl Raster for PathStroke {
    fn rasterize(
        &self,
        buffer: &mut crate::gpu::buffer::StageBuffer,
    ) -> (
        std::ops::Range<wgpu::BufferAddress>,
        std::ops::Range<wgpu::BufferAddress>,
        VertexMode,
        u32,
    ) {
        let polyline = PolylineBuilder::from(&self.path).build();

        let mut points: Vec<Point> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for contour in &polyline.contours {
            (points, indices) = self.stroke_contour(contour, points, indices);
        }

        let vertex_range = buffer.push_data(bytemuck::cast_slice(&points));
        let index_range = buffer.push_data(bytemuck::cast_slice(&indices));

        return (
            vertex_range,
            index_range,
            VertexMode::NonOverlap,
            indices.len() as u32,
        );
    }
}
