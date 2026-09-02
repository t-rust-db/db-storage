//! Memory-mapping helper used by [`crate::posix::PosixFile`]. `memmap2` is
//! the one external dependency in this crate — hand-rolling a
//! cross-platform (Unix + Windows) mmap wrapper is not worth the risk of
//! getting page-fault/lifetime edge cases wrong.
//!
//! The OS handles page caching, so this gives random-access reads over
//! files larger than RAM without explicit seeking, and multiple readers
//! can map the same file concurrently (the mapping is read-only).

use std::fs::File;
use std::io;

use crate::vfs::MmapRegion;

/// Memory-map a file for read-only access.
pub fn map_file(file: &File) -> io::Result<MmapRegion> {
    // Safety: mutating the backing file while it is mapped is undefined
    // behavior per the mmap(2)/CreateFileMapping contract. `memmap2`
    // cannot enforce this across processes; the caller must ensure the
    // file is not concurrently written.
    let mmap = unsafe { memmap2::Mmap::map(file)? };
    Ok(MmapRegion::Mapped(mmap))
}
