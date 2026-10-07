#![forbid(unsafe_code)]
//! PFx Vector Core: deterministic 2D vector geometry primitives and algorithms.
//! The crate is independent of SVG syntax, DOM, rendering, and UI concerns.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub mod algorithms;
pub mod curves;
pub mod foundation;
pub mod geometry;
pub mod math;
pub mod numeric;
pub mod path;
pub use algorithms::*;
pub use curves::*;
pub use foundation::*;
pub use geometry::*;
pub use math::*;
pub use path::*;
pub mod prelude {
    pub use crate::{
        Angle, Bounds, CubicBezier, DynamicSpatialIndex, EllipticalArc,
        IncrementalPathSpatialIndex, LineSegment, Path, PathBuilder, PathSpatialIndex, Point2,
        QuadraticBezier, SpatialIndex, Tolerance, Transform2D, Vector2,
    };
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn point_vector_and_transform_basics() {
        let p = Point2::new(2.0, 3.0);
        let v = Vector2::new(4.0, -1.0);
        assert_eq!(p + v, Point2::new(6.0, 2.0));
        let t = Transform2D::translation(10.0, 20.0).then(Transform2D::scale(2.0, 3.0));
        assert_eq!(
            t.transform_point(Point2::new(1.0, 1.0)),
            Point2::new(22.0, 63.0)
        );
    }
    #[test]
    fn quadratic_endpoints_and_split() {
        let q = QuadraticBezier::new(
            Point2::new(0.0, 0.0),
            Point2::new(5.0, 10.0),
            Point2::new(10.0, 0.0),
        );
        assert_eq!(q.point_at(0.0), q.p0);
        assert_eq!(q.point_at(1.0), q.p2);
        let (a, b) = q.split(0.5);
        assert!(a.p2.almost_eq(b.p0, Tolerance::default()));
    }
    #[test]
    fn path_measurement_smoke() {
        let mut b = PathBuilder::new();
        b.move_to(Point2::new(0.0, 0.0))
            .unwrap()
            .line_to(Point2::new(3.0, 4.0))
            .unwrap();
        let p = b.finish().unwrap();
        assert!((path_length(&p, Tolerance::default()).unwrap() - 5.0).abs() < 1e-10);
    }
}
