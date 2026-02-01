use chrono::Utc;
use crate::error::Result;
use crate::session::{SessionStatus, Trigger, Event};
use crate::storage::{open_storage, Storage};

/// Run the process command - evaluate triggers and activate one pending session
pub fn run() -> Result<()> {
    let storage = open_storage()?;
    run_with_storage(&storage)
}

/// Process command implementation with injected storage (for testing)
pub fn run_with_storage(storage: &Storage) -> Result<()> {
    let now = Utc::now().to_rfc3339();

    // First, check sleeping sessions for satisfied triggers
    let sleeping = storage.get_sessions_by_status(SessionStatus::Sleeping)?;
    for session in sleeping {
        if let Some(trigger_json) = &session.trigger {
            if let Ok(trigger) = Trigger::from_json(trigger_json) {
                let satisfied = trigger.is_satisfied(|id| {
                    storage.get_session(id).ok().map(|s| s.status)
                });

                if satisfied {
                    // Wake this session
                    storage.update_session_status(&session.id, SessionStatus::Pending, &now)?;
                    storage.insert_event(&Event::new(
                        "trigger_satisfied",
                        Some(&session.id),
                        session.trigger.clone(),
                    ))?;
                    println!("Woke session {} (trigger satisfied)", session.id);
                }
            }
        }
    }

    // Then, find one pending session to activate
    let pending = storage.get_sessions_by_status(SessionStatus::Pending)?;
    if let Some(session) = pending.first() {
        storage.update_session_status(&session.id, SessionStatus::Active, &now)?;
        storage.insert_event(&Event::new(
            "session_activated",
            Some(&session.id),
            None,
        ))?;

        // Output session info for the caller to spawn an agent
        let output = serde_json::json!({
            "action": "spawn",
            "session": {
                "id": session.id,
                "ken": session.ken,
                "task": session.task,
                "checkpoint": session.checkpoint,
            }
        });
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!("{{\"action\":\"none\"}}");
    }

    Ok(())
}

/// Check if any work is pending or active (with injected storage)
pub fn has_work_with_storage(storage: &Storage) -> Result<bool> {
    let pending = storage.get_sessions_by_status(SessionStatus::Pending)?;
    let active = storage.get_sessions_by_status(SessionStatus::Active)?;
    let sleeping = storage.get_sessions_by_status(SessionStatus::Sleeping)?;

    Ok(!pending.is_empty() || !active.is_empty() || !sleeping.is_empty())
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
    fn test_process_activates_pending_session() {
        let (_dir, storage) = create_test_storage();

        // Create a pending session
        let session = Session::with_id("test-123", "test/ken", "do something", None);
        storage.insert_session(&session).unwrap();

        let result = run_with_storage(&storage);

        assert!(result.is_ok());

        // Verify session is now active
        let updated = storage.get_session("test-123").unwrap();
        assert_eq!(updated.status, SessionStatus::Active);
    }

    #[test]
    fn test_process_wakes_sleeping_session_when_trigger_satisfied() {
        let (_dir, storage) = create_test_storage();

        // Create a completed child session
        let mut child = Session::with_id("child-1", "child/ken", "child task", None);
        child.status = SessionStatus::Complete;
        storage.insert_session(&child).unwrap();

        // Create a sleeping parent waiting for the child
        let mut parent = Session::with_id("parent-1", "parent/ken", "parent task", None);
        parent.status = SessionStatus::Sleeping;
        parent.trigger = Some(r#"{"all_complete":["child-1"]}"#.to_string());
        parent.checkpoint = Some("saved state".to_string());
        storage.insert_session(&parent).unwrap();

        let result = run_with_storage(&storage);

        assert!(result.is_ok());

        // Verify parent is now pending (woken by trigger), then activated
        let updated = storage.get_session("parent-1").unwrap();
        // Note: run_with_storage will wake (set to pending) then activate in same call
        assert_eq!(updated.status, SessionStatus::Active);
    }

    #[test]
    fn test_process_does_not_wake_if_trigger_not_satisfied() {
        let (_dir, storage) = create_test_storage();

        // Create an active child session (not complete)
        let mut child = Session::with_id("child-1", "child/ken", "child task", None);
        child.status = SessionStatus::Active;
        storage.insert_session(&child).unwrap();

        // Create a sleeping parent waiting for the child
        let mut parent = Session::with_id("parent-1", "parent/ken", "parent task", None);
        parent.status = SessionStatus::Sleeping;
        parent.trigger = Some(r#"{"all_complete":["child-1"]}"#.to_string());
        storage.insert_session(&parent).unwrap();

        let result = run_with_storage(&storage);

        assert!(result.is_ok());

        // Verify parent is still sleeping
        let updated = storage.get_session("parent-1").unwrap();
        assert_eq!(updated.status, SessionStatus::Sleeping);
    }

    #[test]
    fn test_has_work_returns_true_with_pending() {
        let (_dir, storage) = create_test_storage();

        let session = Session::with_id("test-123", "test/ken", "task", None);
        storage.insert_session(&session).unwrap();

        let result = has_work_with_storage(&storage).unwrap();

        assert!(result);
    }

    #[test]
    fn test_has_work_returns_false_when_all_complete() {
        let (_dir, storage) = create_test_storage();

        let mut session = Session::with_id("test-123", "test/ken", "task", None);
        session.status = SessionStatus::Complete;
        storage.insert_session(&session).unwrap();

        let result = has_work_with_storage(&storage).unwrap();

        assert!(!result);
    }

    #[test]
    fn test_process_outputs_none_when_no_work() {
        let (_dir, storage) = create_test_storage();

        let result = run_with_storage(&storage);

        assert!(result.is_ok());
    }
}
