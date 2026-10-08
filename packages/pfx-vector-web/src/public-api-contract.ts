import { createVectorCore } from "./src/index.js";

// Compile-time public API contract: if a wrapper name or argument changes,
// this file fails the strict TypeScript typecheck before release.
type Api = ReturnType<typeof createVectorCore>;
type Require<K extends keyof Api> = Api[K];
type Contract = [
  Require<"validatePath">,
  Require<"pathLength">,
  Require<"pointAtLength">,
  Require<"pointsAtLengths">,
  Require<"frameAtLength">,
  Require<"framesAtLengths">,
  Require<"pathDistanceAt">,
  Require<"pathBounds">,
  Require<"flattenPath">,
  Require<"intersectPaths">,
  Require<"unionPaths">,
  Require<"intersectPathAreas">,
  Require<"subtractPaths">,
  Require<"xorPaths">,
  Require<"transformPath">,
  Require<"sliceContour">
];
export type PublicApiContract = Contract;
