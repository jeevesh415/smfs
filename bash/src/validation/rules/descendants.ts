import { eisdir, type FsError } from "../../errors.js";
import type { ValidationRule } from "../types.js";

export const ruleNoDescendants: ValidationRule = ({ path, pathIndex }): FsError | null => {
  if (pathIndex.hasDescendant(path)) return eisdir(path);
  return null;
};
