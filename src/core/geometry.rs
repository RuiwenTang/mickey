use nalgebra::Vector2;

use super::Point;

pub(crate) const FLOAT_ROOT2_OVER2: f32 = 0.707106781;
pub(crate) const PI: f32 = 3.1415926;

/// used for eval(t) = a * t ^ 2 + b * t + c
pub(crate) struct QuadCoeff {
    a: Vector2<f64>,
    b: Vector2<f64>,
    c: Vector2<f64>,
}

impl QuadCoeff {
    pub(crate) fn from(a: &Point, b: &Point, c: &Point) -> Self {
        let cc = Vector2::new(a.x as f64, a.y as f64);
        let p1 = Vector2::new(b.x as f64, b.y as f64);
        let p2 = Vector2::new(c.x as f64, c.y as f64);

        let bb = (p1 - cc) * 2.0;
        let aa = p2 - (p1 * 2.0) + cc;

        return Self {
            a: aa,
            b: bb,
            c: cc,
        };
    }

    pub(crate) fn eval(&self, t: f32) -> Point {
        let tt = t as f64;

        let ret = (self.a * tt + self.b) * tt + self.c;

        return Point {
            x: ret.x as f32,
            y: ret.y as f32,
        };
    }
}

pub(crate) fn cross_product(p: &Point, q: &Point, r: &Point) -> f32 {
    (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y)
}

pub(crate) fn dot_product(p: &Vector2<f64>, q: &Vector2<f64>) -> f64 {
    p.x * q.x + p.y * q.y
}

pub(crate) fn distance(p: &Vector2<f64>) -> f64 {
    (p.x * p.x + p.y * p.y).sqrt()
}

pub(crate) fn degree_to_radian(degree: f32) -> f32 {
    degree * PI / 180.0
}

pub(crate) fn circle_interpolation(
    start: &Vector2<f64>,
    end: &Vector2<f64>,
    num: u32,
) -> Vec<Vector2<f64>> {
    let mut ret: Vec<Vector2<f64>> = Vec::with_capacity(num as usize);

    let cos_theta = dot_product(start, end);
    let step = 1.0 / (num as f64);

    let theta = f64::acos(cos_theta);
    let sin_theta = f64::sin(theta);

    for i in 1..(num + 1) {
        let t = step * (i as f64);
        let complement_tt = f64::sin((1.0 - t) * theta) / sin_theta;
        let tt = f64::sin(t * theta) / sin_theta;
        ret.push(complement_tt * start + tt * end);
    }

    return ret;
}

/// used for : eval(t)  a * t ^ 3 + b * t ^ 2 + c * t + d
pub(crate) struct CubicCoeff {
    a: Vector2<f64>,
    b: Vector2<f64>,
    c: Vector2<f64>,
    d: Vector2<f64>,
}

impl CubicCoeff {
    pub(crate) fn from(p1: &Point, p2: &Point, p3: &Point, p4: &Point) -> Self {
        let pp0 = Vector2::new(p1.x as f64, p1.y as f64);
        let pp1 = Vector2::new(p2.x as f64, p2.y as f64);
        let pp2 = Vector2::new(p3.x as f64, p3.y as f64);
        let pp3 = Vector2::new(p4.x as f64, p4.y as f64);

        let a = pp3 + (pp1 - pp2) * 3.0 - pp0;
        let b = (pp2 - pp1 * 2.0 + pp0) * 3.0;
        let c = (pp1 - pp0) * 3.0;
        let d = pp0;

        Self { a, b, c, d }
    }

    pub(crate) fn eval(&self, t: f32) -> Point {
        let tt = t as f64;
        let p = ((self.a * tt + self.b) * tt + self.c) * tt + self.d;

        Point {
            x: p.x as f32,
            y: p.y as f32,
        }
    }
}

/// quad curve with weight
pub(crate) struct ConicCoeff {
    numer: QuadCoeff,
    denom: QuadCoeff,
}

impl ConicCoeff {
    pub(crate) fn from(p1: &Point, p2: &Point, p3: &Point, w: f32) -> Self {
        let p1 = Vector2::new(p1.x as f64, p1.y as f64);
        let p2 = Vector2::new(p2.x as f64, p2.y as f64);
        let p3 = Vector2::new(p3.x as f64, p3.y as f64);

        let w = w as f64;

        let p2w = p2 * w;

        let c1 = p1;
        let a1 = p3 - p2w * 2.0 + p1;
        let b1 = (p2w - p1) * 2.0;

        let c2 = Vector2::<f64>::new(1.0, 1.0);
        let b2 = (Vector2::<f64>::new(w, w) - c2) * 2.0;
        let a2 = Vector2::<f64>::new(0.0, 0.0) - b2;

        Self {
            numer: QuadCoeff {
                a: a1,
                b: b1,
                c: c1,
            },
            denom: QuadCoeff {
                a: a2,
                b: b2,
                c: c2,
            },
        }
    }

    pub(crate) fn eval(&self, t: f32) -> Point {
        let n = self.numer.eval(t);
        let d = self.denom.eval(t);

        Point {
            x: n.x / d.x,
            y: n.y / d.y,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_quad_eval() {
        let quad = QuadCoeff::from(
            &Point { x: 0.0, y: 0.0 },
            &Point { x: 2.0, y: 0.0 },
            &Point { x: 2.0, y: 2.0 },
        );
        let mid = Point { x: 1.5, y: 0.5 };

        let p = quad.eval(0.5);

        assert_eq!(p.x, mid.x);
        assert_eq!(p.y, mid.y);
    }
}
