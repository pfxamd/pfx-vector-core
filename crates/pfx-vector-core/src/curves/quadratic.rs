use crate::{
    Bounds, Point2, Scalar, Tolerance, Vector2
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuadraticBezier {
    pub p0: Point2, pub p1: Point2, pub p2: Point2
}
impl QuadraticBezier {
    #[must_use]
    pub const fn new(p0: Point2, p1: Point2, p2: Point2) -> Self {
        Self {
            p0, p1, p2
        }
    }
    #[must_use]
    pub fn point_at(self, t: Scalar) -> Point2 {
        let mt = 1.0-t;
        Point2::new(mt*mt*self.p0.x+2.0*mt*t*self.p1.x+t*t*self.p2.x, mt*mt*self.p0.y+2.0*mt*t*self.p1.y+t*t*self.p2.y)
    }
    #[must_use]
    pub fn derivative_at(self, t: Scalar) -> Vector2 {
        (self.p1-self.p0)*(2.0*(1.0-t))+(self.p2-self.p1)*(2.0*t)
    }
    pub fn tangent_at(self, t: Scalar, tol: Tolerance) -> crate::CoreResult<Vector2> {
        self.derivative_at(t).normalized(tol)
    }
    #[must_use]
    pub fn split(self, t: Scalar) -> (Self, Self) {
        let a = self.p0.lerp(self.p1, t);
        let b = self.p1.lerp(self.p2, t);
        let m = a.lerp(b, t);
        (Self::new(self.p0, a, m), Self::new(m, b, self.p2))
    }
    #[must_use]
    pub fn subcurve(self, t0: Scalar, t1: Scalar) -> Self {
        if t0 <= 0.0 && t1 >= 1.0 {
            return self
        }
        let (_, right) = self.split(t0.clamp(0.0, 1.0));
        let local = if t0 >= 1.0 {
            0.0
        } else {
            ((t1-t0)/(1.0-t0)).clamp(0.0, 1.0)
        };
        right.split(local).0
    }
    #[must_use]
    pub fn extrema(self) -> Vec<Scalar> {
        let mut out = Vec::with_capacity(2);
        for (a, b, c) in[(self.p0.x, self.p1.x, self.p2.x), (self.p0.y, self.p1.y, self.p2.y)] {
            let d = a-2.0*b+c;
            if d.abs()>f64::EPSILON {
                let t = (a-b)/d;
                if t>0.0 && t<1.0 && out.iter().all(|v|(v-t).abs()>1e-12) {
                    out.push(t)
                }
            }
        }
        out.sort_by(|a, b|a.total_cmp(b));
        out
    }
    #[must_use]
    pub fn bounds(self) -> Bounds {
        let mut b = Bounds::from_points(&[self.p0, self.p2]);
        for t in self.extrema() {
            b = b.include(self.point_at(t));
        }
        b
    }
    #[must_use]
    pub fn flatness(self) -> Scalar {
        let chord = self.p2-self.p0;
        let len = chord.length();
        if len == 0.0 {
            return self.p0.distance_to(self.p1)
        }
        ((self.p1-self.p0).cross(chord)).abs()/len
    }
    #[must_use]
    pub fn reversed(self) -> Self {
        Self::new(self.p2, self.p1, self.p0)
    }
}
