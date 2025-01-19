use nalgebra::Vector2;

use crate::{
    Draw, FillRule, Matrix3x3, Mesh, MeshType, Path, Point, PolylineBuilder, Rect, StageBuffer,
};

pub(crate) trait Raster {
    fn do_raster(&self, transform: &Matrix3x3, buffer: &mut StageBuffer) -> Mesh;
}

impl Draw {
    pub(crate) fn gen_raster(&self) -> Box<dyn Raster> {
        match self {
            Draw::DrawRect(rect, _, _) => Box::new(RectFillRaster { rect: *rect }),
            Draw::DrawPath(path, _, _) => Box::new(PathFillRaster { path: path.clone() }),
        }
    }
}

struct RectFillRaster {
    rect: Rect,
}

impl Raster for RectFillRaster {
    fn do_raster(&self, _m: &Matrix3x3, buffer: &mut StageBuffer) -> Mesh {
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

        Mesh {
            vertex_buffer: vertex_view,
            index_buffer: index_view,
            draw_count: index.len() as u32,
            mesh_type: MeshType::Direct,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    CW,
    CCW,
    LINEAR,
}

impl Orientation {
    pub(crate) fn from(a: &Point, b: &Point, c: &Point) -> Self {
        let aa = Vector2::<f64>::new(a.x as f64, a.y as f64);
        let bb = Vector2::<f64>::new(b.x as f64, b.y as f64);
        let cc = Vector2::<f64>::new(c.x as f64, c.y as f64);

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
    pub(crate) path: Path,
}

impl Raster for PathFillRaster {
    fn do_raster(&self, m: &Matrix3x3, buffer: &mut StageBuffer) -> Mesh {
        let contours = PolylineBuilder::new(&self.path, *m).build();

        let mut points: Vec<Point> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut front_count = 0;
        let mut back_count = 0;

        for contour in contours {
            if contour.points.len() < 3 {
                // can not fill contour with less than 3 points
                continue;
            }

            let first_pt = &contour.points[0];
            let first_index = points.len() as u32;
            points.push(*first_pt);

            let mut prev_pt = &contour.points[1];
            let mut prev_index = points.len() as u32;
            points.push(*prev_pt);

            for i in 2..contour.points.len() {
                let curr_pt = &contour.points[i];
                match Orientation::from(first_pt, prev_pt, curr_pt) {
                    Orientation::LINEAR => {
                        points.last_mut().unwrap().x = curr_pt.x;
                        points.last_mut().unwrap().y = curr_pt.y;
                        prev_pt = curr_pt;
                        continue;
                    }
                    Orientation::CW => front_count += 1,
                    Orientation::CCW => back_count += 1,
                }

                let curr_index = points.len() as u32;
                points.push(*curr_pt);

                indices.push(first_index);
                indices.push(prev_index);
                indices.push(curr_index);

                prev_pt = curr_pt;
                prev_index = curr_index;
            }
        }

        let vertex_buffer = buffer.push(points.as_slice());
        let index_buffer = buffer.push(indices.as_slice());

        let mesh_type = if self.path.fill_rule() == FillRule::EvenOdd {
            MeshType::EvenOdd
        } else if front_count > 0 && back_count > 0 {
            MeshType::NonZero
        } else {
            MeshType::Direct
        };

        Mesh {
            vertex_buffer,
            index_buffer,
            draw_count: indices.len() as u32,
            mesh_type,
        }
    }
}
