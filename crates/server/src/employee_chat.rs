use crate::{
    employee::{employee_root, normalize_employee_id},
    i18n,
    tools::{driver::ToolChatMessage, manager::ToolManager},
    AppState,
};
use axum::{
    extract::{Path as AxumPath, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConversationFile {
    version: u32,
    messages: Vec<StoredMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredMessage {
    id: String,
    role: String,
    content: String,
    created_at_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WireMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at_ms: u64,
}

impl From<&StoredMessage> for WireMessage {
    fn from(m: &StoredMessage) -> Self {
        Self {
            id: m.id.clone(),
            role: m.role.clone(),
            content: m.content.clone(),
            created_at_ms: m.created_at_ms,
        }
    }
}

#[derive(Serialize)]
pub struct MessagesResponse {
    pub messages: Vec<WireMessage>,
}

#[derive(Deserialize)]
pub struct PostMessageBody {
    pub content: String,
}

#[derive(Serialize)]
pub struct PostMessageResultMeta {
    pub exit_code: i32,
    pub tool_instance_id: String,
    pub tool_kind: crate::tools::model::ToolKind,
    pub model: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Serialize)]
pub struct PostMessageResponse {
    pub messages: Vec<WireMessage>,
    pub last_result: PostMessageResultMeta,
}

fn workspace_dir(state: &AppState) -> Option<PathBuf> {
    state
        .workspace
        .read()
        .expect("workspace lock poisoned")
        .path
        .clone()
}

fn conversation_path(workspace: &Path, employee_id: &str) -> PathBuf {
    employee_root(workspace).join(employee_id).join("conversation.json")
}

fn employee_profile_path(workspace: &Path, employee_id: &str) -> PathBuf {
    employee_root(workspace).join(employee_id).join("profile.json")
}

fn load_conversation(path: &Path) -> anyhow::Result<ConversationFile> {
    if !path.exists() {
        return Ok(ConversationFile {
            version: 1,
            messages: vec![],
        });
    }
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

fn save_conversation(path: &Path, file: &ConversationFile) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(file)?)?;
    Ok(())
}

fn new_message_id(prefix: &str) -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("{prefix}_{ms}")
}

fn to_tool_chat(msgs: &[StoredMessage]) -> Vec<ToolChatMessage> {
    msgs.iter()
        .map(|m| ToolChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        })
        .collect()
}

fn map_process_error(err: String) -> &'static str {
    if err == "no_enabled_coding_tool" || err == "chat_tool_missing" {
        return "chat_tool_missing";
    }
    match err.as_str() {
        "employee_not_found" => "employee_not_found",
        "chat_prompt_empty" => "chat_prompt_empty",
        "chat_tool_missing" => "chat_tool_missing",
        _ => "chat_tool_run_failed",
    }
}

fn process_post_message(
    tools: ToolManager,
    workspace: PathBuf,
    employee_id: String,
    content: String,
) -> Result<PostMessageResponse, String> {
    let profile = employee_profile_path(&workspace, &employee_id);
    if !profile.exists() {
        return Err("employee_not_found".to_string());
    }
    let conv_path = conversation_path(&workspace, &employee_id);
    let mut conv = load_conversation(&conv_path).map_err(|e| e.to_string())?;
    let trimmed = content.trim().to_string();
    if trimmed.is_empty() {
        return Err("chat_prompt_empty".to_string());
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    conv.messages.push(StoredMessage {
        id: new_message_id("msg_user"),
        role: "user".to_string(),
        content: trimmed,
        created_at_ms: now,
    });

    let tool_line = to_tool_chat(&conv.messages);
    let (instance, exec_result) = tools.execute_code_chat(&workspace, &tool_line).map_err(|e| {
        let msg = e.root_cause().to_string();
        if msg == "no_enabled_coding_tool" {
            "chat_tool_missing".to_string()
        } else {
            tracing::warn!(error = %e, "code chat tool execution failed");
            msg
        }
    })?;

    let assistant_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    conv.messages.push(StoredMessage {
        id: new_message_id("msg_assistant"),
        role: "assistant".to_string(),
        content: exec_result.output.clone(),
        created_at_ms: assistant_now,
    });

    save_conversation(&conv_path, &conv).map_err(|e| e.to_string())?;

    let meta = PostMessageResultMeta {
        exit_code: exec_result.exit_code,
        tool_instance_id: instance.id.clone(),
        tool_kind: instance.kind.clone(),
        model: exec_result.usage.model.clone(),
        prompt_tokens: exec_result.usage.prompt_tokens,
        completion_tokens: exec_result.usage.completion_tokens,
        total_tokens: exec_result.usage.total_tokens,
    };

    Ok(PostMessageResponse {
        messages: conv.messages.iter().map(WireMessage::from).collect(),
        last_result: meta,
    })
}

fn status_for_process_key(key: &str) -> axum::http::StatusCode {
    match key {
        "employee_not_found" => axum::http::StatusCode::NOT_FOUND,
        "chat_prompt_empty" => axum::http::StatusCode::BAD_REQUEST,
        "chat_tool_missing" => axum::http::StatusCode::CONFLICT,
        _ => axum::http::StatusCode::BAD_GATEWAY,
    }
}

pub async fn get_messages(
    headers: HeaderMap,
    State(state): State<AppState>,
    AxumPath(employee_id): AxumPath<String>,
) -> Result<Json<MessagesResponse>, (axum::http::StatusCode, String)> {
    let Some(workspace) = workspace_dir(&state) else {
        return Err((
            axum::http::StatusCode::CONFLICT,
            i18n::msg(&headers, "workspace_not_configured"),
        ));
    };
    let employee_id = normalize_employee_id(&employee_id).map_err(|err| {
        let key = err.to_string();
        (
            axum::http::StatusCode::BAD_REQUEST,
            i18n::msg(&headers, key.as_str()),
        )
    })?;
    let profile = employee_profile_path(&workspace, &employee_id);
    if !profile.exists() {
        return Err((
            axum::http::StatusCode::NOT_FOUND,
            i18n::msg(&headers, "employee_not_found"),
        ));
    }
    let conv_path = conversation_path(&workspace, &employee_id);
    let conv = load_conversation(&conv_path).map_err(|_e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            i18n::msg(&headers, "chat_conversation_load_failed"),
        )
    })?;
    Ok(Json(MessagesResponse {
        messages: conv.messages.iter().map(WireMessage::from).collect(),
    }))
}

pub async fn post_message(
    headers: HeaderMap,
    State(state): State<AppState>,
    AxumPath(employee_id): AxumPath<String>,
    Json(body): Json<PostMessageBody>,
) -> Result<Json<PostMessageResponse>, (axum::http::StatusCode, String)> {
    let Some(workspace) = workspace_dir(&state) else {
        return Err((
            axum::http::StatusCode::CONFLICT,
            i18n::msg(&headers, "workspace_not_configured"),
        ));
    };
    let employee_id = normalize_employee_id(&employee_id).map_err(|err| {
        let key = err.to_string();
        (
            axum::http::StatusCode::BAD_REQUEST,
            i18n::msg(&headers, key.as_str()),
        )
    })?;
    let tools = state.tools.read().expect("tools lock poisoned").clone();
    let workspace_path = workspace;
    let content = body.content;
    let res = tokio::task::spawn_blocking(move || process_post_message(tools, workspace_path, employee_id, content))
        .await
        .map_err(|_e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                i18n::msg(&headers, "chat_blocking_task_failed"),
            )
        })?
        .map_err(|raw| {
            let key = map_process_error(raw.clone());
            (
                status_for_process_key(key),
                if key == "chat_tool_run_failed" && raw != "chat_tool_run_failed" {
                    format!("{}: {}", i18n::msg(&headers, key), raw)
                } else {
                    i18n::msg(&headers, key)
                },
            )
        })?;
    Ok(Json(res))
}
