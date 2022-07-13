use super::ArchiveWriter;
use crate::error::{Error, Result};
use std::io::{Seek, Write};

pub struct ZipWriter<W: Write + Seek>(zip::ZipWriter<W>);

impl<W> ZipWriter<W>
where
    W: Write + Seek,
{
    pub fn new(writer: W) -> Self {
        ZipWriter(zip::ZipWriter::new(writer))
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
            self.0.add_directory(&parent, Default::default());
        }

        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        self.0.start_file(&path.join("/"), options);
        self.0.write_all(data);
        Ok(())
    }

    fn finish(&mut self) -> super::Result<()> {
        self.0.finish().map_or_else(|e| Err(super::Error::Message(e.to_string())), |_| Ok(()))
    }
}
