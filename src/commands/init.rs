use std::fs;
use std::path::PathBuf;
use crate::error::{KenError, Result};
use crate::storage::Storage;

/// Run the init command - creates .ken directory with ken.db
pub fn run() -> Result<()> {
    let ken_dir = PathBuf::from(".ken");

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

    /// Test version that accepts a specific path
    fn init_at_path(base_path: &std::path::Path) -> Result<()> {
        let ken_dir = base_path.join(".ken");

        if ken_dir.exists() {
            return Err(KenError::AlreadyInitialized);
        }

        std::fs::create_dir(&ken_dir)?;
        let db_path = ken_dir.join("ken.db");
        Storage::create(&db_path)?;
        Ok(())
    }

    #[test]
    fn test_init_creates_ken_directory() {
        let dir = tempdir().unwrap();
        let result = init_at_path(dir.path());

        assert!(result.is_ok());
        assert!(dir.path().join(".ken").exists());
        assert!(dir.path().join(".ken/ken.db").exists());
    }

    #[test]
    fn test_init_fails_if_already_initialized() {
        let dir = tempdir().unwrap();

        // First init should succeed
        let _ = init_at_path(dir.path());

        // Second init should fail
        let result = init_at_path(dir.path());

        assert!(result.is_err());
        match result {
            Err(KenError::AlreadyInitialized) => (),
            _ => panic!("Expected AlreadyInitialized error"),
        }
    }
}
