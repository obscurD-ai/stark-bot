use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::channels::NormalizedMessage;
use crate::AppState;

/// Web channel ID - a reserved ID for web-based chat
/// This is used to identify messages from the web frontend
const WEB_CHANNEL_ID: i64 = 0;
const WEB_CHANNEL_TYPE: &str = "web";

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    /// Optional user identifier for the web session
    #[serde(default)]
    pub user_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Session ID for persistent conversations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<i64>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/api/chat").route(web::post().to(chat)));
}

async fn chat(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<ChatRequest>,
) -> impl Responder {
    // Validate session token
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").to_string());

    let token = match token {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized().json(ChatResponse {
                success: false,
                message: None,
                error: Some("No authorization token provided".to_string()),
                session_id: None,
            });
        }
    };

    // Validate the session
    match state.db.validate_session(&token) {
        Ok(Some(_)) => {} // Session is valid
        Ok(None) => {
            return HttpResponse::Unauthorized().json(ChatResponse {
                success: false,
                message: None,
                error: Some("Invalid or expired session".to_string()),
                session_id: None,
            });
        }
        Err(e) => {
            log::error!("Failed to validate session: {}", e);
            return HttpResponse::InternalServerError().json(ChatResponse {
                success: false,
                message: None,
                error: Some("Internal server error".to_string()),
                session_id: None,
            });
        }
    };

    // Get the latest user message from the request
    let user_message = match body.messages.iter().rev().find(|m| m.role == "user") {
        Some(msg) => msg.content.clone(),
        None => {
            return HttpResponse::BadRequest().json(ChatResponse {
                success: false,
                message: None,
                error: Some("No user message provided".to_string()),
                session_id: None,
            });
        }
    };

    // Generate a user ID for the web session
    // Use the provided user_id, or derive from the session token
    let user_id = body.user_id.clone()
        .unwrap_or_else(|| format!("web-{}", &token[..8.min(token.len())]));

    // Create a normalized message for the dispatcher
    // This makes web chat go through the same pipeline as Telegram/Slack
    let normalized = NormalizedMessage {
        channel_id: WEB_CHANNEL_ID,
        channel_type: WEB_CHANNEL_TYPE.to_string(),
        chat_id: user_id.clone(),  // For web, chat_id == user_id (always DM-like)
        user_id: user_id.clone(),
        user_name: format!("web-user-{}", &user_id[..8.min(user_id.len())]),
        text: user_message,
        message_id: None,
    };

    // Dispatch through the unified pipeline
    // This gives us: sessions, identities, memories, tool execution, gateway events
    let result = state.dispatcher.dispatch(normalized).await;

    if let Some(error) = result.error {
        log::error!("Chat dispatch error: {}", error);
        return HttpResponse::InternalServerError().json(ChatResponse {
            success: false,
            message: None,
            error: Some(error),
            session_id: None,
        });
    }

    HttpResponse::Ok().json(ChatResponse {
        success: true,
        message: Some(ChatMessage {
            role: "assistant".to_string(),
            content: result.response,
        }),
        error: None,
        session_id: None, // Could return session ID if needed
    })
}
