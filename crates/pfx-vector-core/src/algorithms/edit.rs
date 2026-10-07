use crate::{
    Angle, CoreError, CoreResult, EllipticalArc, LineSegment, Path, Point2, Scalar, Segment,
    Subpath, Tolerance, segment_length, segment_parameter_at_length,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentAddress {
    pub subpath_index: usize,
    pub segment_index: usize,
}

impl SegmentAddress {
    #[must_use]
    pub const fn new(subpath_index: usize, segment_index: usize) -> Self {
        Self {
            subpath_index,
            segment_index,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubpathEndpoint {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathEditKind {
    SplitSegment,
    ReverseSubpath,
    JoinSubpaths,
    SetClosed,
    ReplaceSegment,
    RemoveSegment,
    RemoveSubpath,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathEditReport {
    pub kind: PathEditKind,
    pub affected_subpaths: Vec<usize>,
    pub subpaths_before: usize,
    pub subpaths_after: usize,
    pub segments_before: usize,
    pub segments_after: usize,
    pub topology_changed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathEditResult {
    pub path: Path,
    pub report: PathEditReport,
}

impl PathEditResult {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub const fn report(&self) -> &PathEditReport {
        &self.report
    }

    #[must_use]
    pub fn into_path(self) -> Path {
        self.path
    }
}

pub fn split_segment(
    path: &Path,
    address: SegmentAddress,
    t: Scalar,
    tolerance: Tolerance,
) -> CoreResult<PathEditResult> {
    if !t.is_finite() {
        return Err(CoreError::InvalidNumber);
    }
    if !(0.0 < t && t < 1.0) {
        return Err(CoreError::InvalidGeometry);
    }

    let subpath = subpath_at(path, address.subpath_index)?;
    let segment = segment_at(subpath, address.segment_index)?;
    let (left, right) = split_segment_native(segment, t);

    let mut segments = subpath.segments().to_vec();
    segments.splice(address.segment_index..=address.segment_index, [left, right]);

    let replacement = Subpath::new(subpath.start(), segments, subpath.is_closed(), tolerance)
        .ok_or(CoreError::InvalidGeometry)?;
    replace_subpath_internal(
        path,
        address.subpath_index,
        vec![replacement],
        PathEditKind::SplitSegment,
        true,
    )
}

pub fn split_segment_at_length(
    path: &Path,
    address: SegmentAddress,
    distance: Scalar,
    tolerance: Tolerance,
) -> CoreResult<PathEditResult> {
    if !distance.is_finite() {
        return Err(CoreError::InvalidNumber);
    }

    let subpath = subpath_at(path, address.subpath_index)?;
    let segment = segment_at(subpath, address.segment_index)?;
    let total = segment_length(segment, tolerance)?;
    if !(0.0 < distance && distance < total) {
        return Err(CoreError::InvalidGeometry);
    }

    let t = segment_parameter_at_length(segment, distance, tolerance)?;
    split_segment(path, address, t, tolerance)
}

pub fn insert_anchor(
    path: &Path,
    address: SegmentAddress,
    t: Scalar,
    tolerance: Tolerance,
) -> CoreResult<PathEditResult> {
    split_segment(path, address, t, tolerance)
}

pub fn reverse_subpath(path: &Path, subpath_index: usize) -> CoreResult<PathEditResult> {
    let subpath = subpath_at(path, subpath_index)?;
    replace_subpath_internal(
        path,
        subpath_index,
        vec![subpath.reversed()],
        PathEditKind::ReverseSubpath,
        false,
    )
}

pub fn set_subpath_closed(
    path: &Path,
    subpath_index: usize,
    closed: bool,
    tolerance: Tolerance,
) -> CoreResult<PathEditResult> {
    let subpath = subpath_at(path, subpath_index)?;
    if subpath.is_closed() == closed {
        return Ok(edit_result(
            path.clone(),
            path,
            PathEditKind::SetClosed,
            vec![subpath_index],
            false,
        ));
    }

    let replacement = Subpath::new(
        subpath.start(),
        subpath.segments().to_vec(),
        closed,
        tolerance,
    )
    .ok_or(CoreError::InvalidGeometry)?;
    replace_subpath_internal(
        path,
        subpath_index,
        vec![replacement],
        PathEditKind::SetClosed,
        true,
    )
}

pub fn replace_segment(
    path: &Path,
    address: SegmentAddress,
    replacement: Segment,
    tolerance: Tolerance,
) -> CoreResult<PathEditResult> {
    validate_segment(replacement)?;

    let subpath = subpath_at(path, address.subpath_index)?;
    let original = segment_at(subpath, address.segment_index)?;
    if !replacement.start().almost_eq(original.start(), tolerance)
        || !replacement.end().almost_eq(original.end(), tolerance)
    {
        return Err(CoreError::InvalidGeometry);
    }

    let mut segments = subpath.segments().to_vec();
    segments[address.segment_index] = replacement;
    let updated = Subpath::new(subpath.start(), segments, subpath.is_closed(), tolerance)
        .ok_or(CoreError::InvalidGeometry)?;

    replace_subpath_internal(
        path,
        address.subpath_index,
        vec![updated],
        PathEditKind::ReplaceSegment,
        false,
    )
}

pub fn remove_segment(
    path: &Path,
    address: SegmentAddress,
    tolerance: Tolerance,
) -> CoreResult<PathEditResult> {
    let subpath = subpath_at(path, address.subpath_index)?;
    let segments = subpath.segments();
    let removed = segment_at(subpath, address.segment_index)?;

    let replacements = if subpath.is_closed() {
        if segments.len() == 1 {
            Vec::new()
        } else {
            let mut remaining = Vec::with_capacity(segments.len() - 1);
            remaining.extend_from_slice(&segments[address.segment_index + 1..]);
            remaining.extend_from_slice(&segments[..address.segment_index]);
            vec![
                Subpath::new(removed.end(), remaining, false, tolerance)
                    .ok_or(CoreError::InvalidGeometry)?,
            ]
        }
    } else if segments.len() == 1 {
        Vec::new()
    } else if address.segment_index == 0 {
        let remaining = segments[1..].to_vec();
        vec![
            Subpath::new(remaining[0].start(), remaining, false, tolerance)
                .ok_or(CoreError::InvalidGeometry)?,
        ]
    } else if address.segment_index + 1 == segments.len() {
        vec![
            Subpath::new(
                subpath.start(),
                segments[..address.segment_index].to_vec(),
                false,
                tolerance,
            )
            .ok_or(CoreError::InvalidGeometry)?,
        ]
    } else {
        let left = Subpath::new(
            subpath.start(),
            segments[..address.segment_index].to_vec(),
            false,
            tolerance,
        )
        .ok_or(CoreError::InvalidGeometry)?;
        let right_segments = segments[address.segment_index + 1..].to_vec();
        let right = Subpath::new(
            right_segments[0].start(),
            right_segments,
            false,
            tolerance,
        )
        .ok_or(CoreError::InvalidGeometry)?;
        vec![left, right]
    };

    replace_subpath_internal(
        path,
        address.subpath_index,
        replacements,
        PathEditKind::RemoveSegment,
        true,
    )
}

pub fn remove_subpath(path: &Path, subpath_index: usize) -> CoreResult<PathEditResult> {
    subpath_at(path, subpath_index)?;

    let mut subpaths = path.subpaths().to_vec();
    subpaths.remove(subpath_index);
    let edited = Path::from_subpaths(subpaths);
    Ok(edit_result(
        edited,
        path,
        PathEditKind::RemoveSubpath,
        Vec::new(),
        true,
    ))
}

pub fn join_open_subpaths(
    path: &Path,
    first_index: usize,
    first_endpoint: SubpathEndpoint,
    second_index: usize,
    second_endpoint: SubpathEndpoint,
    tolerance: Tolerance,
) -> CoreResult<PathEditResult> {
    if first_index == second_index {
        return Err(CoreError::InvalidGeometry);
    }

    let first_source = subpath_at(path, first_index)?;
    let second_source = subpath_at(path, second_index)?;
    if first_source.is_closed() || second_source.is_closed() {
        return Err(CoreError::UnsupportedCase);
    }

    let first = match first_endpoint {
        SubpathEndpoint::End => first_source.clone(),
        SubpathEndpoint::Start => first_source.reversed(),
    };
    let second = match second_endpoint {
        SubpathEndpoint::Start => second_source.clone(),
        SubpathEndpoint::End => second_source.reversed(),
    };

    if !first.end().almost_eq(second.start(), tolerance) {
        return Err(CoreError::InvalidGeometry);
    }

    let mut segments = first.segments().to_vec();
    if first.end() != second.start() {
        segments.push(Segment::Line(LineSegment::new(first.end(), second.start())));
    }
    segments.extend_from_slice(second.segments());

    let joined = Subpath::new(first.start(), segments, false, tolerance)
        .ok_or(CoreError::InvalidGeometry)?;

    let keep_index = first_index.min(second_index);
    let remove_index = first_index.max(second_index);
    let mut subpaths = path.subpaths().to_vec();
    subpaths[keep_index] = joined;
    subpaths.remove(remove_index);

    let edited = Path::from_subpaths(subpaths);
    Ok(edit_result(
        edited,
        path,
        PathEditKind::JoinSubpaths,
        vec![keep_index],
        true,
    ))
}

fn replace_subpath_internal(
    path: &Path,
    subpath_index: usize,
    replacements: Vec<Subpath>,
    kind: PathEditKind,
    topology_changed: bool,
) -> CoreResult<PathEditResult> {
    subpath_at(path, subpath_index)?;

    let mut subpaths = path.subpaths().to_vec();
    subpaths.splice(subpath_index..=subpath_index, replacements);
    let edited = Path::from_subpaths(subpaths);
    Ok(edit_result(
        edited,
        path,
        kind,
        vec![subpath_index],
        topology_changed,
    ))
}

fn edit_result(
    edited: Path,
    original: &Path,
    kind: PathEditKind,
    affected_subpaths: Vec<usize>,
    topology_changed: bool,
) -> PathEditResult {
    PathEditResult {
        report: PathEditReport {
            kind,
            affected_subpaths,
            subpaths_before: original.subpaths().len(),
            subpaths_after: edited.subpaths().len(),
            segments_before: original.segment_count(),
            segments_after: edited.segment_count(),
            topology_changed,
        },
        path: edited,
    }
}

fn subpath_at(path: &Path, index: usize) -> CoreResult<&Subpath> {
    path.subpaths().get(index).ok_or(CoreError::InvalidGeometry)
}

fn segment_at(subpath: &Subpath, index: usize) -> CoreResult<Segment> {
    subpath
        .segments()
        .get(index)
        .copied()
        .ok_or(CoreError::InvalidGeometry)
}

fn split_segment_native(segment: Segment, t: Scalar) -> (Segment, Segment) {
    match segment {
        Segment::Line(line) => {
            let point = line.point_at(t);
            (
                Segment::Line(LineSegment::new(line.start, point)),
                Segment::Line(LineSegment::new(point, line.end)),
            )
        }
        Segment::Quadratic(curve) => {
            let (left, right) = curve.split(t);
            (Segment::Quadratic(left), Segment::Quadratic(right))
        }
        Segment::Cubic(curve) => {
            let (left, right) = curve.split(t);
            (Segment::Cubic(left), Segment::Cubic(right))
        }
        Segment::Arc(arc) => {
            let sweep = arc.sweep_angle.as_radians();
            let left = EllipticalArc::new(
                arc.center,
                arc.radius_x,
                arc.radius_y,
                arc.rotation,
                arc.start_angle,
                Angle::radians(sweep * t),
            );
            let right = EllipticalArc::new(
                arc.center,
                arc.radius_x,
                arc.radius_y,
                arc.rotation,
                Angle::radians(arc.start_angle.as_radians() + sweep * t),
                Angle::radians(sweep * (1.0 - t)),
            );
            (Segment::Arc(left), Segment::Arc(right))
        }
    }
}

fn validate_segment(segment: Segment) -> CoreResult<()> {
    let valid = match segment {
        Segment::Line(line) => line.start.is_finite() && line.end.is_finite(),
        Segment::Quadratic(curve) => {
            curve.p0.is_finite() && curve.p1.is_finite() && curve.p2.is_finite()
        }
        Segment::Cubic(curve) => {
            curve.p0.is_finite()
                && curve.p1.is_finite()
                && curve.p2.is_finite()
                && curve.p3.is_finite()
        }
        Segment::Arc(arc) => {
            arc.center.is_finite()
                && arc.radius_x.is_finite()
                && arc.radius_y.is_finite()
                && arc.radius_x > 0.0
                && arc.radius_y > 0.0
                && arc.rotation.as_radians().is_finite()
                && arc.start_angle.as_radians().is_finite()
                && arc.sweep_angle.as_radians().is_finite()
        }
    };

    if valid {
        Ok(())
    } else {
        Err(CoreError::InvalidGeometry)
    }
}
