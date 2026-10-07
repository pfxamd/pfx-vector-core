use crate::{CoreResult, Path, PathBuilder, Segment, Tolerance, Transform2D, flatten_path};
pub fn transform_path(path: &Path, t: Transform2D, tol: Tolerance) -> CoreResult<Path> {
    let mut b = PathBuilder::with_tolerance(tol);
    for sub in path.subpaths() {
        b.move_to(t.transform_point(sub.start()))?;
        for &seg in sub.segments() {
            match seg {
                Segment::Line(l) => {
                    b.line_to(t.transform_point(l.end))?;
                }
                Segment::Quadratic(q) => {
                    b.quad_to(t.transform_point(q.p1), t.transform_point(q.p2))?;
                }
                Segment::Cubic(c) => {
                    b.cubic_to(
                        t.transform_point(c.p1),
                        t.transform_point(c.p2),
                        t.transform_point(c.p3),
                    )?;
                }
                Segment::Arc(_) => {
                    let tmp = {
                        let mut pb = PathBuilder::with_tolerance(tol);
                        pb.move_to(seg.start())?;
                        if let Segment::Arc(a) = seg {
                            pb.arc_to(a)?;
                        }
                        pb.finish()?
                    };
                    let f = flatten_path(&tmp, tol)?;
                    for p in f[0].points.iter().skip(1) {
                        b.line_to(t.transform_point(*p))?;
                    }
                }
            }
        }
        if sub.is_closed() {
            b.close()?;
        }
    }
    b.finish()
}
