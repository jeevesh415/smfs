import { describe, expect, it } from "vitest";
import { FsError } from "../../src/errors.js";
import { PathIndex } from "../../src/path-index.js";
import { ruleAncestorNotFile } from "../../src/validation/rules/ancestor.js";
import { ruleNoDescendants } from "../../src/validation/rules/descendants.js";
import { ruleReserved } from "../../src/validation/rules/reserved.js";
import { ruleShape } from "../../src/validation/rules/shape.js";
import type { ValidationCtx } from "../../src/validation/types.js";

const ctx = (path: string, pathIndex = new PathIndex()): ValidationCtx => ({
  path,
  intent: "addDoc",
  pathIndex,
});

describe("ruleShape", () => {
  it("returns null for valid path", () => {
    expect(ruleShape(ctx("/foo.md"))).toBeNull();
  });

  it("returns EINVAL for missing extension", () => {
    const err = ruleShape(ctx("/memory"));
    expect(err).toBeInstanceOf(FsError);
    expect((err as FsError).code).toBe("EINVAL");
  });

  it("returns ENAMETOOLONG when path exceeds cap", () => {
    const err = ruleShape(ctx(`/${"a".repeat(5000)}.md`));
    expect((err as FsError).code).toBe("ENAMETOOLONG");
  });

  it("returns ENAMETOOLONG when basename exceeds cap", () => {
    const err = ruleShape(ctx(`/${"a".repeat(300)}.md`));
    expect((err as FsError).code).toBe("ENAMETOOLONG");
  });
});

describe("ruleReserved", () => {
  it("returns null for non-reserved", () => {
    expect(ruleReserved(ctx("/notes/foo.md"))).toBeNull();
  });

  it("returns EPERM for /profile.md", () => {
    const err = ruleReserved(ctx("/profile.md"));
    expect((err as FsError).code).toBe("EPERM");
  });

  it("does not block subpath /sub/profile.md", () => {
    expect(ruleReserved(ctx("/sub/profile.md"))).toBeNull();
  });
});

describe("ruleAncestorNotFile", () => {
  it("returns null when no ancestor is a file", () => {
    expect(ruleAncestorNotFile(ctx("/notes/foo.md"))).toBeNull();
  });

  it("returns ENOTDIR when an ancestor path is a known file", () => {
    const idx = new PathIndex();
    idx.insert("/foo.md", "doc-1");
    const err = ruleAncestorNotFile(ctx("/foo.md/bar.md", idx));
    expect((err as FsError).code).toBe("ENOTDIR");
    expect((err as FsError).message).toContain("/foo.md");
  });

  it("walks multiple levels", () => {
    const idx = new PathIndex();
    idx.insert("/a/b.md", "doc-1");
    const err = ruleAncestorNotFile(ctx("/a/b.md/c/d.md", idx));
    expect((err as FsError).code).toBe("ENOTDIR");
  });
});

describe("ruleNoDescendants", () => {
  it("returns null when nothing exists under the path", () => {
    expect(ruleNoDescendants(ctx("/foo.md"))).toBeNull();
  });

  it("returns EISDIR when a descendant doc exists", () => {
    const idx = new PathIndex();
    idx.insert("/foo.md/bar.md", "doc-1");
    const err = ruleNoDescendants(ctx("/foo.md", idx));
    expect((err as FsError).code).toBe("EISDIR");
  });

  it("does not match unrelated prefixes", () => {
    const idx = new PathIndex();
    idx.insert("/foo.markdown", "doc-1");
    expect(ruleNoDescendants(ctx("/foo.md", idx))).toBeNull();
  });
});
