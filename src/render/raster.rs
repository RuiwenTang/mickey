use super::{Raster, VertexMode};
use crate::core::{
    geometry::{circle_interpolation, cross_product, distance},
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

fn handle_bevel_join(
    prev_join: &Point,
    next_join: &Point,
    center: &Point,
    points: &mut Vec<Point>,
    indices: &mut Vec<u32>,
) {
    let center_index = points.len() as u32;
    points.push(center.clone());

    let prev_index = points.len() as u32;
    points.push(prev_join.clone());

    let next_index = points.len() as u32;
    points.push(next_join.clone());

    indices.push(prev_index);
    indices.push(center_index);
    indices.push(next_index);
}

fn handle_miter_join(
    prev_join: &Point,
    next_join: &Point,
    center: &Point,
    stroke_radius: f32,
    limit: f32,
    points: &mut Vec<Point>,
    indices: &mut Vec<u32>,
) -> bool {
    let limit = limit as f64;
    let stroke_radius = stroke_radius as f64;
    let prev_join = Vector2::new(prev_join.x as f64, prev_join.y as f64);
    let next_join = Vector2::new(next_join.x as f64, next_join.y as f64);
    let center = Vector2::new(center.x as f64, center.y as f64);

    let pp1 = prev_join - center;
    let pp2 = next_join - center;

    let out_dir = pp1 + pp2;

    let k = 2.0 * stroke_radius * stroke_radius / (out_dir.x * out_dir.x + out_dir.y * out_dir.y);

    let pe = out_dir * k;

    if distance(&pe) >= limit * stroke_radius {
        return false;
    }

    let join = center + pe;

    let center_index = points.len() as u32;
    points.push(Point::from_highp(center.x, center.y));

    let join_index = points.len() as u32;
    points.push(Point::from_highp(join.x, join.y));

    let prev_index = points.len() as u32;
    points.push(Point::from_highp(prev_join.x, prev_join.y));

    let next_index = points.len() as u32;
    points.push(Point::from_highp(next_join.x, next_join.y));

    indices.push(join_index);
    indices.push(prev_index);
    indices.push(center_index);

    indices.push(join_index);
    indices.push(center_index);
    indices.push(next_index);

    return true;
}

fn gen_round_mesh(
    prev_join: &Vector2<f64>,
    next_join: &Vector2<f64>,
    center: &Vector2<f64>,
    stroke_radius: f64,
    points: &mut Vec<Point>,
    indices: &mut Vec<u32>,
) {
    let start = (prev_join - center).normalize();
    let end = (next_join - center).normalize();

    let result = circle_interpolation(&start, &end, 8);

    let center_index = points.len() as u32;
    points.push(Point::from_highp(center.x, center.y));

    let mut prev_index = points.len() as u32;
    points.push(Point::from_highp(prev_join.x, prev_join.y));

    for d in &result {
        let curr_index = points.len() as u32;
        let p = d * stroke_radius + center;

        points.push(Point::from_highp(p.x, p.y));

        indices.push(prev_index);
        indices.push(center_index);
        indices.push(curr_index);

        prev_index = curr_index;
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
        for i in 0..contour.points.len() {
            if !contour.closed && i == contour.points.len() - 1 {
                break;
            }

            let curr_index = i;
            let next_index = if i == contour.points.len() - 1 {
                let mut j: usize = 0;
                while j < contour.points.len() && contour.points[j] == contour.points[curr_index] {
                    j = j + 1;
                }
                j
            } else {
                i + 1
            };

            let p1 = &contour.points[curr_index];
            let p2 = &contour.points[next_index];

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

            if !contour.closed && i == 0 {
                continue;
            }

            let prev = if i == 0 {
                &contour.points.len() - 1
            } else {
                i - 1
            };

            let p0 = &contour.points[prev];
            let orientation = Orientation::from(p0, p1, p2);
            let cross = cross_product(p0, p1, p2);

            if orientation == Orientation::LINEAR && cross > 0.0 {
                continue;
            }

            let (prev_join, next_join) = self.get_join_points(p0, p1, p2, orientation, cross);

            match self.join {
                StrokeJoin::Miter => {
                    if handle_miter_join(
                        &prev_join,
                        &next_join,
                        p1,
                        self.stroke_width * 0.5,
                        self.miter_limit,
                        &mut points,
                        &mut indices,
                    ) {
                        continue;
                    }

                    handle_bevel_join(&prev_join, &next_join, p1, &mut points, &mut indices);
                }
                StrokeJoin::Round => {
                    let p0 = Vector2::new(p0.x as f64, p0.y as f64);
                    let p1 = Vector2::new(p1.x as f64, p1.y as f64);
                    let p2 = Vector2::new(p2.x as f64, p2.y as f64);

                    let pp1 = (p1 - p0).normalize();
                    let pp2 = (p1 - p2).normalize();

                    let out_dir = ((pp1 + pp2) * 0.5).normalize();
                    let stroke_radius = self.stroke_width as f64 * 0.5;

                    let out_p = p1 + out_dir * stroke_radius;

                    let prev_join = Vector2::new(prev_join.x as f64, prev_join.y as f64);
                    let next_join = Vector2::new(next_join.x as f64, next_join.y as f64);

                    gen_round_mesh(
                        &prev_join,
                        &out_p,
                        &p1,
                        stroke_radius,
                        &mut points,
                        &mut indices,
                    );

                    gen_round_mesh(
                        &out_p,
                        &next_join,
                        &p1,
                        stroke_radius,
                        &mut points,
                        &mut indices,
                    );
                }
                StrokeJoin::Bevel => {
                    handle_bevel_join(&prev_join, &next_join, p1, &mut points, &mut indices);
                }
            }
        }

        if contour.closed {
        } else {
            self.handle_cap(contour, &mut points, &mut indices);
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

    fn get_join_points(
        &self,
        p0: &Point,
        p1: &Point,
        p2: &Point,
        orientation: Orientation,
        cross: f32,
    ) -> (Point, Point) {
        let p0 = Vector2::new(p0.x as f64, p0.y as f64);
        let p1 = Vector2::new(p1.x as f64, p1.y as f64);
        let p2 = Vector2::new(p2.x as f64, p2.y as f64);

        let prev_dir = (p1 - p0).normalize();
        let next_dir = (p2 - p1).normalize();

        let prev_normal = Vector2::new(-prev_dir.y, prev_dir.x);
        let next_normal = Vector2::new(-next_dir.y, next_dir.x);

        let stroke_radius = self.stroke_width as f64 * 0.5;

        if orientation == Orientation::CW || (orientation == Orientation::LINEAR && cross < 0.0) {
            let prev_join_pt = p1 - prev_normal * stroke_radius;
            let next_join_pt = p1 - next_normal * stroke_radius;

            return (
                Point::from_highp(prev_join_pt.x, prev_join_pt.y),
                Point::from_highp(next_join_pt.x, next_join_pt.y),
            );
        } else {
            let prev_join_pt = p1 + prev_normal * stroke_radius;
            let next_join_pt = p1 + next_normal * stroke_radius;

            return (
                Point::from_highp(prev_join_pt.x, prev_join_pt.y),
                Point::from_highp(next_join_pt.x, next_join_pt.y),
            );
        }
    }

    fn handle_cap(&self, contour: &Contour, points: &mut Vec<Point>, indices: &mut Vec<u32>) {
        match self.cap {
            StrokeCap::Butt => {}
            StrokeCap::Round => {
                let start = Vector2::new(contour.points[0].x as f64, contour.points[0].y as f64);
                let next = Vector2::new(contour.points[1].x as f64, contour.points[1].y as f64);

                let out = (start - next).normalize();
                let normal = Vector2::new(-out.y, out.x);

                let stroke_radius = self.stroke_width as f64 * 0.5;
                let out_p = start + out * stroke_radius;

                let p0 = start + normal * stroke_radius;
                let p1 = start - normal * stroke_radius;

                gen_round_mesh(&p0, &out_p, &start, stroke_radius, points, indices);
                gen_round_mesh(&out_p, &p1, &start, stroke_radius, points, indices);

                let start = Vector2::new(
                    contour.points[contour.points.len() - 1].x as f64,
                    contour.points[contour.points.len() - 1].y as f64,
                );

                let next = Vector2::new(
                    contour.points[contour.points.len() - 2].x as f64,
                    contour.points[contour.points.len() - 2].y as f64,
                );

                let out = (start - next).normalize();
                let normal = Vector2::new(-out.y, out.x);

                let stroke_radius = self.stroke_width as f64 * 0.5;
                let out_p = start + out * stroke_radius;

                let p0 = start + normal * stroke_radius;
                let p1 = start - normal * stroke_radius;

                gen_round_mesh(&p0, &out_p, &start, stroke_radius, points, indices);
                gen_round_mesh(&out_p, &p1, &start, stroke_radius, points, indices);
            }
            StrokeCap::Square => {
                let start = Vector2::new(contour.points[0].x as f64, contour.points[0].y as f64);
                let next = Vector2::new(contour.points[1].x as f64, contour.points[1].y as f64);

                let out = (start - next).normalize();
                let normal = Vector2::new(-out.y, out.x);

                let stroke_radius = self.stroke_width as f64 * 0.5;

                let p0 = start + normal * stroke_radius;
                let p1 = start - normal * stroke_radius;

                let p2 = p0 + out * stroke_radius;
                let p3 = p1 + out * stroke_radius;

                let a = points.len() as u32;
                points.push(Point::from_highp(p0.x, p0.y));

                let b = points.len() as u32;
                points.push(Point::from_highp(p1.x, p1.y));

                let c = points.len() as u32;
                points.push(Point::from_highp(p2.x, p2.y));

                let d = points.len() as u32;
                points.push(Point::from_highp(p3.x, p3.y));

                indices.push(a);
                indices.push(b);
                indices.push(c);

                indices.push(b);
                indices.push(d);
                indices.push(c);

                let start = Vector2::new(
                    contour.points[contour.points.len() - 1].x as f64,
                    contour.points[contour.points.len() - 1].y as f64,
                );

                let next = Vector2::new(
                    contour.points[contour.points.len() - 2].x as f64,
                    contour.points[contour.points.len() - 2].y as f64,
                );

                let out = (start - next).normalize();
                let normal = Vector2::new(-out.y, out.x);

                let stroke_radius = self.stroke_width as f64 * 0.5;

                let p0 = start + normal * stroke_radius;
                let p1 = start - normal * stroke_radius;

                let p2 = p0 + out * stroke_radius;
                let p3 = p1 + out * stroke_radius;

                let a = points.len() as u32;
                points.push(Point::from_highp(p0.x, p0.y));

                let b = points.len() as u32;
                points.push(Point::from_highp(p1.x, p1.y));

                let c = points.len() as u32;
                points.push(Point::from_highp(p2.x, p2.y));

                let d = points.len() as u32;
                points.push(Point::from_highp(p3.x, p3.y));

                indices.push(a);
                indices.push(b);
                indices.push(c);

                indices.push(b);
                indices.push(d);
                indices.push(c);
            }
        }
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
