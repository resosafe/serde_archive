use super::Archive;
use crate::error::Result;
use std::io::Read;

pub fn read_archive<R: Read>(reader: R) -> Result<Archive> {
    let mut archive_in = tar::Archive::new(reader);
    let mut archive = Archive::new();
    for entry in archive_in.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_string_lossy().into_owned();
        if path.ends_with('/') {
            continue;
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        archive.insert(path, data);
    }
    Ok(archive)
}
