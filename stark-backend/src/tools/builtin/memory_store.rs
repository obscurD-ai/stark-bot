//! Memory storage tool for saving important information
//!
//! This tool allows the agent to explicitly store memories:
//! - Facts about users, projects, or topics
//! - User preferences and settings
//! - Important information to remember long-term
//! - Daily observations and logs

use crate::models::MemoryType;
use crate::tools::registry::Tool;
use crate::tools::types::{
    PropertySchema, ToolContext, ToolDefinition, ToolGroup, ToolInputSchema, ToolResult,
};
use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Tool for storing memories
pub struct MemoryStoreTool {
    definition: ToolDefinition,
}

impl MemoryStoreTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();

        properties.insert(
            "content".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "The information to remember. Be specific and concise.".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "memory_type".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Type of memory: 'fact' (general knowledge), 'preference' (user likes/dislikes), 'long_term' (important info), 'task' (commitment/todo), 'entity' (info about a person/project/thing).".to_string(),
                default: Some(json!("fact")),
                items: None,
                enum_values: Some(vec![
                    "fact".to_string(),
                    "preference".to_string(),
                    "long_term".to_string(),
                    "task".to_string(),
                    "entity".to_string(),
                ]),
            },
        );

        properties.insert(
            "importance".to_string(),
            PropertySchema {
                schema_type: "integer".to_string(),
                description: "How important is this? 1-3 (low), 4-6 (medium), 7-9 (high), 10 (critical). Default: 5.".to_string(),
                default: Some(json!(5)),
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "entity_type".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "For entity memories: the type (e.g., 'person', 'project', 'company', 'topic').".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "entity_name".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "For entity memories: the name (e.g., 'Alice', 'StarkBot', 'Ethereum').".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "category".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Optional category for organization (e.g., 'work', 'personal', 'technical').".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "tags".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Optional comma-separated tags for searchability.".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        MemoryStoreTool {
            definition: ToolDefinition {
                name: "memory_store".to_string(),
                description: "Store important information in long-term memory. Use this to remember facts about users, preferences, project details, commitments, or any information that should persist across conversations.".to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties,
                    required: vec!["content".to_string()],
                },
                group: ToolGroup::Memory,
            },
        }
    }
}

impl Default for MemoryStoreTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct MemoryStoreParams {
    content: String,
    memory_type: Option<String>,
    importance: Option<i32>,
    entity_type: Option<String>,
    entity_name: Option<String>,
    category: Option<String>,
    tags: Option<String>,
}

#[async_trait]
impl Tool for MemoryStoreTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(&self, params: Value, context: &ToolContext) -> ToolResult {
        let params: MemoryStoreParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        // Validate content
        let content = params.content.trim();
        if content.is_empty() {
            return ToolResult::error("Content cannot be empty");
        }

        if content.len() > 10000 {
            return ToolResult::error("Content too long (max 10000 characters)");
        }

        // Parse memory type
        let memory_type = match params.memory_type.as_deref().unwrap_or("fact") {
            "fact" => MemoryType::Fact,
            "preference" => MemoryType::Preference,
            "long_term" => MemoryType::LongTerm,
            "task" => MemoryType::Task,
            "entity" => MemoryType::Entity,
            other => return ToolResult::error(format!("Invalid memory_type: '{}'. Use: fact, preference, long_term, task, entity", other)),
        };

        // Validate importance
        let importance = params.importance.unwrap_or(5).clamp(1, 10);

        // For entity type, require entity_name
        if memory_type == MemoryType::Entity && params.entity_name.is_none() {
            return ToolResult::error("entity_name is required for entity memories");
        }

        // Get database from context
        let db = match &context.database {
            Some(db) => db,
            None => return ToolResult::error("Database not available. Memory storage requires database access."),
        };

        // Get identity from context (user who initiated the conversation)
        let identity_id = context.identity_id.as_deref();

        // Create the memory
        let result = db.create_memory_extended(
            memory_type.clone(),
            content,
            params.category.as_deref(),
            params.tags.as_deref(),
            importance,
            identity_id,
            context.session_id,
            context.channel_type.as_deref(),
            None, // source_message_id
            if memory_type == MemoryType::Task { Some(Utc::now().date_naive()) } else { None },
            None, // expires_at
            params.entity_type.as_deref(),
            params.entity_name.as_deref(),
            None, // confidence
            Some("agent"), // source_type
            None, // valid_from
            None, // valid_until
            None, // temporal_type
        );

        match result {
            Ok(memory) => {
                let type_label = match memory_type {
                    MemoryType::Fact => "fact",
                    MemoryType::Preference => "preference",
                    MemoryType::LongTerm => "long-term memory",
                    MemoryType::Task => "task",
                    MemoryType::Entity => "entity info",
                    _ => "memory",
                };

                let mut msg = format!("Stored {} (ID: {}, importance: {})", type_label, memory.id, importance);

                if let Some(ref name) = params.entity_name {
                    msg.push_str(&format!(" about '{}'", name));
                }

                ToolResult::success(msg).with_metadata(json!({
                    "id": memory.id,
                    "memory_type": memory_type.as_str(),
                    "importance": importance,
                    "entity_type": params.entity_type,
                    "entity_name": params.entity_name,
                    "category": params.category,
                    "tags": params.tags,
                }))
            }
            Err(e) => ToolResult::error(format!("Failed to store memory: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition() {
        let tool = MemoryStoreTool::new();
        let def = tool.definition();

        assert_eq!(def.name, "memory_store");
        assert_eq!(def.group, ToolGroup::Memory);
        assert!(def.input_schema.required.contains(&"content".to_string()));
    }
}
