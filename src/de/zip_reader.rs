use super::Archive;
use crate::error::Result;
use std::io::{Read, Seek};

pub fn read_archive<R: Read + Seek>(reader: R) -> Result<Archive> {
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut archive = Archive::new();
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let name = file.name().to_owned();
        if name.ends_with('/') {
            continue;
        }
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        archive.insert(name, data);
    }
    Ok(archive)
}
