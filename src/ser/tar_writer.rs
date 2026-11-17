use super::ArchiveWriter;
use crate::error::Result;
use crate::path::join_path;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TarWriter<W: Write> {
    builder: tar::Builder<W>,
    finished: bool,
}

impl<W> TarWriter<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        TarWriter {
            builder: tar::Builder::new(writer),
            finished: false,
        }
    }
}

impl<W> ArchiveWriter for TarWriter<W>
where
    W: Write,
{
    fn write(&mut self, path: &[String], data: &[u8]) -> Result<()> {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        header.set_mtime(time.as_secs());
        header.set_cksum();

        self.builder
            .append_data(&mut header, join_path(path), data)?;
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        self.finished = true;
        self.builder.finish()?;
        Ok(())
    }
}

impl<W: Write> Drop for TarWriter<W> {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}
