export type StrokeCap = "butt" | "round" | "square";
export type StrokeJoin = "miter" | "round" | "bevel";

export interface VectorWasmBindings {
  validate_path(data: string): boolean;
  path_length_svg(data: string): number;
  path_bounds_svg(data: string): string;
  point_at_length_svg(data: string, distance: number): string;
  hit_test_fill_svg(data: string, x: number, y: number, evenOdd: boolean): boolean;
  flatten_path_svg(data: string, flatness: number): string;
  normalize_path_svg(data: string): string;
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

export interface OffsetOptions {
  join?: StrokeJoin;
  miterLimit?: number;
}

export interface OutlineOptions extends OffsetOptions {
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
    flattenPath: (data: string, flatness = 1e-4) =>
      wasm.flatten_path_svg(data, flatness),
    normalizePath: (data: string) => wasm.normalize_path_svg(data),
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
    ) =>
      wasm.outline_path_svg(
        data,
        width,
        options.cap ?? "butt",
        options.join ?? "miter",
        options.miterLimit ?? 4,
      ),
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
    ) =>
      JSON.parse(
        wasm.tessellate_stroke_svg(
          data,
          width,
          options.cap ?? "butt",
          options.join ?? "miter",
          options.miterLimit ?? 4,
          options.flatness ?? 1e-4,
        ),
      ) as Mesh2D,
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
