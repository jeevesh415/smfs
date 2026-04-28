import { describe, expect, it } from "vitest";
import { PathIndex } from "../src/path-index.js";

describe("PathIndex.findPath (reverse lookup)", () => {
  it("returns the path for a docId after insert; null after remove", () => {
    const idx = new PathIndex();
    idx.insert("/a.md", "doc-a");
    idx.insert("/b.md", "doc-b");
    expect(idx.findPath("doc-a")).toBe("/a.md");
    expect(idx.findPath("doc-b")).toBe("/b.md");
    expect(idx.findPath("doc-missing")).toBeNull();
    idx.remove("/a.md");
    expect(idx.findPath("doc-a")).toBeNull();
  });

  it("re-inserting the same path with a different docId clears the old reverse mapping", () => {
    const idx = new PathIndex();
    idx.insert("/a.md", "doc-old");
    idx.insert("/a.md", "doc-new");
    expect(idx.findPath("doc-old")).toBeNull();
    expect(idx.findPath("doc-new")).toBe("/a.md");
  });
});

describe("PathIndex", () => {
  it("resolve on empty index returns null", () => {
    const idx = new PathIndex();
    expect(idx.resolve("/a.md")).toBeNull();
  });

  it("insert + resolve returns the docId", () => {
    const idx = new PathIndex();
    idx.insert("/a.md", "doc1");
    expect(idx.resolve("/a.md")).toBe("doc1");
  });

  it("inserting the same path overwrites the docId", () => {
    const idx = new PathIndex();
    idx.insert("/a.md", "doc1");
    idx.insert("/a.md", "doc2");
    expect(idx.resolve("/a.md")).toBe("doc2");
  });

  it("remove drops the mapping", () => {
    const idx = new PathIndex();
    idx.insert("/a.md", "doc1");
    idx.remove("/a.md");
    expect(idx.resolve("/a.md")).toBeNull();
  });

  it("remove on non-existent path is a no-op", () => {
    const idx = new PathIndex();
    expect(() => idx.remove("/missing.md")).not.toThrow();
  });

  it("multiple inserts each resolve independently", () => {
    const idx = new PathIndex();
    idx.insert("/a.md", "doc1");
    idx.insert("/b.md", "doc2");
    idx.insert("/c.md", "doc3");
    expect(idx.resolve("/a.md")).toBe("doc1");
    expect(idx.resolve("/b.md")).toBe("doc2");
    expect(idx.resolve("/c.md")).toBe("doc3");
  });

  it("paths() on empty index returns empty array", () => {
    const idx = new PathIndex();
    expect(idx.paths()).toEqual([]);
  });

  it("paths() returns inserted paths in sorted order", () => {
    const idx = new PathIndex();
    idx.insert("/c.md", "doc3");
    idx.insert("/a.md", "doc1");
    idx.insert("/b.md", "doc2");
    expect(idx.paths()).toEqual(["/a.md", "/b.md", "/c.md"]);
  });

  it("markSyntheticDir makes isDirectory true for that path", () => {
    const idx = new PathIndex();
    idx.markSyntheticDir("/foo");
    expect(idx.isDirectory("/foo")).toBe(true);
  });

  it("inserting a child path makes the parent isDirectory true", () => {
    const idx = new PathIndex();
    idx.insert("/foo/bar.md", "doc1");
    expect(idx.isDirectory("/foo")).toBe(true);
  });

  it("isDirectory('/') is always true", () => {
    const idx = new PathIndex();
    expect(idx.isDirectory("/")).toBe(true);
  });

  it("isDirectory on an unknown path returns false", () => {
    const idx = new PathIndex();
    expect(idx.isDirectory("/nonexistent")).toBe(false);
  });

  it("isFile is true after insert, false after remove", () => {
    const idx = new PathIndex();
    expect(idx.isFile("/a.md")).toBe(false);
    idx.insert("/a.md", "doc1");
    expect(idx.isFile("/a.md")).toBe(true);
    idx.remove("/a.md");
    expect(idx.isFile("/a.md")).toBe(false);
  });

  it("isFile on a synthetic-only directory path returns false", () => {
    const idx = new PathIndex();
    idx.markSyntheticDir("/foo");
    expect(idx.isFile("/foo")).toBe(false);
    expect(idx.isDirectory("/foo")).toBe(true);
  });

  it("size() reports the count of inserted files (not directories)", () => {
    const idx = new PathIndex();
    expect(idx.size()).toBe(0);
    idx.insert("/a.md", "doc1");
    idx.insert("/b.md", "doc2");
    idx.markSyntheticDir("/dir");
    expect(idx.size()).toBe(2);
    idx.remove("/a.md");
    expect(idx.size()).toBe(1);
  });

  it("findAncestorFile walks up and returns first known file", () => {
    const idx = new PathIndex();
    idx.insert("/a/b.md", "doc1");
    expect(idx.findAncestorFile("/a/b.md/c.md")).toBe("/a/b.md");
    expect(idx.findAncestorFile("/a/b.md/c/d.md")).toBe("/a/b.md");
  });

  it("findAncestorFile returns null when no ancestor is a file", () => {
    const idx = new PathIndex();
    idx.insert("/a/b.md", "doc1");
    expect(idx.findAncestorFile("/a/c.md")).toBeNull();
    expect(idx.findAncestorFile("/x.md")).toBeNull();
  });

  it("findAncestorFile does not return the path itself", () => {
    const idx = new PathIndex();
    idx.insert("/a.md", "doc1");
    expect(idx.findAncestorFile("/a.md")).toBeNull();
  });

  it("hasDescendant returns true when a doc exists under prefix", () => {
    const idx = new PathIndex();
    idx.insert("/foo.md/bar.md", "doc1");
    expect(idx.hasDescendant("/foo.md")).toBe(true);
  });

  it("hasDescendant returns false when no descendants", () => {
    const idx = new PathIndex();
    idx.insert("/foo.md", "doc1");
    expect(idx.hasDescendant("/foo.md")).toBe(false);
    expect(idx.hasDescendant("/bar")).toBe(false);
  });

  it("hasDescendant does not match by partial prefix (trailing slash)", () => {
    const idx = new PathIndex();
    idx.insert("/foo.markdown", "doc1");
    expect(idx.hasDescendant("/foo.md")).toBe(false);
  });
});
