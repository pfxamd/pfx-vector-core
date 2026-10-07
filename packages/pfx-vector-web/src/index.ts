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
  };
}
