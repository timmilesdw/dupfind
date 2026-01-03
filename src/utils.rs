use anyhow::{Result, bail};
use std::path::Path;
use std::sync::atomic::AtomicBool;

pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);

pub fn validate_path(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("Path does not exist: {}", path.display());
    }
    if !path.is_dir() {
        bail!("Path is not a directory: {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_validate_path_existing_dir() {
        let dir = tempdir().unwrap();
        assert!(validate_path(dir.path()).is_ok());
    }

    #[test]
    fn test_validate_path_nonexistent() {
        let path = Path::new("/nonexistent/path/that/does/not/exist");
        let result = validate_path(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_validate_path_file_not_dir() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        fs::write(&file_path, "test").unwrap();

        let result = validate_path(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }
}
