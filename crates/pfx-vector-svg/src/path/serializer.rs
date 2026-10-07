use pfx_vector_core::{Path, Scalar, Segment};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SerializeOptions {
    pub precision: usize,
    pub compact: bool,
}

impl Default for SerializeOptions {
    fn default() -> Self {
        Self {
            precision: 6,
            compact: false,
        }
    }
}

fn num(value: Scalar, precision: usize) -> String {
    let mut text = format!("{value:.precision$}");

    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }

    if text == "-0" {
        "0".to_owned()
    } else {
        text
    }
}

#[must_use]
pub fn serialize_path(path: &Path, options: SerializeOptions) -> String {
    let mut parts = Vec::new();

    for subpath in path.subpaths() {
        let start = subpath.start();
        parts.push(format!(
            "M {} {}",
            num(start.x, options.precision),
            num(start.y, options.precision)
        ));

        for &segment in subpath.segments() {
            match segment {
                Segment::Line(line) => {
                    parts.push(format!(
                        "L {} {}",
                        num(line.end.x, options.precision),
                        num(line.end.y, options.precision)
                    ));
                }
                Segment::Quadratic(curve) => {
                    parts.push(format!(
                        "Q {} {} {} {}",
                        num(curve.p1.x, options.precision),
                        num(curve.p1.y, options.precision),
                        num(curve.p2.x, options.precision),
                        num(curve.p2.y, options.precision)
                    ));
                }
                Segment::Cubic(curve) => {
                    parts.push(format!(
                        "C {} {} {} {} {} {}",
                        num(curve.p1.x, options.precision),
                        num(curve.p1.y, options.precision),
                        num(curve.p2.x, options.precision),
                        num(curve.p2.y, options.precision),
                        num(curve.p3.x, options.precision),
                        num(curve.p3.y, options.precision)
                    ));
                }
                Segment::Arc(arc) => {
                    let end = arc.point_at(1.0);
                    parts.push(format!(
                        "A {} {} {} {} {} {} {}",
                        num(arc.radius_x, options.precision),
                        num(arc.radius_y, options.precision),
                        num(arc.rotation.as_degrees(), options.precision),
                        i32::from(
                            arc.sweep_angle.as_radians().abs() > core::f64::consts::PI
                        ),
                        i32::from(arc.sweep_angle.as_radians() >= 0.0),
                        num(end.x, options.precision),
                        num(end.y, options.precision)
                    ));
                }
            }
        }

        if subpath.is_closed() {
            parts.push("Z".to_owned());
        }
    }

    if options.compact {
        parts.join(" ")
    } else {
        parts.join(" ")
    }
}
