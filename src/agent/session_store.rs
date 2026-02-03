//! Session store - manages sessions.json metadata file
//!
//! Matches OpenClaw's session store format for compatibility:
//! ~/.localgpt/agents/<agentId>/sessions/sessions.json

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::debug;

use super::session::{get_sessions_dir_for_agent, DEFAULT_AGENT_ID};

/// Session entry in sessions.json (matches OpenClaw's SessionEntry)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionEntry {
    /// Internal session ID (UUID)
    pub session_id: String,

    /// Last update timestamp (milliseconds since epoch)
    pub updated_at: u64,

    /// CLI session IDs per provider (e.g., "claude-cli" -> "session-uuid")
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub cli_session_ids: HashMap<String, String>,

    /// Legacy field for backward compatibility with OpenClaw
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude_cli_session_id: Option<String>,

    /// Token usage tracking
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,

    /// Compaction count
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compaction_count: Option<u32>,
}

impl SessionEntry {
    pub fn new(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
            ..Default::default()
        }
    }

    /// Get CLI session ID for a provider
    pub fn get_cli_session_id(&self, provider: &str) -> Option<&str> {
        // Try the map first
        if let Some(id) = self.cli_session_ids.get(provider) {
            if !id.is_empty() {
                return Some(id.as_str());
            }
        }

        // Fallback to legacy field for claude-cli
        if provider == "claude-cli" {
            if let Some(ref id) = self.claude_cli_session_id {
                if !id.is_empty() {
                    return Some(id.as_str());
                }
            }
        }

        None
    }

    /// Set CLI session ID for a provider
    pub fn set_cli_session_id(&mut self, provider: &str, session_id: &str) {
        self.cli_session_ids
            .insert(provider.to_string(), session_id.to_string());

        // Also set legacy field for claude-cli compatibility
        if provider == "claude-cli" {
            self.claude_cli_session_id = Some(session_id.to_string());
        }

        self.updated_at = chrono::Utc::now().timestamp_millis() as u64;
    }
}

/// Session store - manages the sessions.json file
pub struct SessionStore {
    path: PathBuf,
    entries: HashMap<String, SessionEntry>,
}

impl SessionStore {
    /// Load session store for the default agent
    pub fn load() -> Result<Self> {
        Self::load_for_agent(DEFAULT_AGENT_ID)
    }

    /// Load session store for a specific agent
    pub fn load_for_agent(agent_id: &str) -> Result<Self> {
        let sessions_dir = get_sessions_dir_for_agent(agent_id)?;
        let path = sessions_dir.join("sessions.json");

        let entries = if path.exists() {
            let content = fs::read_to_string(&path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        debug!(
            "Loaded session store from {:?} ({} entries)",
            path,
            entries.len()
        );

        Ok(Self { path, entries })
    }

    /// Save session store to disk
    pub fn save(&self) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.entries)?;
        fs::write(&self.path, content)?;

        debug!("Saved session store to {:?}", self.path);
        Ok(())
    }

    /// Get a session entry by key (typically "main" for the default session)
    pub fn get(&self, session_key: &str) -> Option<&SessionEntry> {
        self.entries.get(session_key)
    }

    /// Get or create a session entry
    pub fn get_or_create(&mut self, session_key: &str, session_id: &str) -> &mut SessionEntry {
        self.entries
            .entry(session_key.to_string())
            .or_insert_with(|| SessionEntry::new(session_id))
    }

    /// Update a session entry
    pub fn update<F>(&mut self, session_key: &str, session_id: &str, f: F) -> Result<()>
    where
        F: FnOnce(&mut SessionEntry),
    {
        let entry = self.get_or_create(session_key, session_id);
        f(entry);
        entry.updated_at = chrono::Utc::now().timestamp_millis() as u64;
        self.save()
    }

    /// Get CLI session ID for a session and provider
    pub fn get_cli_session_id(&self, session_key: &str, provider: &str) -> Option<String> {
        self.get(session_key)
            .and_then(|e| e.get_cli_session_id(provider))
            .map(|s| s.to_string())
    }

    /// Set CLI session ID for a session and provider
    pub fn set_cli_session_id(
        &mut self,
        session_key: &str,
        session_id: &str,
        provider: &str,
        cli_session_id: &str,
    ) -> Result<()> {
        self.update(session_key, session_id, |entry| {
            entry.set_cli_session_id(provider, cli_session_id);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_entry_cli_session() {
        let mut entry = SessionEntry::new("test-session");

        // Initially empty
        assert!(entry.get_cli_session_id("claude-cli").is_none());

        // Set and get
        entry.set_cli_session_id("claude-cli", "cli-123");
        assert_eq!(entry.get_cli_session_id("claude-cli"), Some("cli-123"));

        // Legacy field should also be set
        assert_eq!(entry.claude_cli_session_id, Some("cli-123".to_string()));
    }
}
