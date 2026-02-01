use crate::error::Result;
use crate::session::{Session, Event};
use crate::storage::{open_storage, Storage};

/// Run the wake command - creates a new session and starts an agent
pub fn run(ken: &str, task: &str) -> Result<()> {
    let storage = open_storage()?;
    run_with_storage(&storage, ken, task)
}

/// Wake command implementation that accepts a storage instance (for testing)
pub fn run_with_storage(storage: &Storage, ken: &str, task: &str) -> Result<()> {
    // Create new session
    let mut session = Session::new(ken, task, None);
    session.status = crate::session::SessionStatus::Pending;

    storage.insert_session(&session)?;

    // Log event
    let event = Event::new("session_created", Some(&session.id), None);
    storage.insert_event(&event)?;

    println!("{}", session.id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_storage() -> (tempfile::TempDir, Storage) {
        let dir = tempdir().unwrap();
        let ken_dir = dir.path().join(".ken");
        std::fs::create_dir(&ken_dir).unwrap();
        let db_path = ken_dir.join("ken.db");
        let storage = Storage::create(&db_path).unwrap();
        (dir, storage)
    }

    #[test]
    fn test_wake_creates_session() {
        let (_dir, storage) = create_test_storage();

        let result = run_with_storage(&storage, "test/ken", "do something");

        assert!(result.is_ok());

        // Verify session was created
        let sessions = storage.get_all_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].ken, "test/ken");
        assert_eq!(sessions[0].task, "do something");
    }

    #[test]
    fn test_wake_creates_pending_session() {
        let (_dir, storage) = create_test_storage();

        run_with_storage(&storage, "test/ken", "do something").unwrap();

        let sessions = storage.get_all_sessions().unwrap();
        assert_eq!(sessions[0].status, crate::session::SessionStatus::Pending);
    }
}
