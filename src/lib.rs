//! `db-storage`: a small VFS abstraction (`Vfs` / `VfsFile`) shared across
//! the t-rust-db engines, so storage backends can be swapped without
//! touching engine code — a real filesystem in production, an in-memory
//! VFS in tests, and later something like S3.
//!
//! `src/mmap.rs` is the only place in this crate that uses `unsafe`; it
//! is required by `memmap2`'s `Mmap::map` contract and cannot be avoided
//! without hand-rolling a cross-platform mmap wrapper. Everything else in
//! this crate is safe code.

pub mod memory;
pub mod mmap;
pub mod posix;
pub mod vfs;

pub use memory::MemoryVfs;
pub use posix::PosixVfs;
pub use vfs::{MmapRegion, Vfs, VfsFile};

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn maps_and_reads_small_file() {
        let tmp = tempfile_with_contents(b"hello mmap world");
        let vfs = PosixVfs;

        let file = vfs.open(tmp.path()).unwrap();
        assert_eq!(file.size().unwrap(), 16);
        let mapped = file.mmap().unwrap();
        assert_eq!(&*mapped, b"hello mmap world");
    }

    #[test]
    fn maps_empty_file() {
        let tmp = tempfile_with_contents(b"");
        let vfs = PosixVfs;
        let file = vfs.open(tmp.path()).unwrap();
        assert_eq!(file.size().unwrap(), 0);
        assert!(file.mmap().unwrap().is_empty());
    }

    #[test]
    fn concurrent_reads_from_same_file() {
        let tmp = tempfile_with_contents(b"shared contents");
        let vfs = PosixVfs;

        let a = vfs.open(tmp.path()).unwrap();
        let b = vfs.open(tmp.path()).unwrap();
        assert_eq!(&*a.mmap().unwrap(), &*b.mmap().unwrap());
    }

    #[test]
    fn missing_file_errors_not_panics() {
        let vfs = PosixVfs;
        let result = vfs.open(std::path::Path::new("/nonexistent/path/does-not-exist.bin"));
        assert!(result.is_err());
    }

    #[test]
    fn read_at_reads_from_offset() {
        let tmp = tempfile_with_contents(b"0123456789");
        let vfs = PosixVfs;
        let file = vfs.open(tmp.path()).unwrap();

        let mut buf = [0u8; 4];
        let n = file.read_at(3, &mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(&buf, b"3456");
    }

    /// Minimal std-only temp file helper (no external test dependency).
    struct TempFile {
        path: std::path::PathBuf,
    }

    impl TempFile {
        fn path(&self) -> &std::path::Path {
            &self.path
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    fn tempfile_with_contents(contents: &[u8]) -> TempFile {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "db-storage-test-{}-{}.tmp",
            std::process::id(),
            fastrand_like()
        );
        path.push(unique);
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(contents).unwrap();
        file.flush().unwrap();
        TempFile { path }
    }

    /// A tiny non-cryptographic nonce so parallel tests don't collide on
    /// the same temp file name (no external RNG dependency).
    fn fastrand_like() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let addr = &nanos as *const _ as u64;
        (nanos as u64).wrapping_mul(2654435761).wrapping_add(addr)
    }
}
