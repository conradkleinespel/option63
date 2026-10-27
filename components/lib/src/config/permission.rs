//! Filesystem permission checks ensuring configuration files are accessible only by their owner.

use crate::config::ConfigError;
use std::path::Path;

pub fn reject_loose_permissions(path: &Path) -> Result<(), ConfigError> {
    let metadata = std::fs::metadata(path).map_err(|e| ConfigError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    if !is_owner_only(metadata) {
        return Err(ConfigError::InvalidPermissions(path.to_path_buf()));
    }
    Ok(())
}

#[cfg(unix)]
fn is_owner_only(metadata: std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    metadata.mode() & 0o077 == 0
}

#[cfg(not(unix))]
fn is_owner_only(_metadata: std::fs::Metadata) -> bool {
    true
}
