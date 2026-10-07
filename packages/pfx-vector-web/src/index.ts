export type StrokeCap = "butt" | "round" | "square";
export type StrokeJoin = "miter" | "round" | "bevel";
export type SubpathEndpoint = "start" | "end";
export type TransformMatrix = [
  number,
  number,
  number,
  number,
  number,
  number,
];

export interface VectorWasmBindings {
  validate_path(data: string): boolean;
  path_length_svg(data: string): number;
  contour_length_svg(data: string, subpathIndex: number): number;
  slice_contour_svg(
    data: string,
    subpathIndex: number,
    startDistance: number,
    endDistance: number,
    wrap: boolean,
  ): string;
  split_segment_svg(
    data: string,
    subpathIndex: number,
    segmentIndex: number,
    t: number,
  ): string;
  split_segment_at_length_svg(
    data: string,
    subpathIndex: number,
    segmentIndex: number,
    distance: number,
  ): string;
  reverse_subpath_svg(data: string, subpathIndex: number): string;
  set_subpath_closed_svg(data: string, subpathIndex: number, closed: boolean): string;
  remove_segment_svg(data: string, subpathIndex: number, segmentIndex: number): string;
  remove_subpath_svg(data: string, subpathIndex: number): string;
  join_subpaths_svg(
    data: string,
    firstIndex: number,
    firstAtStart: boolean,
    secondIndex: number,
    secondAtStart: boolean,
  ): string;
  transform_path_svg(data: string, matrix: string, flatness: number): string;
  path_bounds_svg(data: string): string;
  point_at_length_svg(data: string, distance: number): string;
  hit_test_fill_svg(data: string, x: number, y: number, evenOdd: boolean): boolean;
  hit_test_stroke_svg(
    data: string,
    x: number,
    y: number,
    width: number,
    cap: StrokeCap,
    join: StrokeJoin,
    miterLimit: number,
  ): boolean;
  flatten_path_svg(data: string, flatness: number): string;
  normalize_path_svg(data: string): string;
  normalize_fill_contours_svg(data: string, evenOdd: boolean): string;
  dash_path_svg(data: string, dashArray: string, dashOffset: number): string;
  intersect_paths_svg(a: string, b: string): string;
  boolean_union_svg(a: string, b: string): string;
  boolean_intersection_svg(a: string, b: string): string;
  boolean_difference_svg(a: string, b: string): string;
  boolean_xor_svg(a: string, b: string): string;
  offset_path_svg(
    data: string,
    distance: number,
    join: StrokeJoin,
    miterLimit: number,
  ): string;
  outline_path_svg(
    data: string,
    width: number,
    cap: StrokeCap,
    join: StrokeJoin,
    miterLimit: number,
  ): string;
  outline_dashed_path_svg(
    data: string,
    width: number,
    cap: StrokeCap,
    join: StrokeJoin,
    miterLimit: number,
    dashArray: string,
    dashOffset: number,
  ): string;
  tessellate_fill_svg(data: string, evenOdd: boolean, flatness: number): string;
  tessellate_stroke_svg(
    data: string,
    width: number,
    cap: StrokeCap,
    join: StrokeJoin,
    miterLimit: number,
    flatness: number,
  ): string;
  cleanup_path_svg(
    data: string,
    pointTolerance: number,
    collinearTolerance: number,
  ): string;
  simplify_path_svg(data: string, maxDeviation: number): string;
  fit_path_curves_svg(data: string, maxError: number): string;
}

export interface Bounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

export interface PathPoint {
  x: number;
  y: number;
  subpath: number;
  segment: number;
  t: number;
  distance: number;
}

export interface IntersectionPoint {
  x: number;
  y: number;
}

export interface Mesh2D {
  vertices: number[];
  indices: number[];
}

export interface TessellationOptions {
  flatness?: number;
}

export interface ContourOptions {
  subpathIndex?: number;
}

export interface ContourSliceOptions extends ContourOptions {
  wrap?: boolean;
}

export interface ContourSplit {
  before: string;
  after: string;
  totalLength: number;
}

export interface OffsetOptions {
  join?: StrokeJoin;
  miterLimit?: number;
}

export interface DashOptions {
  dashArray?: number[];
  dashOffset?: number;
}

export interface OutlineOptions extends OffsetOptions, DashOptions {
  cap?: StrokeCap;
}

export interface CleanupOptions {
  pointTolerance?: number;
  collinearTolerance?: number;
}

export function createVectorCore(wasm: VectorWasmBindings) {
  return {
    validatePath: (data: string) => wasm.validate_path(data),
    pathLength: (data: string) => wasm.path_length_svg(data),
    contourLength: (data: string, subpathIndex = 0) =>
      wasm.contour_length_svg(data, subpathIndex),
    sliceContour: (
      data: string,
      startDistance: number,
      endDistance: number,
      options: ContourSliceOptions = {},
    ) =>
      wasm.slice_contour_svg(
        data,
        options.subpathIndex ?? 0,
        startDistance,
        endDistance,
        options.wrap ?? false,
      ),
    splitContour: (
      data: string,
      distance: number,
      options: ContourOptions = {},
    ): ContourSplit => {
      const subpathIndex = options.subpathIndex ?? 0;
      const totalLength = wasm.contour_length_svg(data, subpathIndex);
      return {
        before: wasm.slice_contour_svg(data, subpathIndex, 0, distance, false),
        after: wasm.slice_contour_svg(
          data,
          subpathIndex,
          distance,
          totalLength,
          false,
        ),
        totalLength,
      };
    },
    splitSegment: (
      data: string,
      subpathIndex: number,
      segmentIndex: number,
      t: number,
    ) => wasm.split_segment_svg(data, subpathIndex, segmentIndex, t),
    splitSegmentAtLength: (
      data: string,
      subpathIndex: number,
      segmentIndex: number,
      distance: number,
    ) =>
      wasm.split_segment_at_length_svg(
        data,
        subpathIndex,
        segmentIndex,
        distance,
      ),
    insertAnchor: (
      data: string,
      subpathIndex: number,
      segmentIndex: number,
      t: number,
    ) => wasm.split_segment_svg(data, subpathIndex, segmentIndex, t),
    reverseSubpath: (data: string, subpathIndex: number) =>
      wasm.reverse_subpath_svg(data, subpathIndex),
    setSubpathClosed: (data: string, subpathIndex: number, closed: boolean) =>
      wasm.set_subpath_closed_svg(data, subpathIndex, closed),
    removeSegment: (data: string, subpathIndex: number, segmentIndex: number) =>
      wasm.remove_segment_svg(data, subpathIndex, segmentIndex),
    removeSubpath: (data: string, subpathIndex: number) =>
      wasm.remove_subpath_svg(data, subpathIndex),
    joinSubpaths: (
      data: string,
      firstIndex: number,
      firstEndpoint: SubpathEndpoint,
      secondIndex: number,
      secondEndpoint: SubpathEndpoint,
    ) =>
      wasm.join_subpaths_svg(
        data,
        firstIndex,
        firstEndpoint === "start",
        secondIndex,
        secondEndpoint === "start",
      ),
    transformPath: (
      data: string,
      matrix: TransformMatrix,
      options: TessellationOptions = {},
    ) =>
      wasm.transform_path_svg(
        data,
        matrix.join(","),
        options.flatness ?? 1e-4,
      ),
    pathBounds: (data: string) =>
      JSON.parse(wasm.path_bounds_svg(data)) as Bounds | null,
    pointAtLength: (data: string, distance: number) =>
      JSON.parse(wasm.point_at_length_svg(data, distance)) as PathPoint,
    containsPoint: (
      data: string,
      x: number,
      y: number,
      rule: "nonzero" | "evenodd" = "nonzero",
    ) => wasm.hit_test_fill_svg(data, x, y, rule === "evenodd"),
    hitStroke: (
      data: string,
      x: number,
      y: number,
      width: number,
      options: OutlineOptions = {},
    ) => {
      const dashArray = options.dashArray ?? [];
      const strokeData =
        dashArray.length > 0
          ? wasm.dash_path_svg(data, dashArray.join(","), options.dashOffset ?? 0)
          : data;
      return wasm.hit_test_stroke_svg(
        strokeData,
        x,
        y,
        width,
        options.cap ?? "butt",
        options.join ?? "miter",
        options.miterLimit ?? 4,
      );
    },
    flattenPath: (data: string, flatness = 1e-4) =>
      wasm.flatten_path_svg(data, flatness),
    normalizePath: (data: string) => wasm.normalize_path_svg(data),
    normalizeFillContours: (
      data: string,
      rule: "nonzero" | "evenodd" = "nonzero",
    ) => wasm.normalize_fill_contours_svg(data, rule === "evenodd"),
    dashPath: (data: string, dashArray: number[], dashOffset = 0) =>
      wasm.dash_path_svg(data, dashArray.join(","), dashOffset),
    intersectPaths: (a: string, b: string) =>
      JSON.parse(wasm.intersect_paths_svg(a, b)) as IntersectionPoint[],
    unionPaths: (a: string, b: string) => wasm.boolean_union_svg(a, b),
    intersectPathAreas: (a: string, b: string) =>
      wasm.boolean_intersection_svg(a, b),
    subtractPaths: (a: string, b: string) =>
      wasm.boolean_difference_svg(a, b),
    xorPaths: (a: string, b: string) => wasm.boolean_xor_svg(a, b),
    offsetPath: (
      data: string,
      distance: number,
      options: OffsetOptions = {},
    ) =>
      wasm.offset_path_svg(
        data,
        distance,
        options.join ?? "miter",
        options.miterLimit ?? 4,
      ),
    outlinePath: (
      data: string,
      width: number,
      options: OutlineOptions = {},
    ) => {
      const dashArray = options.dashArray ?? [];
      if (dashArray.length > 0) {
        return wasm.outline_dashed_path_svg(
          data,
          width,
          options.cap ?? "butt",
          options.join ?? "miter",
          options.miterLimit ?? 4,
          dashArray.join(","),
          options.dashOffset ?? 0,
        );
      }
      return wasm.outline_path_svg(
        data,
        width,
        options.cap ?? "butt",
        options.join ?? "miter",
        options.miterLimit ?? 4,
      );
    },
    tessellateFill: (
      data: string,
      rule: "nonzero" | "evenodd" = "nonzero",
      options: TessellationOptions = {},
    ) =>
      JSON.parse(
        wasm.tessellate_fill_svg(
          data,
          rule === "evenodd",
          options.flatness ?? 1e-4,
        ),
      ) as Mesh2D,
    tessellateStroke: (
      data: string,
      width: number,
      options: OutlineOptions & TessellationOptions = {},
    ) => {
      const dashArray = options.dashArray ?? [];
      const strokeData =
        dashArray.length > 0
          ? wasm.dash_path_svg(data, dashArray.join(","), options.dashOffset ?? 0)
          : data;
      return JSON.parse(
        wasm.tessellate_stroke_svg(
          strokeData,
          width,
          options.cap ?? "butt",
          options.join ?? "miter",
          options.miterLimit ?? 4,
          options.flatness ?? 1e-4,
        ),
      ) as Mesh2D;
    },
    cleanupPath: (data: string, options: CleanupOptions = {}) =>
      wasm.cleanup_path_svg(
        data,
        options.pointTolerance ?? 1e-9,
        options.collinearTolerance ?? 1e-7,
      ),
    simplifyPath: (data: string, maxDeviation: number) =>
      wasm.simplify_path_svg(data, maxDeviation),
    fitPathCurves: (data: string, maxError: number) =>
      wasm.fit_path_curves_svg(data, maxError),
  };
}
