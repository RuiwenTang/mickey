use nalgebra::Vector2;

use super::Point;

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
