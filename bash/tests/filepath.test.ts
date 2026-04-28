import { describe, expect, it } from "vitest";
import {
  BASENAME_MAX_BYTES,
  checkFilepath,
  FILEPATH_MAX_BYTES,
  isReservedFilepath,
  isValidFilepath,
} from "../src/filepath.js";

describe("isValidFilepath", () => {
  it.each([
    ["/notes/foo.md", true],
    ["/.gitignore", true],
    ["/.env", true],
    ["/a.b.c.tar.gz", true],
    ["/foo.bar/baz.md", true],
    ["/x/y/z.txt", true],
    ["/profile.md", true],
    ["/memory", false],
    ["/foo", false],
    ["/foo.", false],
    ["/", false],
    ["", false],
    ["foo.md", false],
    ["/foo.md/", false],
    ["//foo.md", false],
    ["/foo//bar.md", false],
    ["/foo\x00bar.md", false],
    ["/foo\x01bar.md", false],
    ["/foo\x7Fbar.md", false],
  ])("isValidFilepath(%j) → %s", (input, expected) => {
    expect(isValidFilepath(input)).toBe(expected);
  });

  it("rejects when total length exceeds FILEPATH_MAX_BYTES", () => {
    const padding = "a".repeat(FILEPATH_MAX_BYTES);
    expect(isValidFilepath(`/${padding}.md`)).toBe(false);
  });

  it("rejects when basename length exceeds BASENAME_MAX_BYTES", () => {
    const stem = "a".repeat(BASENAME_MAX_BYTES);
    expect(isValidFilepath(`/${stem}.md`)).toBe(false);
  });

  it("accepts basename exactly at BASENAME_MAX_BYTES", () => {
    const stem = "a".repeat(BASENAME_MAX_BYTES - 3);
    expect(isValidFilepath(`/${stem}.md`)).toBe(true);
  });
});

describe("checkFilepath returns specific rejection reasons", () => {
  it.each([
    ["/memory", "missing_extension"],
    ["/foo.", "missing_extension"],
    ["/foo.md/", "empty_leaf"],
    ["//foo.md", "double_slash"],
    ["foo.md", "not_absolute"],
    ["", "empty"],
    ["/foo\x00.md", "control_char"],
  ])("checkFilepath(%j) → %s", (input, expected) => {
    expect(checkFilepath(input)).toBe(expected);
  });

  it("returns null for valid input", () => {
    expect(checkFilepath("/notes/foo.md")).toBeNull();
  });
});

describe("isReservedFilepath", () => {
  it("rejects /profile.md", () => {
    expect(isReservedFilepath("/profile.md")).toBe(true);
  });

  it("allows /profile.md.bak", () => {
    expect(isReservedFilepath("/profile.md.bak")).toBe(false);
  });

  it("allows /sub/profile.md (not at root)", () => {
    expect(isReservedFilepath("/sub/profile.md")).toBe(false);
  });

  it("does not match by prefix", () => {
    expect(isReservedFilepath("/profile.markdown")).toBe(false);
  });
});
