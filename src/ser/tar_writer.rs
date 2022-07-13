use super::ArchiveWriter;
use crate::error::{Error, Result};
use std::io::{Seek, Write};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TarWriter<W: Write>(tar::Builder<W>);

impl<W> TarWriter<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        TarWriter(tar::Builder::new(writer))
    }
}

impl<W> ArchiveWriter for TarWriter<W>
where
    W: Write,
{
    fn write(&mut self, path: &Vec<String>, data: &[u8]) -> super::Result<()> {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(420);
        let start = SystemTime::now();
        let time = start.duration_since(UNIX_EPOCH).unwrap_or_default();
        header.set_mtime(time.as_secs());
        header.set_cksum();

        self.0.append_data(&mut header, path.join("/"), data).unwrap();
        Ok(())
    }

    fn finish(&mut self) -> super::Result<()> {
        self.0.finish().map_or_else(|e| Err(super::Error::Message(e.to_string())), |_| Ok(()))
    }
}
