export class PathIndex {
  private files: Map<string, string> = new Map();
  private byDocId: Map<string, string> = new Map();
  private syntheticDirs: Set<string> = new Set();

  insert(path: string, docId: string): void {
    const existing = this.files.get(path);
    if (existing && existing !== docId) this.byDocId.delete(existing);
    this.files.set(path, docId);
    this.byDocId.set(docId, path);
  }

  resolve(path: string): string | null {
    return this.files.get(path) ?? null;
  }

  findPath(docId: string): string | null {
    return this.byDocId.get(docId) ?? null;
  }

  remove(path: string): void {
    const docId = this.files.get(path);
    if (docId !== undefined) this.byDocId.delete(docId);
    this.files.delete(path);
  }

  markSyntheticDir(path: string): void {
    if (path === "/" || path === "") return;
    this.syntheticDirs.add(path);
  }

  removeSyntheticDir(path: string): void {
    this.syntheticDirs.delete(path);
  }

  isFile(path: string): boolean {
    return this.files.has(path);
  }

  isDirectory(path: string): boolean {
    if (path === "/" || path === "") return true;
    if (this.syntheticDirs.has(path)) return true;
    const prefix = path.endsWith("/") ? path : `${path}/`;
    for (const f of this.files.keys()) {
      if (f.startsWith(prefix)) return true;
    }
    return false;
  }

  findAncestorFile(path: string): string | null {
    const segments = path.split("/").filter((s) => s !== "");
    for (let i = segments.length - 1; i >= 1; i--) {
      const ancestor = `/${segments.slice(0, i).join("/")}`;
      if (this.files.has(ancestor)) return ancestor;
    }
    return null;
  }

  hasDescendant(path: string): boolean {
    if (path === "/" || path === "") return false;
    const prefix = path.endsWith("/") ? path : `${path}/`;
    for (const f of this.files.keys()) {
      if (f.startsWith(prefix)) return true;
    }
    return false;
  }

  paths(): string[] {
    return Array.from(this.files.keys()).sort();
  }

  syntheticDirPaths(): string[] {
    return Array.from(this.syntheticDirs).sort();
  }

  size(): number {
    return this.files.size;
  }
}
