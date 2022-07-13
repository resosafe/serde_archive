use super::ArchiveWriter;
use crate::error::{Error, Result};
use std::io::{Seek, Write};



pub enum ZipCompression {
    None,
    Deflated,
    Zstd,
    Aes,
    Bzip2
}


impl From<ZipCompression> for zip::CompressionMethod {
    fn from(compression: ZipCompression) -> Self {
        match compression {
            ZipCompression::None => zip::CompressionMethod::Stored,
            ZipCompression::Deflated => zip::CompressionMethod::Deflated,
            ZipCompression::Zstd => zip::CompressionMethod::Zstd,
            ZipCompression::Aes => zip::CompressionMethod::Aes,
            ZipCompression::Bzip2 => zip::CompressionMethod::Bzip2,
        }
    }
}
pub struct ZipWriter<W: Write + Seek> {
    writer :zip::ZipWriter<W>,
    compression: zip::CompressionMethod,
}

impl<W> ZipWriter<W>
where
    W: Write + Seek,
{
    pub fn new(writer: W, compression: ZipCompression) -> Self {
        ZipWriter {
            writer : zip::ZipWriter::new(writer),
            compression: compression.into(),
        }
    }
}

impl<W> ArchiveWriter for ZipWriter<W>
where
    W: Write + Seek,
{
    fn write(&mut self, path: &Vec<String>, data: &[u8]) -> super::Result<()> {
        let mut parent = path.clone();
        parent.pop();
        if parent.len() > 0 {
            let mut parent = parent.join("/");
            parent.push('/');
            self.writer.add_directory(&parent, Default::default());
        }

        let options = zip::write::FileOptions::default().compression_method(self.compression);
        self.writer.start_file(&path.join("/"), options);
        self.writer.write_all(data);
        Ok(())
    }

    fn finish(&mut self) -> super::Result<()> {
        self.writer.finish().map_or_else(|e| Err(super::Error::Message(e.to_string())), |_| Ok(()))
    }
}
