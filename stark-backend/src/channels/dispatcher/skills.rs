use crate::ai::multi_agent::{types as agent_types, Orchestrator};
use crate::gateway::protocol::GatewayEvent;
use crate::tools::{ToolConfig, ToolDefinition};

use super::MessageDispatcher;

impl MessageDispatcher {
    /// Auto-set the orchestrator's subtype if the skill specifies one.
    /// Returns the new subtype key if it changed, so the caller can use it for tool refresh.
    pub(super) fn apply_skill_subtype(
        &self,
        skill: &crate::skills::types::DbSkill,
        orchestrator: &mut Orchestrator,
        channel_id: i64,
    ) -> Option<String> {
        if let Some(ref subtype_str) = skill.subagent_type {
            if let Some(resolved_key) = agent_types::resolve_subtype_key(subtype_str) {
                orchestrator.set_subtype(Some(resolved_key.clone()));
                log::info!(
                    "[SKILL] Auto-set subtype to {} for skill '{}'",
                    agent_types::subtype_label(&resolved_key),
                    skill.name
                );
                self.broadcaster.broadcast(GatewayEvent::agent_subtype_change(
                    channel_id,
                    &resolved_key,
                    &agent_types::subtype_label(&resolved_key),
                ));
                return Some(resolved_key);
            }
        }
        None
    }

    /// Returns the list of skills available for the given context.
    ///
    /// Filtering layers:
    /// 1. Only enabled skills from the database
    /// 2. Only skills whose tags intersect with the subtype's `skill_tags`
    /// 3. In safe mode, only skills whose `requires_tools` are all available
    ///    under the current tool config
    pub(super) fn available_skills_for_context(
        &self,
        subtype_key: &str,
        tool_config: &ToolConfig,
    ) -> Vec<crate::skills::types::DbSkill> {
        use crate::tools::ToolProfile;

        let skills = match self.db.list_enabled_skills() {
            Ok(s) => s,
            Err(e) => {
                log::warn!("[SKILL] Failed to query enabled skills: {}", e);
                return vec![];
            }
        };

        // Filter by subtype's skill_tags OR explicit grant by name.
        // A skill is visible if:
        //   - any of its tags match the subtype's allowed tags, OR
        //   - it was explicitly granted by name via a special role
        let allowed_tags = agent_types::allowed_skill_tags_for_key(subtype_key);
        let skills: Vec<_> = skills
            .into_iter()
            .filter(|skill| {
                skill.tags.iter().any(|tag| allowed_tags.contains(tag))
                    || tool_config.extra_skill_names.contains(&skill.name)
            })
            .collect();

        // In safe mode, additionally filter out skills whose requires_tools
        // include tools that aren't available under the safe mode config
        if tool_config.profile == ToolProfile::SafeMode {
            skills
                .into_iter()
                .filter(|skill| {
                    // Skills with no required tools are fine (instruction-only)
                    skill.requires_tools.is_empty()
                        || skill.requires_tools.iter().all(|tool_name| {
                            self.tool_registry
                                .get(tool_name)
                                .map(|tool| {
                                    tool_config.is_tool_allowed(
                                        &tool.definition().name,
                                        tool.group(),
                                    )
                                })
                                .unwrap_or(false)
                        })
                })
                .collect()
        } else {
            skills
        }
    }

    /// Build a context-aware `use_skill` tool definition with dynamic enum_values
    /// and description based on the skills available in the current context
    /// (subtype tags + special-role grants + safe-mode filtering).
    ///
    /// Returns `None` if no skills are available (the registered tool will be
    /// stripped from the tool list in that case).
    pub(super) fn create_skill_tool_definition_for_subtype(
        &self,
        subtype_key: &str,
        tool_config: &ToolConfig,
    ) -> Option<ToolDefinition> {
        use crate::tools::{PropertySchema, ToolGroup, ToolInputSchema};

        let skills = self.available_skills_for_context(subtype_key, tool_config);

        if skills.is_empty() {
            log::debug!(
                "[SKILL] No skills available for subtype '{}' -- use_skill will NOT be injected",
                subtype_key
            );
            return None;
        }

        let skill_names: Vec<String> = skills.iter().map(|s| s.name.clone()).collect();

        let mut properties = std::collections::HashMap::new();
        properties.insert(
            "skill_name".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: format!("The skill to execute. Options: {}", skill_names.join(", ")),
                default: None,
                items: None,
                enum_values: Some(skill_names),
            },
        );
        properties.insert(
            "input".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Input or query for the skill".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        // Format skill descriptions with newlines for better readability
        let formatted_skills = skills
            .iter()
            .map(|s| format!("  - {}: {}", s.name, s.description))
            .collect::<Vec<_>>()
            .join("\n");

        Some(ToolDefinition {
            name: "use_skill".to_string(),
            description: format!(
                "Execute a specialized skill. YOU MUST use this tool when a user asks for something that matches a skill.\n\nAvailable skills:\n{}",
                formatted_skills
            ),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties,
                required: vec!["skill_name".to_string(), "input".to_string()],
            },
            group: ToolGroup::System,
                hidden: false,
        })
    }

    /// Build a `use_skill` definition showing ALL enabled skills (no subtype filtering).
    /// Used when no subtype is set yet so the AI can select a skill alongside set_agent_subtype.
    pub(super) fn create_skill_tool_definition_all_skills(
        &self,
        _tool_config: &ToolConfig,
    ) -> Option<ToolDefinition> {
        use crate::tools::{PropertySchema, ToolGroup, ToolInputSchema};

        let skills = match self.db.list_enabled_skills() {
            Ok(s) => s,
            Err(_) => return None,
        };

        if skills.is_empty() {
            return None;
        }

        let skill_names: Vec<String> = skills.iter().map(|s| s.name.clone()).collect();

        let mut properties = std::collections::HashMap::new();
        properties.insert(
            "skill_name".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: format!("The skill to execute. Options: {}", skill_names.join(", ")),
                default: None,
                items: None,
                enum_values: Some(skill_names),
            },
        );
        properties.insert(
            "input".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Input or query for the skill".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        let formatted_skills = skills
            .iter()
            .map(|s| format!("  - {}: {}", s.name, s.description))
            .collect::<Vec<_>>()
            .join("\n");

        Some(ToolDefinition {
            name: "use_skill".to_string(),
            description: format!(
                "Execute a specialized skill. YOU MUST use this tool when a user asks for something that matches a skill.\n\nAvailable skills:\n{}",
                formatted_skills
            ),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties,
                required: vec!["skill_name".to_string(), "input".to_string()],
            },
            group: ToolGroup::System,
            hidden: false,
        })
    }

    /// Build the complete tool list for the current agent state.
    ///
    /// This centralizes tool list construction that was previously duplicated
    /// across 7+ sites. The tool list is built in layers:
    ///
    /// 1. **Subtype group filtering** -- each subtype key allows specific `ToolGroup`s.
    ///    Tools outside those groups are excluded.
    /// 2. **Skill `requires_tools` force-inclusion** -- if the active skill specifies
    ///    `requires_tools`, those tools are force-included even if their group isn't
    ///    allowed by the subtype.
    /// 3. **`use_skill` pseudo-tool** -- added if any skills are enabled in the DB.
    /// 4. **Orchestrator mode tools** -- e.g. `define_tasks` in TaskPlanner mode.
    /// 5. **`define_tasks` stripping** -- removed unless the active skill's
    ///    `requires_tools` explicitly includes it (keeps it out of Assistant mode).
    ///
    /// Note: Safe mode filtering is handled upstream by `ToolConfig`, not here.
    pub(super) fn build_tool_list(
        &self,
        tool_config: &ToolConfig,
        subtype_key: &str,
        orchestrator: &Orchestrator,
    ) -> Vec<ToolDefinition> {
        let requires_tools = orchestrator.context().active_skill
            .as_ref()
            .map(|s| s.requires_tools.clone())
            .unwrap_or_default();

        let mut tools = if !requires_tools.is_empty() {
            self.tool_registry
                .get_tool_definitions_for_subtype_with_required(
                    tool_config,
                    subtype_key,
                    &requires_tools,
                )
        } else {
            self.tool_registry
                .get_tool_definitions_for_subtype(tool_config, subtype_key)
        };

        // Patch use_skill: replace the static registered definition with a
        // context-aware one (dynamic enum_values + description), or remove it
        // entirely if no skills are available for this context.
        //
        // When no subtype is set yet (key=""), we still show use_skill with ALL
        // enabled skills so the AI can call set_agent_subtype + use_skill in the
        // same turn. Once a subtype is active, skills are filtered by tags.
        let has_skill_tags = !agent_types::allowed_skill_tags_for_key(subtype_key).is_empty();
        let use_skill_allowed = tool_config.allow_list.iter().any(|t| t == "use_skill");
        let no_subtype_yet = subtype_key.is_empty();
        if has_skill_tags || use_skill_allowed || no_subtype_yet {
            if let Some(patched_def) = if no_subtype_yet {
                // No subtype -> show all enabled skills (unfiltered)
                self.create_skill_tool_definition_all_skills(tool_config)
            } else {
                // Subtype active -> filter by tags + safe mode
                self.create_skill_tool_definition_for_subtype(subtype_key, tool_config)
            } {
                // Replace the static definition with the context-aware one
                if let Some(existing) = tools.iter_mut().find(|t| t.name == "use_skill") {
                    *existing = patched_def;
                }
            } else {
                // No skills available -- remove use_skill entirely
                tools.retain(|t| t.name != "use_skill");
            }
        } else {
            // Subtype has no skill tags and no special role grant -- remove use_skill
            tools.retain(|t| t.name != "use_skill");
        }

        tools.extend(orchestrator.get_mode_tools());

        // Strip define_tasks unless a skill requires it or the subtype explicitly includes it
        let skill_requires_define_tasks = requires_tools.iter().any(|t| t == "define_tasks");
        let subtype_has_define_tasks = agent_types::get_subtype_config(subtype_key)
            .map(|c| c.additional_tools.iter().any(|t| t == "define_tasks"))
            .unwrap_or(false);
        if !skill_requires_define_tasks && !subtype_has_define_tasks {
            tools.retain(|t| t.name != "define_tasks");
        }

        // When a subtype is already active, patch set_agent_subtype description
        // so the LLM doesn't re-call it every turn
        if orchestrator.current_subtype().is_some() {
            if let Some(tool) = tools.iter_mut().find(|t| t.name == "set_agent_subtype") {
                tool.description = format!(
                    "Switch toolbox (currently: {} {}). Only call this if the user's request \
                     requires a DIFFERENT toolbox than what's already active. \
                     Do NOT call this if you're already in the right mode.",
                    agent_types::subtype_emoji(subtype_key),
                    agent_types::subtype_label(subtype_key),
                );
            }
        }

        tools
    }
}
