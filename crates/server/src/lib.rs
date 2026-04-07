use application::HealthService;
use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use domain::{SetWorkspaceRequest, WorkspaceSource, WorkspaceStatus};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

#[derive(Clone)]
struct AppState {
    health: HealthService,
    workspace: Arc<RwLock<WorkspaceState>>,
}

async fn health(State(state): State<AppState>) -> Json<domain::HealthStatus> {
    Json(state.health.get_status())
}

#[derive(Clone)]
pub struct WorkspaceInit {
    pub path: Option<PathBuf>,
    pub source: WorkspaceSource,
    pub config_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceConfigFile {
    path: String,
}

#[derive(Clone)]
struct WorkspaceState {
    source: WorkspaceSource,
    path: Option<PathBuf>,
    config_file: PathBuf,
}

impl WorkspaceState {
    fn to_status(&self) -> WorkspaceStatus {
        WorkspaceStatus {
            configured: self.path.is_some(),
            path: self.path.as_ref().map(|p| p.to_string_lossy().to_string()),
            source: self.source.clone(),
        }
    }
}

pub fn resolve_workspace_from_env(env_name: &str, config_file: PathBuf) -> anyhow::Result<WorkspaceInit> {
    if let Ok(value) = std::env::var(env_name) {
        let path = normalize_workspace_path(PathBuf::from(value.trim()))?;
        fs::create_dir_all(&path)?;
        return Ok(WorkspaceInit {
            path: Some(path),
            source: WorkspaceSource::Env,
            config_file,
        });
    }

    if config_file.exists() {
        let raw = fs::read_to_string(&config_file)?;
        let config: WorkspaceConfigFile = serde_json::from_str(&raw)?;
        let path = normalize_workspace_path(PathBuf::from(config.path))?;
        fs::create_dir_all(&path)?;
        return Ok(WorkspaceInit {
            path: Some(path),
            source: WorkspaceSource::Config,
            config_file,
        });
    }

    Ok(WorkspaceInit {
        path: None,
        source: WorkspaceSource::Unset,
        config_file,
    })
}

fn normalize_workspace_path(path: PathBuf) -> anyhow::Result<PathBuf> {
    let trimmed = PathBuf::from(path.to_string_lossy().trim().to_string());
    if trimmed.as_os_str().is_empty() {
        anyhow::bail!("workspace path cannot be empty");
    }

    Ok(if trimmed.is_absolute() {
        trimmed
    } else {
        std::env::current_dir()?.join(trimmed)
    })
}

async fn get_workspace(State(state): State<AppState>) -> Json<WorkspaceStatus> {
    let workspace = state.workspace.read().expect("workspace lock poisoned");
    Json(workspace.to_status())
}

async fn set_workspace(
    State(state): State<AppState>,
    Json(payload): Json<SetWorkspaceRequest>,
) -> Result<Json<WorkspaceStatus>, (axum::http::StatusCode, String)> {
    let mut workspace = state.workspace.write().expect("workspace lock poisoned");

    if matches!(workspace.source, WorkspaceSource::Env) {
        return Err((
            axum::http::StatusCode::CONFLICT,
            "workspace is controlled by KAISHA_WORKDIR environment variable".to_string(),
        ));
    }

    let normalized = normalize_workspace_path(PathBuf::from(payload.path.trim()))
        .map_err(|err| (axum::http::StatusCode::BAD_REQUEST, err.to_string()))?;
    fs::create_dir_all(&normalized)
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    persist_workspace_config(&workspace.config_file, &normalized)
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    workspace.path = Some(normalized);
    workspace.source = WorkspaceSource::Config;
    Ok(Json(workspace.to_status()))
}

fn persist_workspace_config(config_file: &Path, path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = config_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let payload = WorkspaceConfigFile {
        path: path.to_string_lossy().to_string(),
    };
    fs::write(config_file, serde_json::to_string_pretty(&payload)?)?;
    Ok(())
}

pub async fn run_http(addr: SocketAddr, workspace_init: WorkspaceInit) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/workspace", get(get_workspace).post(set_workspace))
        .with_state(AppState {
            health: HealthService,
            workspace: Arc::new(RwLock::new(WorkspaceState {
                source: workspace_init.source,
                path: workspace_init.path,
                config_file: workspace_init.config_file,
            })),
        });

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("HTTP API listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
