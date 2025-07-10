#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::repl_engine::{ReplValue, ExecutionContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplSession {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub variables: HashMap<String, SessionVariable>,
    pub imports: HashMap<String, SessionImport>,
    pub history: Vec<SessionHistoryEntry>,
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionVariable {
    pub name: String,
    pub value: SessionValue,
    pub var_type: String,
    pub mutable: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionImport {
    pub module_name: String,
    pub imported_names: Vec<String>,
    pub alias: Option<String>,
    pub source_path: String,
    pub imported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHistoryEntry {
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub nagari_version: String,
    pub session_name: Option<String>,
    pub working_directory: String,
    pub command_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SessionValue {
    Number(f64),
    String(String),
    Boolean(bool),
    List(Vec<SessionValue>),
    Object(HashMap<String, SessionValue>),
    Function(String),
    Null,
    Undefined,
}

pub struct SessionManager {
    sessions_dir: PathBuf,
    current_session: Option<ReplSession>,
}

impl ReplSession {
    pub fn new() -> Self {
        let now = Utc::now();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            last_modified: now,
            variables: HashMap::new(),
            imports: HashMap::new(),
            history: Vec::new(),
            metadata: SessionMetadata {
                nagari_version: env!("CARGO_PKG_VERSION").to_string(),
                session_name: None,
                working_directory: std::env::current_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                command_count: 0,
                error_count: 0,
            },
        }
    }

    pub fn from_context(context: &ExecutionContext) -> Self {
        let mut session = Self::new();

        // Convert variables
        for variable in context.list_variables() {
            let session_var = SessionVariable {
                name: variable.name.clone(),
                value: SessionValue::from_repl_value(&variable.value),
                var_type: format!("{:?}", variable.var_type),
                mutable: variable.mutable,
                created_at: variable.created_at,
            };

            session.variables.insert(variable.name.clone(), session_var);
        }

        // Convert imports
        for import in context.list_imports() {
            let session_import = SessionImport {
                module_name: import.module_name.clone(),
                imported_names: import.imported_names.clone(),
                alias: import.alias.clone(),
                source_path: import.source_path.to_string_lossy().to_string(),
                imported_at: import.imported_at,
            };

            session.imports.insert(import.module_name.clone(), session_import);
        }

        session.metadata.working_directory = context.get_working_directory()
            .to_string_lossy()
            .to_string();

        session
    }

    pub fn update_from_context(&mut self, context: &ExecutionContext) {
        self.last_modified = Utc::now();

        // Update variables
        self.variables.clear();
        for variable in context.list_variables() {
            let session_var = SessionVariable {
                name: variable.name.clone(),
                value: SessionValue::from_repl_value(&variable.value),
                var_type: format!("{:?}", variable.var_type),
                mutable: variable.mutable,
                created_at: variable.created_at,
            };

            self.variables.insert(variable.name.clone(), session_var);
        }

        // Update imports
        self.imports.clear();
        for import in context.list_imports() {
            let session_import = SessionImport {
                module_name: import.module_name.clone(),
                imported_names: import.imported_names.clone(),
                alias: import.alias.clone(),
                source_path: import.source_path.to_string_lossy().to_string(),
                imported_at: import.imported_at,
            };

            self.imports.insert(import.module_name.clone(), session_import);
        }
    }

    pub fn add_history_entry(&mut self, command: String, success: bool, output: Option<String>) {
        let entry = SessionHistoryEntry {
            command,
            timestamp: Utc::now(),
            success,
            output,
        };

        self.history.push(entry);
        self.metadata.command_count += 1;

        if !success {
            self.metadata.error_count += 1;
        }

        self.last_modified = Utc::now();
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let session: ReplSession = serde_json::from_str(&content)?;
        Ok(session)
    }

    pub fn set_name(&mut self, name: String) {
        self.metadata.session_name = Some(name);
        self.last_modified = Utc::now();
    }

    pub fn get_name(&self) -> Option<&String> {
        self.metadata.session_name.as_ref()
    }

    pub fn get_stats(&self) -> SessionStats {
        SessionStats {
            id: self.id.clone(),
            name: self.metadata.session_name.clone(),
            created_at: self.created_at,
            last_modified: self.last_modified,
            variable_count: self.variables.len(),
            import_count: self.imports.len(),
            command_count: self.metadata.command_count,
            error_count: self.metadata.error_count,
            success_rate: if self.metadata.command_count > 0 {
                ((self.metadata.command_count - self.metadata.error_count) as f64 / self.metadata.command_count as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

impl SessionManager {
    pub fn new() -> Result<Self> {
        let sessions_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nagari")
            .join("sessions");

        std::fs::create_dir_all(&sessions_dir)?;

        Ok(Self {
            sessions_dir,
            current_session: None,
        })
    }

    pub fn create_session(&mut self) -> &ReplSession {
        let session = ReplSession::new();
        self.current_session = Some(session);
        self.current_session.as_ref().unwrap()
    }

    pub fn save_current_session(&self, name: Option<String>) -> Result<PathBuf> {
        if let Some(ref session) = self.current_session {
            let filename = match name {
                Some(name) => format!("{}.json", name),
                None => format!("session_{}.json", session.id),
            };

            let path = self.sessions_dir.join(filename);
            session.save_to_file(&path)?;
            Ok(path)
        } else {
            Err(anyhow::anyhow!("No active session"))
        }
    }

    pub fn load_session(&mut self, path: &PathBuf) -> Result<()> {
        let session = ReplSession::load_from_file(path)?;
        self.current_session = Some(session);
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let mut sessions = Vec::new();

        for entry in std::fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(session) = ReplSession::load_from_file(&path) {
                    sessions.push(SessionInfo {
                        path: path.clone(),
                        stats: session.get_stats(),
                    });
                }
            }
        }

        sessions.sort_by(|a, b| b.stats.last_modified.cmp(&a.stats.last_modified));
        Ok(sessions)
    }

    pub fn delete_session(&self, path: &PathBuf) -> Result<()> {
        std::fs::remove_file(path)?;
        Ok(())
    }

    pub fn get_current_session(&self) -> Option<&ReplSession> {
        self.current_session.as_ref()
    }

    pub fn get_current_session_mut(&mut self) -> Option<&mut ReplSession> {
        self.current_session.as_mut()
    }
}

#[derive(Debug, Clone)]
pub struct SessionStats {
    pub id: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub variable_count: usize,
    pub import_count: usize,
    pub command_count: usize,
    pub error_count: usize,
    pub success_rate: f64,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub path: PathBuf,
    pub stats: SessionStats,
}

impl SessionValue {
    pub fn from_repl_value(value: &ReplValue) -> Self {
        match value {
            ReplValue::Number(n) => SessionValue::Number(*n),
            ReplValue::String(s) => SessionValue::String(s.clone()),
            ReplValue::Boolean(b) => SessionValue::Boolean(*b),
            ReplValue::List(items) => {
                let session_items = items.iter()
                    .map(|item| SessionValue::from_repl_value(item))
                    .collect();
                SessionValue::List(session_items)
            }
            ReplValue::Object(obj) => {
                let session_obj = obj.iter()
                    .map(|(k, v)| (k.clone(), SessionValue::from_repl_value(v)))
                    .collect();
                SessionValue::Object(session_obj)
            }
            ReplValue::Function(name) => SessionValue::Function(name.clone()),
            ReplValue::Null => SessionValue::Null,
            ReplValue::Undefined => SessionValue::Undefined,
        }
    }

    pub fn to_repl_value(&self) -> ReplValue {
        match self {
            SessionValue::Number(n) => ReplValue::Number(*n),
            SessionValue::String(s) => ReplValue::String(s.clone()),
            SessionValue::Boolean(b) => ReplValue::Boolean(*b),
            SessionValue::List(items) => {
                let repl_items = items.iter()
                    .map(|item| item.to_repl_value())
                    .collect();
                ReplValue::List(repl_items)
            }
            SessionValue::Object(obj) => {
                let repl_obj = obj.iter()
                    .map(|(k, v)| (k.clone(), v.to_repl_value()))
                    .collect();
                ReplValue::Object(repl_obj)
            }
            SessionValue::Function(name) => ReplValue::Function(name.clone()),
            SessionValue::Null => ReplValue::Null,
            SessionValue::Undefined => ReplValue::Undefined,
        }
    }
}

impl std::fmt::Display for SessionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Session: {}", self.name.as_deref().unwrap_or(&self.id))?;
        writeln!(f, "  ID: {}", self.id)?;
        writeln!(f, "  Created: {}", self.created_at.format("%Y-%m-%d %H:%M:%S UTC"))?;
        writeln!(f, "  Last modified: {}", self.last_modified.format("%Y-%m-%d %H:%M:%S UTC"))?;
        writeln!(f, "  Variables: {}", self.variable_count)?;
        writeln!(f, "  Imports: {}", self.import_count)?;
        writeln!(f, "  Commands executed: {}", self.command_count)?;
        writeln!(f, "  Errors: {}", self.error_count)?;
        writeln!(f, "  Success rate: {:.1}%", self.success_rate)
    }
}
