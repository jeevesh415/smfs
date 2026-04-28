export class FsError extends Error {
  constructor(
    public readonly code: string,
    public readonly errno: number,
    message: string,
  ) {
    super(message);
    this.name = "FsError";
  }
}

const make = (code: string, errno: number, suffix: string) =>
  new FsError(code, errno, `${code}: ${suffix}`);

export const enoent = (path: string): FsError =>
  make("ENOENT", -2, `no such file or directory, '${path}'`);

export const eperm = (path: string, op?: string): FsError =>
  make("EPERM", -1, `operation not permitted${op ? `, ${op}` : ""} '${path}'`);

export const eio = (reason: string): FsError => make("EIO", -5, `I/O error, ${reason}`);

export const eisdir = (path: string): FsError => make("EISDIR", -21, `is a directory, '${path}'`);

export const enotdir = (path: string): FsError =>
  make("ENOTDIR", -20, `not a directory, '${path}'`);

export const enotempty = (path: string): FsError =>
  make("ENOTEMPTY", -39, `directory not empty, '${path}'`);

export const eexist = (path: string): FsError =>
  make("EEXIST", -17, `file already exists, '${path}'`);

export const enosys = (op: string): FsError => make("ENOSYS", -38, `function not supported, ${op}`);

export const einval = (reason: string): FsError =>
  make("EINVAL", -22, `invalid argument, ${reason}`);

export const efbig = (path: string): FsError => make("EFBIG", -27, `file too large, '${path}'`);

export const ebusy = (path: string): FsError =>
  make("EBUSY", -16, `resource busy or locked, '${path}'`);

export const enametoolong = (path: string): FsError =>
  make("ENAMETOOLONG", -36, `file name too long, '${path}'`);
