# db-storage

A small `Vfs` / `VfsFile` abstraction shared across the t-rust-db engines
(sqlite-rs, column-rs, loglume). Engines open and read files through these
traits instead of `std::fs` directly, so the actual storage backend can be
swapped without touching engine code.

## Why

`column-rs` originally had its own private `mmap.rs` wrapping `memmap2`.
That's fine for one engine, but sqlite-rs needs the same "map a file
read-only, or read it at an offset" capability, and both engines need a
way to test against fake storage without touching disk. `db-storage`
pulls that into a shared crate and adds the seam a real VFS needs: a
trait, not a concrete file type.

## What's here

- `vfs.rs` — the `Vfs` and `VfsFile` traits, plus `MmapRegion`, a
  read-only byte view that's either an OS mmap or an owned in-memory
  buffer.
- `posix.rs` — `PosixVfs` / `PosixFile`, backed by `std::fs::File`.
- `mmap.rs` — the `memmap2` call, isolated to one function. This is the
  only place in the crate using `unsafe` (required by `memmap2`'s
  `Mmap::map` contract).
- `memory.rs` — `MemoryVfs` / `MemoryFile`, an in-memory backend for
  tests: no filesystem involved, so callers can seed exact byte layouts.

## Usage

```rust
use db_storage::{PosixVfs, Vfs, VfsFile};

let vfs = PosixVfs;
let file = vfs.open(std::path::Path::new("data.parquet"))?;
let mapped = file.mmap()?; // Deref's to &[u8]
```

Swap in `MemoryVfs` in tests to avoid touching disk:

```rust
use db_storage::MemoryVfs;

let vfs = MemoryVfs::new();
vfs.insert("data.parquet", some_bytes);
let file = vfs.open(std::path::Path::new("data.parquet"))?;
```

## How engines use it

- **column-rs** replaces its private `src/mmap.rs` with `db-storage`'s
  `PosixVfs`, and hands the mapped bytes to its own `ParquetFile::open`.
- **sqlite-rs** opens its page file through the same `Vfs` trait; a
  future S3-backed or copy-on-write `Vfs` implementation plugs in without
  either engine changing.
