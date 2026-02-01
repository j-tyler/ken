use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

/// Session status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Pending,
    Active,
    Sleeping,
    Complete,
    Failed,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionStatus::Pending => "pending",
            SessionStatus::Active => "active",
            SessionStatus::Sleeping => "sleeping",
            SessionStatus::Complete => "complete",
            SessionStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => SessionStatus::Pending,
            "active" => SessionStatus::Active,
            "sleeping" => SessionStatus::Sleeping,
            "complete" => SessionStatus::Complete,
            "failed" => SessionStatus::Failed,
            _ => SessionStatus::Pending,
        }
    }
}

/// A session represents one instance working within a ken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub ken: String,
    pub task: String,
    pub status: SessionStatus,
    pub parent_id: Option<String>,
    pub trigger: Option<String>,
    pub checkpoint: Option<String>,
    pub result: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Session {
    /// Create a new session with a generated ID
    pub fn new(ken: &str, task: &str, parent_id: Option<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Session {
            id: Uuid::new_v4().to_string(),
            ken: ken.to_string(),
            task: task.to_string(),
            status: SessionStatus::Pending,
            parent_id,
            trigger: None,
            checkpoint: None,
            result: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Create a session with a specific ID (for testing)
    pub fn with_id(id: &str, ken: &str, task: &str, parent_id: Option<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Session {
            id: id.to_string(),
            ken: ken.to_string(),
            task: task.to_string(),
            status: SessionStatus::Pending,
            parent_id,
            trigger: None,
            checkpoint: None,
            result: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// Event for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub ts: String,
    pub session_id: Option<String>,
    pub event_type: String,
    pub data: Option<String>,
}

impl Event {
    pub fn new(event_type: &str, session_id: Option<&str>, data: Option<String>) -> Self {
        Event {
            ts: Utc::now().to_rfc3339(),
            session_id: session_id.map(|s| s.to_string()),
            event_type: event_type.to_string(),
            data,
        }
    }
}

/// Trigger condition for waking a sleeping session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    AllComplete(Vec<String>),
    AnyComplete(Vec<String>),
    TimeoutSeconds(u64),
}

impl Trigger {
    /// Parse trigger from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        // Handle the __CHILDREN__ placeholder specially
        if json.contains("__CHILDREN__") {
            // This is a template that will be filled in later
            return Ok(Trigger::AllComplete(vec![]));
        }
        serde_json::from_str(json)
    }

    /// Check if trigger is satisfied given the current session states
    pub fn is_satisfied(&self, get_status: impl Fn(&str) -> Option<SessionStatus>) -> bool {
        match self {
            Trigger::AllComplete(ids) => {
                ids.iter().all(|id| {
                    matches!(get_status(id), Some(SessionStatus::Complete))
                })
            }
            Trigger::AnyComplete(ids) => {
                ids.iter().any(|id| {
                    matches!(get_status(id), Some(SessionStatus::Complete))
                })
            }
            Trigger::TimeoutSeconds(_) => {
                // TODO: implement timeout checking
                false
            }
        }
    }
}

/// Request from agent to ken
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentRequest {
    Complete {
        session_id: String,
        result: String,
    },
    SpawnAndSleep {
        session_id: String,
        children: Vec<ChildSpec>,
        trigger: serde_json::Value,
        checkpoint: String,
    },
    Sleep {
        session_id: String,
        trigger: serde_json::Value,
        checkpoint: String,
    },
}

/// Specification for a child session to spawn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildSpec {
    pub ken: String,
    pub task: String,
}

/// Response from ken to agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl AgentResponse {
    pub fn success(data: Option<serde_json::Value>) -> Self {
        AgentResponse {
            ok: true,
            data,
            error: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        AgentResponse {
            ok: false,
            data: None,
            error: Some(msg.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        let session = Session::new("core/cli", "build parser", None);
        assert!(!session.id.is_empty());
        assert_eq!(session.ken, "core/cli");
        assert_eq!(session.task, "build parser");
        assert_eq!(session.status, SessionStatus::Pending);
        assert!(session.parent_id.is_none());
    }

    #[test]
    fn test_session_with_parent() {
        let parent = Session::new("parent", "parent task", None);
        let child = Session::new("child", "child task", Some(parent.id.clone()));
        assert_eq!(child.parent_id, Some(parent.id));
    }

    #[test]
    fn test_session_status_roundtrip() {
        for status in [
            SessionStatus::Pending,
            SessionStatus::Active,
            SessionStatus::Sleeping,
            SessionStatus::Complete,
            SessionStatus::Failed,
        ] {
            let s = status.as_str();
            let recovered = SessionStatus::from_str(s);
            assert_eq!(recovered, status);
        }
    }

    #[test]
    fn test_trigger_all_complete_satisfied() {
        let trigger = Trigger::AllComplete(vec!["a".to_string(), "b".to_string()]);

        let satisfied = trigger.is_satisfied(|id| {
            match id {
                "a" | "b" => Some(SessionStatus::Complete),
                _ => None,
            }
        });
        assert!(satisfied);
    }

    #[test]
    fn test_trigger_all_complete_not_satisfied() {
        let trigger = Trigger::AllComplete(vec!["a".to_string(), "b".to_string()]);

        let satisfied = trigger.is_satisfied(|id| {
            match id {
                "a" => Some(SessionStatus::Complete),
                "b" => Some(SessionStatus::Active),
                _ => None,
            }
        });
        assert!(!satisfied);
    }

    #[test]
    fn test_trigger_any_complete_satisfied() {
        let trigger = Trigger::AnyComplete(vec!["a".to_string(), "b".to_string()]);

        let satisfied = trigger.is_satisfied(|id| {
            match id {
                "a" => Some(SessionStatus::Complete),
                "b" => Some(SessionStatus::Active),
                _ => None,
            }
        });
        assert!(satisfied);
    }

    #[test]
    fn test_agent_request_parse_complete() {
        let json = r#"{"type":"complete","session_id":"abc123","result":"done"}"#;
        let req: AgentRequest = serde_json::from_str(json).unwrap();
        match req {
            AgentRequest::Complete { session_id, result } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(result, "done");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_agent_request_parse_spawn_and_sleep() {
        let json = r#"{
            "type":"spawn_and_sleep",
            "session_id":"abc123",
            "children":[{"ken":"child/ken","task":"do stuff"}],
            "trigger":{"all_complete":"__CHILDREN__"},
            "checkpoint":"my checkpoint"
        }"#;
        let req: AgentRequest = serde_json::from_str(json).unwrap();
        match req {
            AgentRequest::SpawnAndSleep { session_id, children, checkpoint, .. } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(children.len(), 1);
                assert_eq!(children[0].ken, "child/ken");
                assert_eq!(checkpoint, "my checkpoint");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_agent_response_success() {
        let resp = AgentResponse::success(Some(serde_json::json!({"id": "test"})));
        assert!(resp.ok);
        assert!(resp.data.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_agent_response_error() {
        let resp = AgentResponse::error("something went wrong");
        assert!(!resp.ok);
        assert!(resp.data.is_none());
        assert_eq!(resp.error, Some("something went wrong".to_string()));
    }
}
