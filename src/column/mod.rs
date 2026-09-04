//! Column-oriented storage: the mmap-based `Vfs`/`VfsFile` pair plus the
//! Parquet reader that consumes it. Folded together into one module
//! (`db-storage#4`) so `column-rs` depends on a single crate instead of
//! two separately-versioned ones.

pub mod memory;
pub mod mmap;
pub mod parquet;
pub mod posix;
pub mod vfs;

pub use memory::MemoryVfs;
pub use posix::PosixVfs;
pub use vfs::{MmapRegion, Vfs, VfsFile};

pub use parquet::{DictionaryIndices, FileError, ParquetFile, RowGroupReader};
