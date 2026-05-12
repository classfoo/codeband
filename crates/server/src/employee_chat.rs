use crate::{
    employee::{employee_root, normalize_employee_id},
    i18n,
    tools::{driver::ToolChatMessage, manager::ToolManager},
    AppState,
};
use axum::{
    extract::{Path as AxumPath, State},
    http::HeaderMap,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio_stream::wrappers::ReceiverStream;

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

async fn forward_sse_deltas(
    mut delta_rx: tokio::sync::mpsc::Receiver<String>,
    tx: tokio::sync::mpsc::Sender<Result<Event, Infallible>>,
) {
    while let Some(chunk) = delta_rx.recv().await {
        let Ok(payload) = serde_json::to_string(&serde_json::json!({ "text": chunk })) else {
            continue;
        };
        if tx
            .send(Ok(Event::default().event("delta").data(payload)))
            .await
            .is_err()
        {
            break;
        }
    }
}

async fn run_stream_turn_inner(
    tools: ToolManager,
    workspace: PathBuf,
    employee_id: String,
    content: String,
    sse_tx: tokio::sync::mpsc::Sender<Result<Event, Infallible>>,
) -> Result<(), String> {
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
    save_conversation(&conv_path, &conv).map_err(|e| e.to_string())?;

    let tool_line = to_tool_chat(&conv.messages);
    let (delta_tx, delta_rx) = tokio::sync::mpsc::channel::<String>(64);
    let forward = tokio::spawn(forward_sse_deltas(delta_rx, sse_tx.clone()));

    let exec = tools
        .execute_code_chat_streaming(&workspace, &tool_line, delta_tx)
        .await
        .map_err(|e| {
            let msg = e.root_cause().to_string();
            if msg == "no_enabled_coding_tool" {
                "chat_tool_missing".to_string()
            } else {
                tracing::warn!(error = %e, "code chat tool execution failed (stream)");
                msg
            }
        });

    let (instance, tool_result) = match exec {
        Ok(v) => v,
        Err(raw) => {
            forward.abort();
            return Err(raw);
        }
    };

    if forward.await.is_err() {
        tracing::warn!("sse delta forwarder join failed");
    }

    let assistant_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    conv.messages.push(StoredMessage {
        id: new_message_id("msg_assistant"),
        role: "assistant".to_string(),
        content: tool_result.output.clone(),
        created_at_ms: assistant_now,
    });
    save_conversation(&conv_path, &conv).map_err(|e| e.to_string())?;

    let meta = PostMessageResultMeta {
        exit_code: tool_result.exit_code,
        tool_instance_id: instance.id.clone(),
        tool_kind: instance.kind.clone(),
        model: tool_result.usage.model.clone(),
        prompt_tokens: tool_result.usage.prompt_tokens,
        completion_tokens: tool_result.usage.completion_tokens,
        total_tokens: tool_result.usage.total_tokens,
    };
    let resp = PostMessageResponse {
        messages: conv.messages.iter().map(WireMessage::from).collect(),
        last_result: meta,
    };
    let data = serde_json::to_string(&resp).map_err(|e| e.to_string())?;
    let _ = sse_tx
        .send(Ok(Event::default().event("done").data(data)))
        .await;
    Ok(())
}

pub async fn post_message_stream(
    headers: HeaderMap,
    State(state): State<AppState>,
    AxumPath(employee_id): AxumPath<String>,
    Json(body): Json<PostMessageBody>,
) -> Result<Sse<ReceiverStream<Result<Event, Infallible>>>, (axum::http::StatusCode, String)> {
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

    let tools = state.tools.read().expect("tools lock poisoned").clone();
    let workspace_path = workspace;
    let content = body.content;

    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(32);

    let err_tool_missing = i18n::msg(&headers, "chat_tool_missing");
    let err_prompt_empty = i18n::msg(&headers, "chat_prompt_empty");
    let err_employee = i18n::msg(&headers, "employee_not_found");
    let err_tool_run = i18n::msg(&headers, "chat_tool_run_failed");

    tokio::spawn(async move {
        let sse_tx = tx.clone();
        let r = run_stream_turn_inner(tools, workspace_path, employee_id, content, sse_tx.clone()).await;
        if let Err(raw) = r {
            let key = map_process_error(raw.clone());
            let msg = match key {
                "employee_not_found" => err_employee.clone(),
                "chat_prompt_empty" => err_prompt_empty.clone(),
                "chat_tool_missing" => err_tool_missing.clone(),
                _ => {
                    if key == "chat_tool_run_failed" && raw != "chat_tool_run_failed" {
                        format!("{err_tool_run}: {raw}")
                    } else {
                        err_tool_run.clone()
                    }
                }
            };
            let payload = serde_json::json!({ "message": msg }).to_string();
            let _ = sse_tx
                .send(Ok(Event::default().event("error").data(payload)))
                .await;
        }
    });

    Ok(
        Sse::new(ReceiverStream::new(rx))
            .keep_alive(KeepAlive::new().interval(Duration::from_secs(20))),
    )
}
