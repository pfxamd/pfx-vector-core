use pfx_vector_core::*;

#[test]
fn previously_nonconvergent_cubic_builds_measure_table() {
    let height = 27.342254980327226;
    let width = 48.06974101419256;
    let shape = 0.10805621451339115;
    let curve = Segment::Cubic(CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(width * 0.25, height),
        Point2::new(width * 0.75, height * shape),
        Point2::new(width, 0.0),
    ));
    let table = SegmentMeasureTable::build(curve, Tolerance::default()).unwrap();
    let mut previous = 0.0;
    for i in 0..=64 {
        let distance = table.length_at_parameter(i as f64 / 64.0).unwrap();
        assert!(distance.is_finite() && distance + 1.0e-8 >= previous);
        previous = distance;
    }
    assert!((previous - table.total_length()).abs() < 1.0e-8);
}

#[test]
fn previously_nonconvergent_elliptical_arc_builds_measure_table() {
    let arc = Segment::Arc(EllipticalArc::new(
        Point2::new(0.0, 0.0),
        37.39417787089321,
        10.0,
        Angle::degrees(0.0),
        Angle::degrees(5.0),
        Angle::degrees(265.99591814643725),
    ));
    let table = SegmentMeasureTable::build(arc, Tolerance::default()).unwrap();
    let mut previous = 0.0;
    for i in 0..=64 {
        let distance = table.length_at_parameter(i as f64 / 64.0).unwrap();
        assert!(distance.is_finite() && distance + 1.0e-8 >= previous);
        previous = distance;
    }
    assert!((previous - table.total_length()).abs() < 1.0e-8);
}
