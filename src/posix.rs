//! Standard-filesystem `Vfs` implementation, backed by `std::fs::File`.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Mutex;

use crate::mmap;
use crate::vfs::{MmapRegion, Vfs, VfsFile};

/// Opens files from the local filesystem.
#[derive(Default)]
pub struct PosixVfs;

impl Vfs for PosixVfs {
    type File = PosixFile;

    fn open(&self, path: &Path) -> io::Result<Self::File> {
        let file = File::open(path)?;
        Ok(PosixFile { file: Mutex::new(file) })
    }
}

/// A file opened from the local filesystem.
///
/// `read_at` needs a cursor seek before reading (std::fs::File has no
/// portable positioned-read that works identically on Unix and Windows),
/// so the underlying `File` is guarded by a `Mutex` to keep `&self` reads
/// safe under concurrent callers.
pub struct PosixFile {
    file: Mutex<File>,
}

impl VfsFile for PosixFile {
    fn size(&self) -> io::Result<u64> {
        self.file.lock().unwrap().metadata().map(|m| m.len())
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        let mut file = self.file.lock().unwrap();
        file.seek(SeekFrom::Start(offset))?;
        file.read(buf)
    }

    fn mmap(&self) -> io::Result<MmapRegion> {
        mmap::map_file(&self.file.lock().unwrap())
    }
}
