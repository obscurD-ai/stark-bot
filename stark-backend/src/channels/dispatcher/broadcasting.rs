use crate::ai::multi_agent::Orchestrator;
use crate::gateway::protocol::GatewayEvent;
use crate::telemetry::{Rollout, SpanCollector, SpanType};
use crate::tools::ToolDefinition;

use super::MessageDispatcher;

impl MessageDispatcher {
    /// Broadcast the current toolset to the UI for debug panel visibility
    pub(super) fn broadcast_toolset_update(
        &self,
        channel_id: i64,
        mode: &str,
        subtype: &str,
        tools: &[ToolDefinition],
    ) {
        let tool_summaries: Vec<serde_json::Value> = tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "group": format!("{:?}", t.group),
                })
            })
            .collect();

        self.broadcaster.broadcast(GatewayEvent::agent_toolset_update(
            channel_id,
            mode,
            subtype,
            tool_summaries,
        ));
    }

    /// Populate the current attempt's stats (tool_calls, llm_calls) from collected spans.
    pub(super) fn populate_attempt_stats(rollout: &mut Rollout, collector: &SpanCollector) {
        let spans = collector.snapshot();
        let mut tool_calls = 0u32;
        let mut llm_calls = 0u32;

        for span in &spans {
            match span.span_type {
                SpanType::ToolCall => tool_calls += 1,
                SpanType::LlmCall => llm_calls += 1,
                // Reward spans from tool_completed also count as tool observations,
                // but the ToolCall span is the canonical count
                _ => {}
            }
        }

        if let Some(attempt) = rollout.current_attempt_mut() {
            attempt.tool_calls = tool_calls;
            attempt.llm_calls = llm_calls;
        }
    }

    /// Broadcast status update event for the debug panel
    pub(super) fn broadcast_tasks_update(&self, channel_id: i64, session_id: i64, orchestrator: &Orchestrator) {
        let context = orchestrator.context();
        let mode = context.mode;
        let has_tasks = !context.task_queue.is_empty();

        // Send simplified status (no task list anymore)
        let stats_json = serde_json::json!({
            "iterations": context.mode_iterations,
            "total_iterations": context.total_iterations,
            "notes_count": context.exploration_notes.len()
        });

        self.broadcaster.broadcast(GatewayEvent::agent_tasks_update(
            channel_id,
            &mode.to_string(),
            mode.label(),
            serde_json::json!([]), // Empty tasks array
            stats_json,
        ));

        // Also broadcast task queue update if there are tasks
        if has_tasks {
            self.broadcast_task_queue_update(channel_id, session_id, orchestrator);
        }
    }

    /// Broadcast task queue update (full queue state)
    pub(super) fn broadcast_task_queue_update(&self, channel_id: i64, session_id: i64, orchestrator: &Orchestrator) {
        let task_queue = orchestrator.task_queue();
        let current_task_id = task_queue.current_task().map(|t| t.id);

        // Store tasks in execution tracker for API access (page refresh)
        self.execution_tracker.set_planner_tasks(channel_id, task_queue.tasks.clone());

        self.broadcaster.broadcast(GatewayEvent::task_queue_update(
            channel_id,
            session_id,
            &task_queue.tasks,
            current_task_id,
        ));
    }

    /// Broadcast task status change
    pub(super) fn broadcast_task_status_change(&self, channel_id: i64, session_id: i64, task_id: u32, status: &str, description: &str) {
        self.broadcaster.broadcast(GatewayEvent::task_status_change(
            channel_id,
            session_id,
            task_id,
            status,
            description,
        ));
    }

    /// Broadcast session complete
    pub(super) fn broadcast_session_complete(&self, channel_id: i64, session_id: i64) {
        // Clear stored planner tasks since session is complete
        self.execution_tracker.clear_planner_tasks(channel_id);

        self.broadcaster.broadcast(GatewayEvent::session_complete(
            channel_id,
            session_id,
        ));
    }
}
