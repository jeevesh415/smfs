export const RESERVED_FILEPATHS: ReadonlySet<string> = new Set(["/profile.md"]);

export const FILEPATH_MAX_BYTES = 4096;
export const BASENAME_MAX_BYTES = 255;

// biome-ignore lint/suspicious/noControlCharactersInRegex: detecting control chars is the purpose
const CONTROL_CHARS = /[\x00-\x1F\x7F]/;
const VALID_BASENAME = /^\.?[^/]*\.[^/.]+$/;

export type FilepathRejection =
  | "not_string"
  | "empty"
  | "too_long"
  | "not_absolute"
  | "control_char"
  | "double_slash"
  | "empty_leaf"
  | "basename_too_long"
  | "missing_extension";

export function checkFilepath(value: string): FilepathRejection | null {
  if (typeof value !== "string") return "not_string";
  if (value.length === 0) return "empty";
  if (value.length > FILEPATH_MAX_BYTES) return "too_long";
  if (!value.startsWith("/")) return "not_absolute";
  if (CONTROL_CHARS.test(value)) return "control_char";
  if (value.includes("//")) return "double_slash";
  const segments = value.split("/").slice(1);
  if (segments.length === 0) return "empty_leaf";
  const basename = segments[segments.length - 1];
  if (basename === undefined || basename === "") return "empty_leaf";
  if (basename.length > BASENAME_MAX_BYTES) return "basename_too_long";
  if (!VALID_BASENAME.test(basename)) return "missing_extension";
  return null;
}

export function isValidFilepath(value: string): boolean {
  return checkFilepath(value) === null;
}

export function isReservedFilepath(value: string): boolean {
  return RESERVED_FILEPATHS.has(value);
}
