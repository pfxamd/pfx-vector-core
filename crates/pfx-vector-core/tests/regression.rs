use pfx_vector_core::*;
#[test]
fn zero_length_line_does_not_create_nan(){let l=LineSegment::new(Point2::new(1.0,2.0),Point2::new(1.0,2.0));assert_eq!(l.length(),0.0);assert!(l.bounds().contains(l.start));}
#[test]
fn overlapping_lines_report_range(){let a=Segment::Line(LineSegment::new(Point2::new(0.0,0.0),Point2::new(10.0,0.0)));let b=Segment::Line(LineSegment::new(Point2::new(5.0,0.0),Point2::new(15.0,0.0)));let r=intersect_segments(a,b,Tolerance::default()).unwrap();assert!(matches!(r.intersections.first(),Some(Intersection::Overlap(_))));}
#[test]
fn near_singular_matrix_is_rejected(){let m=Transform2D::new(1.0,0.0,1.0,1e-15,0.0,0.0);assert!(matches!(m.inverse(Tolerance::default()),Err(CoreError::SingularTransform)));}
