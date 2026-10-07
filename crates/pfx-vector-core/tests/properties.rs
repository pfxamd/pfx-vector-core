use pfx_vector_core::*;
use proptest::prelude::*;
proptest! {
    #[test]
    fn distance_is_symmetric(ax in -1e6f64..1e6, ay in -1e6f64..1e6, bx in -1e6f64..1e6, by in -1e6f64..1e6) {
        let a=Point2::new(ax,ay);let b=Point2::new(bx,by);
        prop_assert!((a.distance_to(b)-b.distance_to(a)).abs()<=1e-9);
    }
    #[test]
    fn affine_inverse_round_trip(x in -1e4f64..1e4, y in -1e4f64..1e4, tx in -1e3f64..1e3, ty in -1e3f64..1e3, sx in 0.1f64..10.0, sy in 0.1f64..10.0) {
        let p=Point2::new(x,y);let m=Transform2D::translation(tx,ty).then(Transform2D::scale(sx,sy));let inv=m.inverse(Tolerance::default()).unwrap();let q=inv.transform_point(m.transform_point(p));prop_assert!(p.almost_eq(q,Tolerance::new(1e-7,1e-11,1e-10,1e-4)));
    }
}
