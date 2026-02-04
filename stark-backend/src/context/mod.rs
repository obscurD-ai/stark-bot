//! Context management for session conversations
//!
//! This module provides:
//! - Token estimation for messages
//! - Context compaction (summarizing old messages when context grows too large)
//! - Pre-compaction memory flush (AI extracts memories before summarization)
//! - Session memory hooks (saving session summaries on reset)

use crate::ai::{AiClient, Message, MessageRole};
use crate::config::MemoryConfig;
use crate::db::Database;
use crate::models::SessionMessage;
use crate::models::session_message::MessageRole as DbMessageRole;
use crate::qmd_memory::MemoryStore;
use chrono::Utc;
use std::sync::Arc;

/// Default context window size (Claude 3.5 Sonnet)
pub const DEFAULT_MAX_CONTEXT_TOKENS: i32 = 100_000;

/// Reserve tokens for system prompt and output
pub const DEFAULT_RESERVE_TOKENS: i32 = 20_000;

/// Minimum messages to keep after compaction
pub const MIN_KEEP_RECENT_MESSAGES: i32 = 5;

/// Default number of messages to keep after compaction
pub const DEFAULT_KEEP_RECENT_MESSAGES: i32 = 10;

/// Estimate token count for a string
/// Uses a simple heuristic: ~4 characters per token for English text
/// This is a rough approximation - actual tokenization varies by model
pub fn estimate_tokens(text: &str) -> i32 {
    // Average ~4 chars per token, but account for whitespace and punctuation
    let chars = text.chars().count();
    ((chars as f64) / 3.5).ceil() as i32
}

/// Estimate total tokens for a list of messages
pub fn estimate_messages_tokens(messages: &[SessionMessage]) -> i32 {
    messages.iter()
        .map(|m| {
            // Add overhead for role prefix (~4 tokens)
            estimate_tokens(&m.content) + 4
        })
        .sum()
}

/// Context manager for handling session context and compaction
pub struct ContextManager {
    db: Arc<Database>,
    /// Maximum context window size in tokens
    max_context_tokens: i32,
    /// Tokens to reserve for system prompt and output
    reserve_tokens: i32,
    /// Number of recent messages to keep after compaction
    keep_recent_messages: i32,
    /// Memory configuration
    memory_config: MemoryConfig,
    /// QMD Memory store for file-based memory
    memory_store: Option<Arc<MemoryStore>>,
}

impl ContextManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            max_context_tokens: DEFAULT_MAX_CONTEXT_TOKENS,
            reserve_tokens: DEFAULT_RESERVE_TOKENS,
            keep_recent_messages: DEFAULT_KEEP_RECENT_MESSAGES,
            memory_config: MemoryConfig::from_env(),
            memory_store: None,
        }
    }

    pub fn with_max_context(mut self, tokens: i32) -> Self {
        self.max_context_tokens = tokens;
        self
    }

    pub fn with_reserve_tokens(mut self, tokens: i32) -> Self {
        self.reserve_tokens = tokens;
        self
    }

    pub fn with_keep_recent(mut self, count: i32) -> Self {
        self.keep_recent_messages = count.max(MIN_KEEP_RECENT_MESSAGES);
        self
    }

    pub fn with_memory_config(mut self, config: MemoryConfig) -> Self {
        self.memory_config = config;
        self
    }

    pub fn with_memory_store(mut self, store: Arc<MemoryStore>) -> Self {
        self.memory_store = Some(store);
        self
    }

    /// Check if compaction is needed for a session
    pub fn needs_compaction(&self, session_id: i64) -> bool {
        if let Ok(session) = self.db.get_chat_session(session_id) {
            if let Some(session) = session {
                let threshold = session.max_context_tokens - self.reserve_tokens;
                return session.context_tokens > threshold;
            }
        }
        false
    }

    /// Get available context budget (after reserving tokens)
    pub fn get_context_budget(&self, session_id: i64) -> i32 {
        if let Ok(Some(session)) = self.db.get_chat_session(session_id) {
            return session.max_context_tokens - self.reserve_tokens - session.context_tokens;
        }
        self.max_context_tokens - self.reserve_tokens
    }

    /// Build conversation context for AI, including compaction summary if present
    pub fn build_context(&self, session_id: i64, limit: i32) -> Vec<SessionMessage> {
        // Get recent messages
        let messages = self.db.get_recent_session_messages(session_id, limit)
            .unwrap_or_default();

        messages
    }

    /// Get compaction summary for a session (if any)
    pub fn get_compaction_summary(&self, session_id: i64) -> Option<String> {
        self.db.get_session_compaction_summary(session_id).ok().flatten()
    }

    /// Phase 1: Flush memories before compaction
    /// Gives the AI a "silent turn" to extract important memories from the conversation
    /// that would otherwise be lost during summarization.
    /// Now writes to QMD markdown files instead of database.
    pub async fn flush_memories_before_compaction(
        &self,
        session_id: i64,
        client: &AiClient,
        identity_id: Option<&str>,
        messages_to_compact: &[SessionMessage],
    ) -> Result<usize, String> {
        if messages_to_compact.is_empty() {
            return Ok(0);
        }

        // Check if we have a memory store
        let memory_store = match &self.memory_store {
            Some(store) => store,
            None => {
                log::warn!("[PRE_FLUSH] No memory store available, skipping memory flush");
                return Ok(0);
            }
        };

        log::info!("[PRE_FLUSH] Starting memory flush for session {} ({} messages)",
            session_id, messages_to_compact.len());

        // Build conversation text
        let conversation_text = messages_to_compact.iter()
            .map(|m| {
                let role = match m.role {
                    DbMessageRole::User => "User",
                    DbMessageRole::Assistant => "Assistant",
                    DbMessageRole::System => "System",
                    DbMessageRole::ToolCall => "Tool Call",
                    DbMessageRole::ToolResult => "Tool Result",
                };
                format!("{}: {}", role, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        // Prompt the AI to extract memories - simplified for markdown output
        let flush_prompt = format!(
            "Before this conversation history is summarized, extract any important information that should be remembered.\n\n\
            Format your response as markdown with sections:\n\
            ## Long-Term (facts, preferences, important info)\n\
            - bullet points\n\n\
            ## Daily Activity (what was done today)\n\
            - bullet points\n\n\
            Only extract genuinely important information. Don't save trivial details.\n\
            If nothing important needs to be saved, respond with just: NO_MEMORIES_NEEDED\n\n\
            Conversation to analyze:\n{}\n\n\
            Extract memories:",
            conversation_text
        );

        let flush_messages = vec![
            Message {
                role: MessageRole::System,
                content: "You are a memory extraction assistant. Extract important information from conversations and format it as markdown.".to_string(),
            },
            Message {
                role: MessageRole::User,
                content: flush_prompt,
            },
        ];

        let response = client.generate_text(flush_messages).await
            .map_err(|e| format!("Failed to generate memory flush: {}", e))?;

        if response.contains("NO_MEMORIES_NEEDED") {
            log::info!("[PRE_FLUSH] No memories to extract for session {}", session_id);
            return Ok(0);
        }

        // Parse and write to markdown files
        let mut count = 0;

        // Extract long-term section
        if let Some(long_term_start) = response.find("## Long-Term") {
            let section_end = response[long_term_start..]
                .find("\n## ")
                .map(|i| long_term_start + i)
                .unwrap_or(response.len());
            let long_term_content = &response[long_term_start..section_end];

            if !long_term_content.trim().is_empty() {
                if let Err(e) = memory_store.append_long_term(long_term_content, identity_id) {
                    log::error!("[PRE_FLUSH] Failed to write long-term memory: {}", e);
                } else {
                    count += 1;
                    log::info!("[PRE_FLUSH] Wrote long-term memories");
                }
            }
        }

        // Extract daily activity section
        if let Some(daily_start) = response.find("## Daily") {
            let section_end = response[daily_start..]
                .find("\n## ")
                .map(|i| daily_start + i)
                .unwrap_or(response.len());
            let daily_content = &response[daily_start..section_end];

            if !daily_content.trim().is_empty() {
                if let Err(e) = memory_store.append_daily_log(daily_content, identity_id) {
                    log::error!("[PRE_FLUSH] Failed to write daily log: {}", e);
                } else {
                    count += 1;
                    log::info!("[PRE_FLUSH] Wrote daily activity");
                }
            }
        }

        log::info!("[PRE_FLUSH] Extracted {} memory sections for session {}", count, session_id);

        // Update last_flush_at timestamp
        if let Err(e) = self.db.update_session_last_flush(session_id) {
            log::warn!("[PRE_FLUSH] Failed to update last_flush_at: {}", e);
        }

        Ok(count)
    }

    /// Perform context compaction for a session
    /// Returns the number of messages compacted
    pub async fn compact_session(
        &self,
        session_id: i64,
        client: &AiClient,
        identity_id: Option<&str>,
    ) -> Result<i32, String> {
        // Get messages to compact (all except recent ones)
        let messages_to_compact = self.db.get_messages_for_compaction(session_id, self.keep_recent_messages)
            .map_err(|e| format!("Failed to get messages for compaction: {}", e))?;

        if messages_to_compact.is_empty() {
            log::info!("[COMPACTION] No messages to compact for session {}", session_id);
            return Ok(0);
        }

        let message_count = messages_to_compact.len() as i32;
        log::info!("[COMPACTION] Compacting {} messages for session {}", message_count, session_id);

        // Phase 1: Pre-compaction memory flush (writes to markdown files)
        if self.memory_config.enable_pre_compaction_flush {
            match self.flush_memories_before_compaction(
                session_id,
                client,
                identity_id,
                &messages_to_compact,
            ).await {
                Ok(count) => {
                    if count > 0 {
                        log::info!("[COMPACTION] Pre-flush saved {} memory sections", count);
                    }
                }
                Err(e) => {
                    log::warn!("[COMPACTION] Pre-flush failed (continuing with compaction): {}", e);
                }
            }
        }

        // Build the conversation text for summarization
        let conversation_text = messages_to_compact.iter()
            .map(|m| {
                let role = match m.role {
                    DbMessageRole::User => "User",
                    DbMessageRole::Assistant => "Assistant",
                    DbMessageRole::System => "System",
                    DbMessageRole::ToolCall => "Tool Call",
                    DbMessageRole::ToolResult => "Tool Result",
                };
                format!("{}: {}", role, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        // Generate summary using AI
        let summary_prompt = format!(
            "Summarize the following conversation history concisely. \
            Focus on: key topics discussed, important decisions made, user preferences learned, \
            and any tasks or commitments. Keep it factual and under 500 words.\n\n\
            Conversation:\n{}\n\nSummary:",
            conversation_text
        );

        let summary_messages = vec![
            Message {
                role: MessageRole::System,
                content: "You are a helpful assistant that summarizes conversations accurately and concisely.".to_string(),
            },
            Message {
                role: MessageRole::User,
                content: summary_prompt,
            },
        ];

        let summary = client.generate_text(summary_messages).await
            .map_err(|e| format!("Failed to generate compaction summary: {}", e))?;

        log::info!("[COMPACTION] Generated summary ({} chars) for session {}", summary.len(), session_id);

        // Write the compaction summary to the daily log as a session summary
        if let Some(ref memory_store) = self.memory_store {
            let summary_entry = format!("### Session Summary\n{}", summary);
            if let Err(e) = memory_store.append_daily_log(&summary_entry, identity_id) {
                log::error!("[COMPACTION] Failed to write session summary to daily log: {}", e);
            }
        }

        // Store summary in session record for context building
        // (The session still needs a compaction summary for the conversation window)
        if let Err(e) = self.db.set_session_compaction_summary(session_id, &summary) {
            log::warn!("[COMPACTION] Failed to store compaction summary in session: {}", e);
        }

        // Delete the compacted messages
        let deleted = self.db.delete_compacted_messages(session_id, self.keep_recent_messages)
            .map_err(|e| format!("Failed to delete compacted messages: {}", e))?;

        log::info!("[COMPACTION] Deleted {} old messages for session {}", deleted, session_id);

        // Recalculate and update context tokens
        let remaining = self.db.get_session_messages(session_id).unwrap_or_default();
        let new_token_count = estimate_messages_tokens(&remaining) + estimate_tokens(&summary);
        self.db.update_session_context_tokens(session_id, new_token_count)
            .map_err(|e| format!("Failed to update context tokens: {}", e))?;

        Ok(message_count)
    }

    /// Update context tokens after adding a message
    pub fn update_context_tokens(&self, session_id: i64, message_tokens: i32) {
        if let Ok(Some(session)) = self.db.get_chat_session(session_id) {
            let new_total = session.context_tokens + message_tokens;
            let _ = self.db.update_session_context_tokens(session_id, new_total);
        }
    }
}

/// Save session summary before reset (session memory hook)
/// Now writes to QMD markdown files instead of database.
pub async fn save_session_memory(
    db: &Arc<Database>,
    client: &AiClient,
    session_id: i64,
    identity_id: Option<&str>,
    message_limit: i32,
    memory_store: Option<&Arc<MemoryStore>>,
) -> Result<(), String> {
    // Get recent messages from the session
    let messages = db.get_recent_session_messages(session_id, message_limit)
        .map_err(|e| format!("Failed to get session messages: {}", e))?;

    if messages.is_empty() {
        return Err("No messages to summarize".to_string());
    }

    log::info!("[SESSION_MEMORY] Saving session memory for {} messages", messages.len());

    // Build conversation text
    let conversation_text = messages.iter()
        .map(|m| {
            let role = match m.role {
                DbMessageRole::User => "User",
                DbMessageRole::Assistant => "Assistant",
                DbMessageRole::System => "System",
                DbMessageRole::ToolCall => "Tool Call",
                DbMessageRole::ToolResult => "Tool Result",
            };
            format!("{}: {}", role, m.content)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    // Generate summary and title using AI
    let summary_prompt = format!(
        "Analyze this conversation and provide:\n\
        1. A short descriptive title (5-10 words)\n\
        2. A brief summary of the key points discussed\n\n\
        Format your response as:\n\
        TITLE: <title here>\n\
        SUMMARY: <summary here>\n\n\
        Conversation:\n{}",
        conversation_text
    );

    let ai_messages = vec![
        Message {
            role: MessageRole::System,
            content: "You summarize conversations concisely. Respond only with the requested TITLE and SUMMARY format.".to_string(),
        },
        Message {
            role: MessageRole::User,
            content: summary_prompt,
        },
    ];

    let response = client.generate_text(ai_messages).await
        .map_err(|e| format!("Failed to generate session summary: {}", e))?;

    // Parse title and summary from response
    let (title, summary) = parse_title_summary(&response);

    // Write to daily log in QMD memory store
    if let Some(store) = memory_store {
        let content = format!("### {}\n{}", title, summary);
        store.append_daily_log(&content, identity_id)
            .map_err(|e| format!("Failed to write session summary: {}", e))?;
        log::info!("[SESSION_MEMORY] Saved session summary to daily log: {}", title);
    } else {
        log::warn!("[SESSION_MEMORY] No memory store available, session summary not saved");
    }

    Ok(())
}

/// Parse title and summary from AI response
fn parse_title_summary(response: &str) -> (String, String) {
    let mut title = String::new();
    let mut summary = String::new();

    for line in response.lines() {
        let line = line.trim();
        if line.to_uppercase().starts_with("TITLE:") {
            title = line[6..].trim().to_string();
        } else if line.to_uppercase().starts_with("SUMMARY:") {
            summary = line[8..].trim().to_string();
        } else if !title.is_empty() && !line.to_uppercase().starts_with("SUMMARY:") && summary.is_empty() {
            // Multi-line handling - append to title if before summary
        } else if !summary.is_empty() {
            // Append to summary if we're past the SUMMARY: prefix
            if !summary.is_empty() {
                summary.push(' ');
            }
            summary.push_str(line);
        }
    }

    // Fallbacks
    if title.is_empty() {
        title = format!("Session {}", Utc::now().format("%Y-%m-%d %H:%M"));
    }
    if summary.is_empty() {
        summary = response.chars().take(500).collect();
    }

    (title, summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        // Roughly 4 chars per token
        assert!(estimate_tokens("hello") >= 1);
        assert!(estimate_tokens("hello world") >= 2);

        // Longer text
        let long_text = "This is a longer piece of text that should estimate to roughly 10-15 tokens based on our heuristic.";
        let tokens = estimate_tokens(long_text);
        assert!(tokens >= 10 && tokens <= 50);
    }

    #[test]
    fn test_parse_title_summary() {
        let response = "TITLE: Discussion about Rust programming\nSUMMARY: User asked about ownership and borrowing in Rust.";
        let (title, summary) = parse_title_summary(response);
        assert_eq!(title, "Discussion about Rust programming");
        assert!(summary.contains("ownership"));
    }
}
