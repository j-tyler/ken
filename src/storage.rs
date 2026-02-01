use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};
use crate::error::{KenError, Result};
use crate::session::{Session, SessionStatus, Event};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    ken TEXT NOT NULL,
    task TEXT NOT NULL,
    status TEXT NOT NULL,
    parent_id TEXT,
    trigger TEXT,
    checkpoint TEXT,
    result TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES sessions(id)
);

CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_parent ON sessions(parent_id);

CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TEXT NOT NULL,
    session_id TEXT,
    event_type TEXT NOT NULL,
    data TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id);
CREATE INDEX IF NOT EXISTS idx_events_ts ON events(ts);
"#;

/// Storage layer for ken - wraps SQLite database
pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Open existing database
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        Ok(Storage { conn })
    }

    /// Create new database with schema
    pub fn create(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Storage { conn })
    }

    /// Insert a new session
    pub fn insert_session(&self, session: &Session) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (id, ken, task, status, parent_id, trigger, checkpoint, result, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                session.id,
                session.ken,
                session.task,
                session.status.as_str(),
                session.parent_id,
                session.trigger,
                session.checkpoint,
                session.result,
                session.created_at,
                session.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Get session by ID
    pub fn get_session(&self, id: &str) -> Result<Session> {
        let mut stmt = self.conn.prepare(
            "SELECT id, ken, task, status, parent_id, trigger, checkpoint, result, created_at, updated_at
             FROM sessions WHERE id = ?1"
        )?;

        let session = stmt.query_row(params![id], |row| {
            Ok(Session {
                id: row.get(0)?,
                ken: row.get(1)?,
                task: row.get(2)?,
                status: SessionStatus::from_str(&row.get::<_, String>(3)?),
                parent_id: row.get(4)?,
                trigger: row.get(5)?,
                checkpoint: row.get(6)?,
                result: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        }).map_err(|_| KenError::SessionNotFound(id.to_string()))?;

        Ok(session)
    }

    /// Get sessions by status
    pub fn get_sessions_by_status(&self, status: SessionStatus) -> Result<Vec<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, ken, task, status, parent_id, trigger, checkpoint, result, created_at, updated_at
             FROM sessions WHERE status = ?1"
        )?;

        let sessions = stmt.query_map(params![status.as_str()], |row| {
            Ok(Session {
                id: row.get(0)?,
                ken: row.get(1)?,
                task: row.get(2)?,
                status: SessionStatus::from_str(&row.get::<_, String>(3)?),
                parent_id: row.get(4)?,
                trigger: row.get(5)?,
                checkpoint: row.get(6)?,
                result: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Get all sessions
    pub fn get_all_sessions(&self) -> Result<Vec<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, ken, task, status, parent_id, trigger, checkpoint, result, created_at, updated_at
             FROM sessions ORDER BY created_at"
        )?;

        let sessions = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                ken: row.get(1)?,
                task: row.get(2)?,
                status: SessionStatus::from_str(&row.get::<_, String>(3)?),
                parent_id: row.get(4)?,
                trigger: row.get(5)?,
                checkpoint: row.get(6)?,
                result: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Get children of a session
    pub fn get_children(&self, parent_id: &str) -> Result<Vec<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, ken, task, status, parent_id, trigger, checkpoint, result, created_at, updated_at
             FROM sessions WHERE parent_id = ?1"
        )?;

        let sessions = stmt.query_map(params![parent_id], |row| {
            Ok(Session {
                id: row.get(0)?,
                ken: row.get(1)?,
                task: row.get(2)?,
                status: SessionStatus::from_str(&row.get::<_, String>(3)?),
                parent_id: row.get(4)?,
                trigger: row.get(5)?,
                checkpoint: row.get(6)?,
                result: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Update session status
    pub fn update_session_status(&self, id: &str, status: SessionStatus, updated_at: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), updated_at, id],
        )?;
        Ok(())
    }

    /// Update session with result (for complete)
    pub fn complete_session(&self, id: &str, result: &str, updated_at: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET status = 'complete', result = ?1, updated_at = ?2 WHERE id = ?3",
            params![result, updated_at, id],
        )?;
        Ok(())
    }

    /// Update session to sleeping with trigger and checkpoint
    pub fn sleep_session(&self, id: &str, trigger: &str, checkpoint: &str, updated_at: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET status = 'sleeping', trigger = ?1, checkpoint = ?2, updated_at = ?3 WHERE id = ?4",
            params![trigger, checkpoint, updated_at, id],
        )?;
        Ok(())
    }

    /// Insert event
    pub fn insert_event(&self, event: &Event) -> Result<()> {
        self.conn.execute(
            "INSERT INTO events (ts, session_id, event_type, data) VALUES (?1, ?2, ?3, ?4)",
            params![event.ts, event.session_id, event.event_type, event.data],
        )?;
        Ok(())
    }

    /// Begin transaction
    pub fn begin_transaction(&self) -> Result<()> {
        self.conn.execute("BEGIN", [])?;
        Ok(())
    }

    /// Commit transaction
    pub fn commit(&self) -> Result<()> {
        self.conn.execute("COMMIT", [])?;
        Ok(())
    }

    /// Rollback transaction
    pub fn rollback(&self) -> Result<()> {
        self.conn.execute("ROLLBACK", [])?;
        Ok(())
    }

    /// Execute atomic spawn_and_sleep operation
    pub fn spawn_and_sleep(
        &self,
        parent_id: &str,
        children: Vec<Session>,
        trigger: &str,
        checkpoint: &str,
        updated_at: &str,
    ) -> Result<Vec<String>> {
        self.begin_transaction()?;

        let result = (|| {
            let mut child_ids = Vec::new();

            // Insert all children
            for child in &children {
                self.insert_session(child)?;
                child_ids.push(child.id.clone());
            }

            // Update parent to sleeping
            self.sleep_session(parent_id, trigger, checkpoint, updated_at)?;

            // Log event
            self.insert_event(&Event {
                ts: updated_at.to_string(),
                session_id: Some(parent_id.to_string()),
                event_type: "children_spawned".to_string(),
                data: Some(serde_json::to_string(&child_ids)?),
            })?;

            Ok(child_ids)
        })();

        match result {
            Ok(ids) => {
                self.commit()?;
                Ok(ids)
            }
            Err(e) => {
                let _ = self.rollback();
                Err(e)
            }
        }
    }
}

/// Find the .ken directory by searching up from current directory
pub fn find_ken_dir() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    loop {
        let ken_dir = current.join(".ken");
        if ken_dir.exists() {
            return Ok(ken_dir);
        }

        if !current.pop() {
            return Err(KenError::NotInitialized);
        }
    }
}

/// Get path to ken.db
pub fn get_db_path() -> Result<PathBuf> {
    let ken_dir = find_ken_dir()?;
    Ok(ken_dir.join("ken.db"))
}

/// Open the storage (finds .ken dir automatically)
pub fn open_storage() -> Result<Storage> {
    let db_path = get_db_path()?;
    Storage::open(&db_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_storage() -> (Storage, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("ken.db");
        let storage = Storage::create(&db_path).unwrap();
        (storage, dir)
    }

    #[test]
    fn test_create_storage() {
        let (storage, _dir) = create_test_storage();
        // Should be able to query empty tables
        let sessions = storage.get_all_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_insert_and_get_session() {
        let (storage, _dir) = create_test_storage();

        let session = Session::new("test-ken", "test task", None);
        storage.insert_session(&session).unwrap();

        let retrieved = storage.get_session(&session.id).unwrap();
        assert_eq!(retrieved.id, session.id);
        assert_eq!(retrieved.ken, "test-ken");
        assert_eq!(retrieved.task, "test task");
        assert_eq!(retrieved.status, SessionStatus::Pending);
    }

    #[test]
    fn test_get_sessions_by_status() {
        let (storage, _dir) = create_test_storage();

        let s1 = Session::new("ken1", "task1", None);
        let mut s2 = Session::new("ken2", "task2", None);
        s2.status = SessionStatus::Active;

        storage.insert_session(&s1).unwrap();
        storage.insert_session(&s2).unwrap();

        let pending = storage.get_sessions_by_status(SessionStatus::Pending).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].ken, "ken1");

        let active = storage.get_sessions_by_status(SessionStatus::Active).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].ken, "ken2");
    }

    #[test]
    fn test_update_session_status() {
        let (storage, _dir) = create_test_storage();

        let session = Session::new("test-ken", "test task", None);
        storage.insert_session(&session).unwrap();

        let now = chrono::Utc::now().to_rfc3339();
        storage.update_session_status(&session.id, SessionStatus::Active, &now).unwrap();

        let retrieved = storage.get_session(&session.id).unwrap();
        assert_eq!(retrieved.status, SessionStatus::Active);
    }

    #[test]
    fn test_complete_session() {
        let (storage, _dir) = create_test_storage();

        let mut session = Session::new("test-ken", "test task", None);
        session.status = SessionStatus::Active;
        storage.insert_session(&session).unwrap();

        let now = chrono::Utc::now().to_rfc3339();
        storage.complete_session(&session.id, "done!", &now).unwrap();

        let retrieved = storage.get_session(&session.id).unwrap();
        assert_eq!(retrieved.status, SessionStatus::Complete);
        assert_eq!(retrieved.result, Some("done!".to_string()));
    }

    #[test]
    fn test_sleep_session() {
        let (storage, _dir) = create_test_storage();

        let mut session = Session::new("test-ken", "test task", None);
        session.status = SessionStatus::Active;
        storage.insert_session(&session).unwrap();

        let now = chrono::Utc::now().to_rfc3339();
        let trigger = r#"{"all_complete":["child1","child2"]}"#;
        storage.sleep_session(&session.id, trigger, "my checkpoint", &now).unwrap();

        let retrieved = storage.get_session(&session.id).unwrap();
        assert_eq!(retrieved.status, SessionStatus::Sleeping);
        assert_eq!(retrieved.trigger, Some(trigger.to_string()));
        assert_eq!(retrieved.checkpoint, Some("my checkpoint".to_string()));
    }

    #[test]
    fn test_get_children() {
        let (storage, _dir) = create_test_storage();

        let parent = Session::new("parent-ken", "parent task", None);
        storage.insert_session(&parent).unwrap();

        let child1 = Session::new("child-ken1", "child task 1", Some(parent.id.clone()));
        let child2 = Session::new("child-ken2", "child task 2", Some(parent.id.clone()));
        storage.insert_session(&child1).unwrap();
        storage.insert_session(&child2).unwrap();

        let children = storage.get_children(&parent.id).unwrap();
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_spawn_and_sleep_atomic() {
        let (storage, _dir) = create_test_storage();

        let mut parent = Session::new("parent-ken", "parent task", None);
        parent.status = SessionStatus::Active;
        storage.insert_session(&parent).unwrap();

        let child1 = Session::new("child-ken1", "child task 1", Some(parent.id.clone()));
        let child2 = Session::new("child-ken2", "child task 2", Some(parent.id.clone()));

        let now = chrono::Utc::now().to_rfc3339();
        let trigger = r#"{"all_complete":"__CHILDREN__"}"#;
        let child_ids = storage.spawn_and_sleep(
            &parent.id,
            vec![child1, child2],
            trigger,
            "checkpoint content",
            &now,
        ).unwrap();

        assert_eq!(child_ids.len(), 2);

        // Parent should be sleeping
        let parent = storage.get_session(&parent.id).unwrap();
        assert_eq!(parent.status, SessionStatus::Sleeping);

        // Children should exist and be pending
        let children = storage.get_children(&parent.id).unwrap();
        assert_eq!(children.len(), 2);
        for child in children {
            assert_eq!(child.status, SessionStatus::Pending);
        }
    }

    #[test]
    fn test_insert_event() {
        let (storage, _dir) = create_test_storage();

        // First create a session (foreign key constraint requires valid session_id)
        let session = Session::with_id("test-session", "test/ken", "test task", None);
        storage.insert_session(&session).unwrap();

        let event = Event {
            ts: chrono::Utc::now().to_rfc3339(),
            session_id: Some("test-session".to_string()),
            event_type: "test_event".to_string(),
            data: Some(r#"{"key":"value"}"#.to_string()),
        };

        storage.insert_event(&event).unwrap();
        // Event inserted successfully
    }

    #[test]
    fn test_insert_event_without_session() {
        let (storage, _dir) = create_test_storage();

        // Events can have no session_id (for system events)
        let event = Event {
            ts: chrono::Utc::now().to_rfc3339(),
            session_id: None,
            event_type: "system_event".to_string(),
            data: Some("system started".to_string()),
        };

        storage.insert_event(&event).unwrap();
    }
}
