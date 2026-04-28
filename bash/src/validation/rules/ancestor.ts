import { enotdir, type FsError } from "../../errors.js";
import type { ValidationRule } from "../types.js";

export const ruleAncestorNotFile: ValidationRule = ({ path, pathIndex }): FsError | null => {
  const ancestor = pathIndex.findAncestorFile(path);
  if (ancestor) return enotdir(ancestor);
  return null;
};
