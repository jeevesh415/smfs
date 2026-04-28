import { eperm, type FsError } from "../../errors.js";
import { isReservedFilepath } from "../../filepath.js";
import type { ValidationRule } from "../types.js";

export const ruleReserved: ValidationRule = ({ path, intent }): FsError | null => {
  if (isReservedFilepath(path)) return eperm(path, intent);
  return null;
};
