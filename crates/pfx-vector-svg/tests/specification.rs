use pfx_vector_core::{path_length,Point2,Tolerance};use pfx_vector_svg::*;
#[test]fn compact_number_grammar_accepts_sign_separation(){let p=parse_path("M10-20L30.5.5").unwrap();assert_eq!(p.subpaths()[0].start(),Point2::new(10.0,-20.0));}
#[test]fn repeated_moveto_parameters_become_lineto(){let p=parse_path("M0 0 10 0 10 10").unwrap();assert_eq!(p.segment_count(),2);}
#[test]fn zero_radius_arc_becomes_line(){let p=parse_path("M0 0 A0 10 0 0 1 20 0").unwrap();assert!((path_length(&p,Tolerance::default()).unwrap()-20.0).abs()<1e-8);}
#[test]fn viewbox_meet_centers_content(){let t=viewbox_transform(ViewBox::new(0.0,0.0,100.0,100.0).unwrap(),Viewport{x:0.0,y:0.0,width:200.0,height:100.0},PreserveAspectRatio::default()).unwrap();assert_eq!(t.transform_point(Point2::new(0.0,0.0)),Point2::new(50.0,0.0));}
