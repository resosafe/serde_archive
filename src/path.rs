use crate::error::{Error, Result};

/// Validate and normalize a single path segment used in archive entries.
pub fn sanitize_segment(segment: &str) -> Result<String> {
    if segment.is_empty() {
        return Err(Error::invalid_path("empty path segment"));
    }
    if segment == "." || segment == ".." {
        return Err(Error::invalid_path(format!("forbidden segment `{segment}`")));
    }
    if segment.contains('/') || segment.contains('\\') {
        return Err(Error::invalid_path(format!(
            "segment must not contain path separators: `{segment}`"
        )));
    }
    Ok(segment.to_owned())
}

pub fn join_path(parts: &[String]) -> String {
    parts.join("/")
}
