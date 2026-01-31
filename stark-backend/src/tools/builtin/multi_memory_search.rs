//! Multi-memory search tool for searching multiple terms at once
//!
//! This tool allows the agent to search through stored memories using multiple queries
//! in a single call, reducing repeated tool calls.

use crate::models::MemoryType;
use crate::tools::registry::Tool;
use crate::tools::types::{
    PropertySchema, ToolContext, ToolDefinition, ToolGroup, ToolInputSchema, ToolResult,
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Tool for searching memories using multiple queries at once
pub struct MultiMemorySearchTool {
    definition: ToolDefinition,
}

impl MultiMemorySearchTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();

        properties.insert(
            "queries".to_string(),
            PropertySchema {
                schema_type: "array".to_string(),
                description: "List of search queries to run. Each query will be searched independently and results combined. Use this to search for multiple related terms in one call.".to_string(),
                default: None,
                items: Some(Box::new(PropertySchema {
                    schema_type: "string".to_string(),
                    description: "A search query".to_string(),
                    default: None,
                    items: None,
                    enum_values: None,
                })),
                enum_values: None,
            },
        );

        properties.insert(
            "memory_type".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Optional filter by memory type (applies to all queries).".to_string(),
                default: None,
                items: None,
                enum_values: Some(vec![
                    "daily_log".to_string(),
                    "long_term".to_string(),
                    "preference".to_string(),
                    "fact".to_string(),
                    "task".to_string(),
                    "entity".to_string(),
                    "session_summary".to_string(),
                ]),
            },
        );

        properties.insert(
            "min_importance".to_string(),
            PropertySchema {
                schema_type: "integer".to_string(),
                description: "Minimum importance level (1-10). Only return memories with importance >= this value.".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "limit_per_query".to_string(),
            PropertySchema {
                schema_type: "integer".to_string(),
                description: "Maximum results per query (default: 5, max: 20). Total results capped at 50.".to_string(),
                default: Some(json!(5)),
                items: None,
                enum_values: None,
            },
        );

        MultiMemorySearchTool {
            definition: ToolDefinition {
                name: "multi_memory_search".to_string(),
                description: "Search stored memories using multiple queries at once. Use this to efficiently search for several related terms in a single call instead of making multiple memory_search calls. If no results are found, accept it and move on - don't retry with variations.".to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties,
                    required: vec!["queries".to_string()],
                },
                group: ToolGroup::System,
            },
        }
    }
}

impl Default for MultiMemorySearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct MultiMemorySearchParams {
    queries: Vec<String>,
    memory_type: Option<String>,
    min_importance: Option<i32>,
    limit_per_query: Option<i32>,
}

#[async_trait]
impl Tool for MultiMemorySearchTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(&self, params: Value, context: &ToolContext) -> ToolResult {
        let params: MultiMemorySearchParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        if params.queries.is_empty() {
            return ToolResult::error("At least one query is required");
        }

        if params.queries.len() > 10 {
            return ToolResult::error("Maximum 10 queries allowed per call");
        }

        // Get database from context
        let db = match &context.database {
            Some(db) => db,
            None => {
                return ToolResult::error(
                    "Database not available. Memory search requires database access.",
                );
            }
        };

        // Parse memory type filter
        let memory_type = params.memory_type.as_ref().and_then(|t| MemoryType::from_str(t));

        // Get identity from context
        let identity_id = context.identity_id.as_deref();

        // Limit per query (max 20, default 5)
        let limit_per_query = params.limit_per_query.unwrap_or(5).min(20);

        let mut all_results = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();
        let mut query_summaries = Vec::new();

        for query in &params.queries {
            // Escape the query for FTS5
            let escaped_query = escape_fts5_query(query);

            match db.search_memories(
                &escaped_query,
                memory_type,
                identity_id,
                None, // category filter not exposed in multi-search for simplicity
                params.min_importance,
                limit_per_query,
            ) {
                Ok(results) => {
                    let count = results.len();
                    let mut added = 0;

                    for result in results {
                        // Deduplicate by memory ID
                        if seen_ids.insert(result.memory.id) {
                            all_results.push((query.clone(), result));
                            added += 1;
                        }
                    }

                    query_summaries.push(format!("'{}': {} found, {} new", query, count, added));
                }
                Err(e) => {
                    query_summaries.push(format!("'{}': error - {}", query, e));
                }
            }

            // Cap total results at 50
            if all_results.len() >= 50 {
                break;
            }
        }

        if all_results.is_empty() {
            return ToolResult::success(format!(
                "No memories found for any query.\n\nQueries searched:\n{}",
                query_summaries.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
            ));
        }

        // Format results as markdown
        let mut output = format!(
            "## Multi-Memory Search Results\n\
             Queries: {}\n\
             Total unique results: {}\n\n\
             **Query breakdown:**\n{}\n\n---\n\n",
            params.queries.len(),
            all_results.len(),
            query_summaries.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
        );

        for (i, (matched_query, result)) in all_results.iter().enumerate() {
            let memory = &result.memory;

            output.push_str(&format!(
                "### {}. {} (ID: {})\n",
                i + 1,
                memory.memory_type.as_str(),
                memory.id
            ));

            output.push_str(&format!(
                "**Matched query:** '{}' | **Importance:** {} | **Relevance:** {:.2}\n",
                matched_query,
                memory.importance,
                -result.rank
            ));

            if let Some(ref category) = memory.category {
                output.push_str(&format!("**Category:** {}\n", category));
            }

            if let Some(ref entity_type) = memory.entity_type {
                output.push_str(&format!(
                    "**Entity:** {} ({})\n",
                    memory.entity_name.as_deref().unwrap_or("unknown"),
                    entity_type
                ));
            }

            output.push_str(&format!(
                "**Created:** {}\n\n",
                memory.created_at.format("%Y-%m-%d %H:%M UTC")
            ));

            // Add content (truncate if too long)
            let content = if memory.content.len() > 400 {
                format!("{}...", &memory.content[..400])
            } else {
                memory.content.clone()
            };
            output.push_str(&format!("{}\n\n---\n\n", content));
        }

        ToolResult::success(output).with_metadata(json!({
            "queries": params.queries,
            "total_results": all_results.len(),
            "memory_ids": all_results.iter().map(|(_, r)| r.memory.id).collect::<Vec<_>>()
        }))
    }
}

/// Escape special characters for FTS5 query syntax
fn escape_fts5_query(query: &str) -> String {
    let cleaned: String = query
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
        .collect();

    // If the query has multiple words, use OR for more flexible matching
    let words: Vec<&str> = cleaned.split_whitespace().collect();
    if words.len() > 1 {
        words.join(" OR ")
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_memory_search_definition() {
        let tool = MultiMemorySearchTool::new();
        let def = tool.definition();

        assert_eq!(def.name, "multi_memory_search");
        assert_eq!(def.group, ToolGroup::System);
        assert!(def.input_schema.required.contains(&"queries".to_string()));
    }

    #[test]
    fn test_fts5_escape() {
        assert_eq!(escape_fts5_query("hello world"), "hello OR world");
        assert_eq!(escape_fts5_query("single"), "single");
    }
}
