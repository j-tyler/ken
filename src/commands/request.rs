use chrono::Utc;
use crate::error::{KenError, Result};
use crate::session::{AgentRequest, AgentResponse, Session, SessionStatus, Event};
use crate::storage::{open_storage, Storage};

/// Run the request command - process an agent request
pub fn run(json: &str) -> Result<()> {
    let request: AgentRequest = serde_json::from_str(json)
        .map_err(|e| KenError::InvalidRequest(e.to_string()))?;

    let storage = open_storage()?;
    let response = handle_request_with_storage(&storage, request)?;

    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}

/// Handle an agent request and return a response (with injected storage for testing)
fn handle_request_with_storage(storage: &Storage, request: AgentRequest) -> Result<AgentResponse> {
    let now = Utc::now().to_rfc3339();

    match request {
        AgentRequest::Complete { session_id, result } => {
            // Verify session exists and is active
            let session = storage.get_session(&session_id)?;
            if session.status != SessionStatus::Active {
                return Ok(AgentResponse::error(&format!(
                    "Session {} is not active (status: {})",
                    session_id,
                    session.status.as_str()
                )));
            }

            // Complete the session
            storage.complete_session(&session_id, &result, &now)?;

            // Log event
            storage.insert_event(&Event::new(
                "session_completed",
                Some(&session_id),
                Some(result),
            ))?;

            Ok(AgentResponse::success(None))
        }

        AgentRequest::SpawnAndSleep { session_id, children, trigger, checkpoint } => {
            // Verify session exists and is active
            let session = storage.get_session(&session_id)?;
            if session.status != SessionStatus::Active {
                return Ok(AgentResponse::error(&format!(
                    "Session {} is not active (status: {})",
                    session_id,
                    session.status.as_str()
                )));
            }

            // Create child sessions
            let child_sessions: Vec<Session> = children
                .iter()
                .map(|spec| Session::new(&spec.ken, &spec.task, Some(session_id.clone())))
                .collect();

            // Resolve trigger with actual child IDs
            let child_ids: Vec<String> = child_sessions.iter().map(|s| s.id.clone()).collect();
            let trigger_str = resolve_trigger(&trigger, &child_ids)?;

            // Atomic spawn and sleep
            let spawned_ids = storage.spawn_and_sleep(
                &session_id,
                child_sessions,
                &trigger_str,
                &checkpoint,
                &now,
            )?;

            Ok(AgentResponse::success(Some(serde_json::json!({
                "children": spawned_ids
            }))))
        }

        AgentRequest::Sleep { session_id, trigger, checkpoint } => {
            // Verify session exists and is active
            let session = storage.get_session(&session_id)?;
            if session.status != SessionStatus::Active {
                return Ok(AgentResponse::error(&format!(
                    "Session {} is not active (status: {})",
                    session_id,
                    session.status.as_str()
                )));
            }

            let trigger_str = serde_json::to_string(&trigger)?;

            // Put session to sleep
            storage.sleep_session(&session_id, &trigger_str, &checkpoint, &now)?;

            // Log event
            storage.insert_event(&Event::new(
                "session_sleeping",
                Some(&session_id),
                Some(trigger_str),
            ))?;

            Ok(AgentResponse::success(None))
        }
    }
}

/// Resolve trigger template by replacing __CHILDREN__ with actual child IDs
fn resolve_trigger(trigger: &serde_json::Value, child_ids: &[String]) -> Result<String> {
    let trigger_str = serde_json::to_string(trigger)?;

    // Replace __CHILDREN__ placeholder with actual child IDs
    if trigger_str.contains("\"__CHILDREN__\"") {
        let ids_json = serde_json::to_string(child_ids)?;
        let resolved = trigger_str.replace("\"__CHILDREN__\"", &ids_json);
        Ok(resolved)
    } else {
        Ok(trigger_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::ChildSpec;
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
    fn test_handle_complete_request() {
        let (_dir, storage) = create_test_storage();

        // Create an active session
        let mut session = Session::with_id("test-123", "test/ken", "do something", None);
        session.status = SessionStatus::Active;
        storage.insert_session(&session).unwrap();

        let request = AgentRequest::Complete {
            session_id: "test-123".to_string(),
            result: "all done".to_string(),
        };

        let response = handle_request_with_storage(&storage, request).unwrap();

        assert!(response.ok);

        // Verify session is complete
        let updated = storage.get_session("test-123").unwrap();
        assert_eq!(updated.status, SessionStatus::Complete);
        assert_eq!(updated.result, Some("all done".to_string()));
    }

    #[test]
    fn test_handle_complete_fails_if_not_active() {
        let (_dir, storage) = create_test_storage();

        // Create a pending session (not active)
        let session = Session::with_id("test-123", "test/ken", "do something", None);
        storage.insert_session(&session).unwrap();

        let request = AgentRequest::Complete {
            session_id: "test-123".to_string(),
            result: "all done".to_string(),
        };

        let response = handle_request_with_storage(&storage, request).unwrap();

        assert!(!response.ok);
        assert!(response.error.unwrap().contains("not active"));
    }

    #[test]
    fn test_handle_spawn_and_sleep() {
        let (_dir, storage) = create_test_storage();

        // Create an active session
        let mut session = Session::with_id("parent-123", "parent/ken", "parent task", None);
        session.status = SessionStatus::Active;
        storage.insert_session(&session).unwrap();

        let request = AgentRequest::SpawnAndSleep {
            session_id: "parent-123".to_string(),
            children: vec![
                ChildSpec { ken: "child/ken1".to_string(), task: "task1".to_string() },
                ChildSpec { ken: "child/ken2".to_string(), task: "task2".to_string() },
            ],
            trigger: serde_json::json!({"all_complete": "__CHILDREN__"}),
            checkpoint: "my checkpoint data".to_string(),
        };

        let response = handle_request_with_storage(&storage, request).unwrap();

        assert!(response.ok);

        // Verify parent is sleeping
        let parent = storage.get_session("parent-123").unwrap();
        assert_eq!(parent.status, SessionStatus::Sleeping);
        assert_eq!(parent.checkpoint, Some("my checkpoint data".to_string()));

        // Verify children were created
        let children = storage.get_children("parent-123").unwrap();
        assert_eq!(children.len(), 2);

        // Verify response contains child IDs
        let data = response.data.unwrap();
        let child_ids: Vec<String> = serde_json::from_value(data["children"].clone()).unwrap();
        assert_eq!(child_ids.len(), 2);
    }

    #[test]
    fn test_handle_sleep() {
        let (_dir, storage) = create_test_storage();

        // Create an active session
        let mut session = Session::with_id("test-123", "test/ken", "do something", None);
        session.status = SessionStatus::Active;
        storage.insert_session(&session).unwrap();

        let request = AgentRequest::Sleep {
            session_id: "test-123".to_string(),
            trigger: serde_json::json!({"timeout_seconds": 60}),
            checkpoint: "waiting for timeout".to_string(),
        };

        let response = handle_request_with_storage(&storage, request).unwrap();

        assert!(response.ok);

        // Verify session is sleeping
        let updated = storage.get_session("test-123").unwrap();
        assert_eq!(updated.status, SessionStatus::Sleeping);
        assert_eq!(updated.checkpoint, Some("waiting for timeout".to_string()));
    }

    #[test]
    fn test_resolve_trigger_replaces_children() {
        let trigger = serde_json::json!({"all_complete": "__CHILDREN__"});
        let child_ids = vec!["id1".to_string(), "id2".to_string()];

        let resolved = resolve_trigger(&trigger, &child_ids).unwrap();

        assert!(resolved.contains("id1"));
        assert!(resolved.contains("id2"));
        assert!(!resolved.contains("__CHILDREN__"));
    }

    #[test]
    fn test_parse_valid_complete_request() {
        let json = r#"{"type":"complete","session_id":"abc123","result":"done"}"#;
        let request: AgentRequest = serde_json::from_str(json).unwrap();

        match request {
            AgentRequest::Complete { session_id, result } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(result, "done");
            }
            _ => panic!("Expected Complete request"),
        }
    }
}
