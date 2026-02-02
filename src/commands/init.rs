use std::fs;
use std::path::Path;
use crate::error::{KenError, Result};
use crate::storage::Storage;

/// Run the init command - creates .ken directory with ken.db in current directory
pub fn run() -> Result<()> {
    run_at_path(Path::new("."))
}

/// Initialize ken at a specific path (for testing and flexibility)
pub fn run_at_path(base_path: &Path) -> Result<()> {
    let ken_dir = base_path.join(".ken");

    if ken_dir.exists() {
        return Err(KenError::AlreadyInitialized);
    }

    // Create .ken directory
    fs::create_dir(&ken_dir)?;

    // Create database with schema
    let db_path = ken_dir.join("ken.db");
    Storage::create(&db_path)?;

    println!("Initialized ken in .ken/");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init_creates_ken_directory() {
        let dir = tempdir().unwrap();
        let result = run_at_path(dir.path());

        assert!(result.is_ok());
        assert!(dir.path().join(".ken").exists());
        assert!(dir.path().join(".ken/ken.db").exists());
    }

    #[test]
    fn test_init_fails_if_already_initialized() {
        let dir = tempdir().unwrap();

        // First init should succeed
        let _ = run_at_path(dir.path());

        // Second init should fail
        let result = run_at_path(dir.path());

        assert!(result.is_err());
        match result {
            Err(KenError::AlreadyInitialized) => (),
            _ => panic!("Expected AlreadyInitialized error"),
        }
    }
}
