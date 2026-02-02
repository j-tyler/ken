use crate::error::Result;
use crate::session::SessionStatus;
use crate::storage::{open_storage, Storage};

/// Run the status command - show current session status
pub fn run() -> Result<()> {
    let storage = open_storage()?;
    run_with_storage(&storage)
}

/// Status command implementation with injected storage (for testing)
pub fn run_with_storage(storage: &Storage) -> Result<()> {
    let sessions = storage.get_all_sessions()?;

    if sessions.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }

    // Count by status
    let mut pending = 0;
    let mut active = 0;
    let mut sleeping = 0;
    let mut complete = 0;
    let mut failed = 0;

    for session in &sessions {
        match session.status {
            SessionStatus::Pending => pending += 1,
            SessionStatus::Active => active += 1,
            SessionStatus::Sleeping => sleeping += 1,
            SessionStatus::Complete => complete += 1,
            SessionStatus::Failed => failed += 1,
        }
    }

    println!("Sessions: {} total", sessions.len());
    println!("  Pending:   {}", pending);
    println!("  Active:    {}", active);
    println!("  Sleeping:  {}", sleeping);
    println!("  Complete:  {}", complete);
    println!("  Failed:    {}", failed);
    println!();

    // Show active sessions
    let active_sessions: Vec<_> = sessions.iter()
        .filter(|s| s.status == SessionStatus::Active)
        .collect();

    if !active_sessions.is_empty() {
        println!("Active sessions:");
        for session in active_sessions {
            println!("  {} - {} ({})", session.id, session.task, session.ken);
        }
        println!();
    }

    // Show pending sessions
    let pending_sessions: Vec<_> = sessions.iter()
        .filter(|s| s.status == SessionStatus::Pending)
        .collect();

    if !pending_sessions.is_empty() {
        println!("Pending sessions:");
        for session in pending_sessions {
            println!("  {} - {} ({})", session.id, session.task, session.ken);
        }
        println!();
    }

    // Show sleeping sessions
    let sleeping_sessions: Vec<_> = sessions.iter()
        .filter(|s| s.status == SessionStatus::Sleeping)
        .collect();

    if !sleeping_sessions.is_empty() {
        println!("Sleeping sessions:");
        for session in sleeping_sessions {
            println!("  {} - {} ({})", session.id, session.task, session.ken);
            if let Some(trigger) = &session.trigger {
                println!("    trigger: {}", trigger);
            }
        }
        println!();
    }

    // Show failed sessions (important for debugging)
    let failed_sessions: Vec<_> = sessions.iter()
        .filter(|s| s.status == SessionStatus::Failed)
        .collect();

    if !failed_sessions.is_empty() {
        println!("Failed sessions:");
        for session in failed_sessions {
            println!("  {} - {} ({})", session.id, session.task, session.ken);
            if let Some(result) = &session.result {
                println!("    error: {}", result);
            }
        }
    }

    Ok(())
}

/// Get status as JSON with injected storage (for testing)
pub fn run_json_with_storage(storage: &Storage) -> Result<()> {
    let sessions = storage.get_all_sessions()?;

    let output = serde_json::json!({
        "sessions": sessions.iter().map(|s| {
            serde_json::json!({
                "id": s.id,
                "ken": s.ken,
                "task": s.task,
                "status": s.status.as_str(),
                "parent_id": s.parent_id,
                "checkpoint": s.checkpoint,
                "result": s.result,
            })
        }).collect::<Vec<_>>()
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Session;
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
    fn test_status_with_no_sessions() {
        let (_dir, storage) = create_test_storage();

        let result = run_with_storage(&storage);

        assert!(result.is_ok());
    }

    #[test]
    fn test_status_with_sessions() {
        let (_dir, storage) = create_test_storage();

        // Create sessions in different states
        let s1 = Session::with_id("pending-1", "ken1", "task1", None);
        storage.insert_session(&s1).unwrap();

        let mut s2 = Session::with_id("active-1", "ken2", "task2", None);
        s2.status = SessionStatus::Active;
        storage.insert_session(&s2).unwrap();

        let mut s3 = Session::with_id("complete-1", "ken3", "task3", None);
        s3.status = SessionStatus::Complete;
        storage.insert_session(&s3).unwrap();

        let result = run_with_storage(&storage);

        assert!(result.is_ok());
    }

    #[test]
    fn test_status_json() {
        let (_dir, storage) = create_test_storage();

        let session = Session::with_id("test-1", "test/ken", "test task", None);
        storage.insert_session(&session).unwrap();

        let result = run_json_with_storage(&storage);

        assert!(result.is_ok());
    }

    #[test]
    fn test_status_counts_correctly() {
        let (_dir, storage) = create_test_storage();

        // Create 2 pending, 1 active, 1 complete
        let s1 = Session::with_id("pending-1", "ken1", "task1", None);
        storage.insert_session(&s1).unwrap();

        let s2 = Session::with_id("pending-2", "ken2", "task2", None);
        storage.insert_session(&s2).unwrap();

        let mut s3 = Session::with_id("active-1", "ken3", "task3", None);
        s3.status = SessionStatus::Active;
        storage.insert_session(&s3).unwrap();

        let mut s4 = Session::with_id("complete-1", "ken4", "task4", None);
        s4.status = SessionStatus::Complete;
        storage.insert_session(&s4).unwrap();

        let result = run_with_storage(&storage);

        assert!(result.is_ok());
        // Output is printed to stdout, test just verifies no errors
    }
}
