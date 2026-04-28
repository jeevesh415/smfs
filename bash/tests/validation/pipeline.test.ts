import { describe, expect, it } from "vitest";
import { FsError } from "../../src/errors.js";
import { PathIndex } from "../../src/path-index.js";
import { assertWritable } from "../../src/validation/pipeline.js";

describe("assertWritable", () => {
  it("returns void for a valid path", () => {
    expect(() =>
      assertWritable({
        path: "/notes/foo.md",
        intent: "addDoc",
        pathIndex: new PathIndex(),
      }),
    ).not.toThrow();
  });

  it("throws EINVAL on shape failure (first rule)", () => {
    expect.assertions(2);
    try {
      assertWritable({
        path: "/memory",
        intent: "addDoc",
        pathIndex: new PathIndex(),
      });
    } catch (e) {
      expect(e).toBeInstanceOf(FsError);
      expect((e as FsError).code).toBe("EINVAL");
    }
  });

  it("throws EPERM for reserved path even when shape is valid", () => {
    expect.assertions(1);
    try {
      assertWritable({
        path: "/profile.md",
        intent: "addDoc",
        pathIndex: new PathIndex(),
      });
    } catch (e) {
      expect((e as FsError).code).toBe("EPERM");
    }
  });

  it("throws ENOTDIR when ancestor is a file (rule 3)", () => {
    const idx = new PathIndex();
    idx.insert("/foo.md", "doc-1");
    expect.assertions(1);
    try {
      assertWritable({
        path: "/foo.md/bar.md",
        intent: "addDoc",
        pathIndex: idx,
      });
    } catch (e) {
      expect((e as FsError).code).toBe("ENOTDIR");
    }
  });

  it("throws EISDIR when descendant exists (rule 4)", () => {
    const idx = new PathIndex();
    idx.insert("/foo.md/bar.md", "doc-1");
    expect.assertions(1);
    try {
      assertWritable({
        path: "/foo.md",
        intent: "addDoc",
        pathIndex: idx,
      });
    } catch (e) {
      expect((e as FsError).code).toBe("EISDIR");
    }
  });

  it("first failure wins (shape before reserved)", () => {
    expect.assertions(1);
    try {
      assertWritable({
        path: "",
        intent: "addDoc",
        pathIndex: new PathIndex(),
      });
    } catch (e) {
      expect((e as FsError).code).toBe("EINVAL");
    }
  });
});
