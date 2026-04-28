import { ruleAncestorNotFile } from "./rules/ancestor.js";
import { ruleNoDescendants } from "./rules/descendants.js";
import { ruleReserved } from "./rules/reserved.js";
import { ruleShape } from "./rules/shape.js";
import type { ValidationCtx, ValidationRule } from "./types.js";

const writeRules: ValidationRule[] = [
  ruleShape,
  ruleReserved,
  ruleAncestorNotFile,
  ruleNoDescendants,
];

export function assertWritable(ctx: ValidationCtx): void {
  for (const rule of writeRules) {
    const err = rule(ctx);
    if (err) throw err;
  }
}
