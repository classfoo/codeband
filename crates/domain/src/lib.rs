use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status: &'static str,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self { status: "ok" }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceSource {
    Env,
    Config,
    Unset,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceStatus {
    pub configured: bool,
    pub path: Option<String>,
    pub source: WorkspaceSource,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetWorkspaceRequest {
    pub path: String,
}
