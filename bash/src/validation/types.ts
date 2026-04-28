import type { FsError } from "../errors.js";
import type { PathIndex } from "../path-index.js";

export type WriteIntent = "addDoc" | "moveDoc";

export interface ValidationCtx {
  path: string;
  intent: WriteIntent;
  pathIndex: PathIndex;
}

export type ValidationRule = (ctx: ValidationCtx) => FsError | null;
