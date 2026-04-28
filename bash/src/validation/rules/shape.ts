import { einval, enametoolong, type FsError } from "../../errors.js";
import { checkFilepath } from "../../filepath.js";
import type { ValidationRule } from "../types.js";

export const ruleShape: ValidationRule = ({ path }): FsError | null => {
  const reason = checkFilepath(path);
  if (reason === null) return null;
  if (reason === "too_long" || reason === "basename_too_long") {
    return enametoolong(path);
  }
  return einval(`'${path}': ${reason}`);
};
