use super::{Raster, VertexMode};
use crate::core::{
    geometry::QuadCoeff,
    path::{Path, PathFillType, PathVerb},
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

const CURVE_STEP: f32 = 16.0;

impl PathFillRaster {
    pub(crate) fn new(path: Path) -> Self {
        Self { path }
    }

    fn do_raster(&self) -> (Vec<Point>, Vec<u32>, VertexMode) {
        let mut points: Vec<Point> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut front_count = 0;
        let mut back_count = 0;

        let mut first_pt: Option<Point> = None;
        let mut first_pt_index: Option<u32> = None;
        let mut prev_pt: Option<Point> = None;
        let mut prev_pt_index: Option<u32> = None;

        for v in &self.path.verts {
            match v {
                PathVerb::MoveTo(p) => {
                    first_pt = Some(p.clone());
                    first_pt_index = Some(points.len() as u32);
                    prev_pt = None;
                    prev_pt_index = None;

                    points.push(p.clone());
                }
                PathVerb::LineTo(p) => {
                    if prev_pt.is_none() {
                        prev_pt = Some(p.clone());
                        prev_pt_index = Some(points.len() as u32);
                        points.push(p.clone());
                    } else {
                        match Orientation::from(
                            first_pt.as_ref().unwrap(),
                            prev_pt.as_ref().unwrap(),
                            p,
                        ) {
                            Orientation::LINEAR => {
                                continue;
                            }
                            Orientation::CW => {
                                front_count += 1;
                            }
                            Orientation::CCW => {
                                back_count += 1;
                            }
                        }
                        let curr_index = points.len() as u32;

                        points.push(p.clone());

                        indices.push(first_pt_index.unwrap());
                        indices.push(prev_pt_index.unwrap());
                        indices.push(curr_index);

                        prev_pt = Some(p.clone());
                        prev_pt_index = Some(curr_index);
                    }
                }
                PathVerb::QuadTo(p1, p2) => {
                    if prev_pt.is_none() && first_pt.is_none() {
                        // some thing is wrong
                        continue;
                    }

                    let a = if prev_pt.is_some() {
                        prev_pt.as_ref().unwrap().clone()
                    } else {
                        first_pt.as_ref().unwrap().clone()
                    };

                    // TODO: flatten curve dynamic
                    let quad = QuadCoeff::from(&a, p1, p2);

                    let start = if prev_pt.is_none() {
                        prev_pt = Some(a.clone());
                        prev_pt_index = Some(points.len() as u32);
                        points.push(a.clone());
                        1
                    } else {
                        0
                    };

                    for step in start..(CURVE_STEP as i32) {
                        let t = (step as f32 + 1.0) / CURVE_STEP;
                        let curr = quad.eval(t);

                        match Orientation::from(
                            first_pt.as_ref().unwrap(),
                            prev_pt.as_ref().unwrap(),
                            &curr,
                        ) {
                            Orientation::LINEAR => {
                                prev_pt_index = Some(points.len() as u32);
                                points.push(curr);
                                prev_pt = Some(curr.clone());
                                continue;
                            }
                            Orientation::CW => {
                                front_count += 1;
                            }
                            Orientation::CCW => {
                                back_count += 1;
                            }
                        }
                        let curr_index = points.len() as u32;

                        points.push(curr.clone());
                        indices.push(first_pt_index.unwrap());
                        indices.push(prev_pt_index.unwrap());
                        indices.push(curr_index);

                        prev_pt = Some(curr.clone());
                        prev_pt_index = Some(curr_index);
                    }
                }
                PathVerb::Close => {
                    first_pt = None;
                    first_pt_index = None;
                    prev_pt = None;
                    prev_pt_index = None;
                }
                _ => {}
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
