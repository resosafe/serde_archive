use super::ArchiveWriter;
use crate::error::Result;
use crate::path::join_path;
use std::collections::HashSet;
use std::io::{Seek, Write};

pub enum ZipCompression {
    None,
    Deflated,
    Zstd,
    Bzip2,
}

impl From<ZipCompression> for zip::CompressionMethod {
    fn from(compression: ZipCompression) -> Self {
        match compression {
            ZipCompression::None => zip::CompressionMethod::Stored,
            ZipCompression::Deflated => zip::CompressionMethod::Deflated,
            ZipCompression::Zstd => zip::CompressionMethod::Zstd,
            ZipCompression::Bzip2 => zip::CompressionMethod::Bzip2,
        }
    }
}

pub struct ZipWriter<W: Write + Seek> {
    writer: zip::ZipWriter<W>,
    compression: zip::CompressionMethod,
    directories: HashSet<String>,
    finished: bool,
}

impl<W> ZipWriter<W>
where
    W: Write + Seek,
{
    pub fn new(writer: W, compression: ZipCompression) -> Self {
        ZipWriter {
            writer: zip::ZipWriter::new(writer),
            compression: compression.into(),
            directories: HashSet::new(),
            finished: false,
        }
    }
}

impl<W> ArchiveWriter for ZipWriter<W>
where
    W: Write + Seek,
{
    fn write(&mut self, path: &[String], data: &[u8]) -> Result<()> {
        if path.len() > 1 {
            let parent = format!("{}/", join_path(&path[..path.len() - 1]));
            if self.directories.insert(parent.clone()) {
                self.writer
                    .add_directory(&parent, Default::default())
                    .map_err(crate::error::Error::from)?;
            }
        }

        let options = zip::write::FileOptions::default().compression_method(self.compression);
        self.writer
            .start_file(join_path(path), options)
            .map_err(crate::error::Error::from)?;
        self.writer.write_all(data)?;
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        self.finished = true;
        self.writer
            .finish()
            .map(|_| ())
            .map_err(crate::error::Error::from)
    }
}

impl<W: Write + Seek> Drop for ZipWriter<W> {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}
