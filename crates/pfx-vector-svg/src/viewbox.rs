use pfx_vector_core::{CoreError, CoreResult, Scalar, Transform2D};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewBox {
    pub min_x: Scalar,
    pub min_y: Scalar,
    pub width: Scalar,
    pub height: Scalar,
}
impl ViewBox {
    pub fn new(min_x: Scalar, min_y: Scalar, width: Scalar, height: Scalar) -> CoreResult<Self> {
        if ![min_x, min_y, width, height].iter().all(|v| v.is_finite())
            || width <= 0.0
            || height <= 0.0
        {
            return Err(CoreError::InvalidGeometry);
        }
        Ok(Self {
            min_x,
            min_y,
            width,
            height,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub x: Scalar,
    pub y: Scalar,
    pub width: Scalar,
    pub height: Scalar,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Align {
    None,
    XMinYMin,
    XMidYMin,
    XMaxYMin,
    XMinYMid,
    XMidYMid,
    XMaxYMid,
    XMinYMax,
    XMidYMax,
    XMaxYMax,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeetOrSlice {
    Meet,
    Slice,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreserveAspectRatio {
    pub align: Align,
    pub meet_or_slice: MeetOrSlice,
}
impl Default for PreserveAspectRatio {
    fn default() -> Self {
        Self {
            align: Align::XMidYMid,
            meet_or_slice: MeetOrSlice::Meet,
        }
    }
}
pub fn viewbox_transform(
    vb: ViewBox,
    vp: Viewport,
    par: PreserveAspectRatio,
) -> CoreResult<Transform2D> {
    if vp.width < 0.0
        || vp.height < 0.0
        || ![vp.x, vp.y, vp.width, vp.height]
            .iter()
            .all(|v| v.is_finite())
    {
        return Err(CoreError::InvalidGeometry);
    }
    let sx = vp.width / vb.width;
    let sy = vp.height / vb.height;
    if par.align == Align::None {
        return Ok(Transform2D::translation(vp.x, vp.y)
            .then(Transform2D::scale(sx, sy))
            .then(Transform2D::translation(-vb.min_x, -vb.min_y)));
    }
    let s = match par.meet_or_slice {
        MeetOrSlice::Meet => sx.min(sy),
        MeetOrSlice::Slice => sx.max(sy),
    };
    let rendered_w = vb.width * s;
    let rendered_h = vb.height * s;
    let extra_x = vp.width - rendered_w;
    let extra_y = vp.height - rendered_h;
    let (ax, ay) = match par.align {
        Align::XMinYMin => (0.0, 0.0),
        Align::XMidYMin => (0.5, 0.0),
        Align::XMaxYMin => (1.0, 0.0),
        Align::XMinYMid => (0.0, 0.5),
        Align::XMidYMid => (0.5, 0.5),
        Align::XMaxYMid => (1.0, 0.5),
        Align::XMinYMax => (0.0, 1.0),
        Align::XMidYMax => (0.5, 1.0),
        Align::XMaxYMax => (1.0, 1.0),
        Align::None => unreachable!(),
    };
    Ok(
        Transform2D::translation(vp.x + extra_x * ax, vp.y + extra_y * ay)
            .then(Transform2D::scale(s, s))
            .then(Transform2D::translation(-vb.min_x, -vb.min_y)),
    )
}
pub fn parse_preserve_aspect_ratio(s: &str) -> Option<PreserveAspectRatio> {
    let mut it = s.split_whitespace();
    let a = it.next()?;
    let align = match a {
        "none" => Align::None,
        "xMinYMin" => Align::XMinYMin,
        "xMidYMin" => Align::XMidYMin,
        "xMaxYMin" => Align::XMaxYMin,
        "xMinYMid" => Align::XMinYMid,
        "xMidYMid" => Align::XMidYMid,
        "xMaxYMid" => Align::XMaxYMid,
        "xMinYMax" => Align::XMinYMax,
        "xMidYMax" => Align::XMidYMax,
        "xMaxYMax" => Align::XMaxYMax,
        _ => return None,
    };
    let mode = match it.next() {
        Some("slice") => MeetOrSlice::Slice,
        Some("meet") | None => MeetOrSlice::Meet,
        _ => return None,
    };
    Some(PreserveAspectRatio {
        align,
        meet_or_slice: mode,
    })
}
