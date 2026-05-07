use crate::{i18n, AppState};
use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Deserialize)]
pub(super) struct InitGitProjectRequest {
    pub project: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct InitGitProjectResponse {
    pub project: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct ExecGitCommandRequest {
    pub project: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ExecGitCommandResponse {
    pub stdout: String,
}

fn workspace_root(state: &AppState) -> Option<PathBuf> {
    state
        .workspace
        .read()
        .expect("workspace lock poisoned")
        .path
        .clone()
}

fn validate_project_name(raw: &str) -> anyhow::Result<String> {
    let project = raw.trim();
    if project.is_empty() {
        anyhow::bail!("git_project_empty");
    }
    if !project
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        anyhow::bail!("git_project_invalid");
    }
    Ok(project.to_string())
}

fn resolve_project_path(workspace: &Path, project: &str) -> anyhow::Result<PathBuf> {
    let safe_project = validate_project_name(project)?;
    Ok(workspace.join(safe_project))
}

fn run_git(project_dir: &Path, args: &[String]) -> anyhow::Result<String> {
    if args.is_empty() {
        anyhow::bail!("git_args_empty");
    }
    let output = Command::new("git")
        .current_dir(project_dir)
        .args(args)
        .output()?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        anyhow::bail!("git command failed");
    }
    anyhow::bail!(stderr);
}

pub(super) fn init_repository(workspace: &Path, project: &str) -> anyhow::Result<String> {
    let project_path = resolve_project_path(workspace, project)?;
    if !project_path.exists() {
        fs::create_dir_all(&project_path)?;
    }
    run_git(&project_path, &[String::from("init")])
}

pub(super) fn exec_git_command(workspace: &Path, project: &str, args: &[String]) -> anyhow::Result<String> {
    let project_path = resolve_project_path(workspace, project)?;
    if !project_path.exists() {
        anyhow::bail!("git_project_not_found");
    }
    run_git(&project_path, args)
}

pub(super) async fn init_git_project(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<InitGitProjectRequest>,
) -> Result<Json<InitGitProjectResponse>, (axum::http::StatusCode, String)> {
    let Some(workspace) = workspace_root(&state) else {
        return Err((
            axum::http::StatusCode::CONFLICT,
            i18n::msg(&headers, "workspace_not_configured"),
        ));
    };

    let project = validate_project_name(&payload.project).map_err(|err| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            i18n::msg(&headers, &err.to_string()),
        )
    })?;

    init_repository(&workspace, &project)
        .map_err(|err| (axum::http::StatusCode::BAD_REQUEST, err.to_string()))?;
    let path = workspace.join(&project).to_string_lossy().to_string();
    Ok(Json(InitGitProjectResponse { project, path }))
}

pub(super) async fn exec_git(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<ExecGitCommandRequest>,
) -> Result<Json<ExecGitCommandResponse>, (axum::http::StatusCode, String)> {
    let Some(workspace) = workspace_root(&state) else {
        return Err((
            axum::http::StatusCode::CONFLICT,
            i18n::msg(&headers, "workspace_not_configured"),
        ));
    };

    let project = validate_project_name(&payload.project).map_err(|err| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            i18n::msg(&headers, &err.to_string()),
        )
    })?;

    if payload.args.is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            i18n::msg(&headers, "git_args_empty"),
        ));
    }

    let stdout = exec_git_command(&workspace, &project, &payload.args).map_err(|err| {
        let key = err.to_string();
        if key == "git_project_not_found" {
            return (axum::http::StatusCode::NOT_FOUND, i18n::msg(&headers, "git_project_not_found"));
        }
        (axum::http::StatusCode::BAD_REQUEST, key)
    })?;
    Ok(Json(ExecGitCommandResponse { stdout }))
}

#[cfg(test)]
mod tests {
    use super::{exec_git_command, init_repository};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock went backwards")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[test]
    fn init_repository_creates_project_with_git_dir() {
        let workspace = unique_temp_dir("kaisha-git-init");
        fs::create_dir_all(&workspace).expect("failed to create workspace");

        let result = init_repository(&workspace, "demo-project");
        assert!(result.is_ok(), "init should succeed");

        let project = workspace.join("demo-project");
        assert!(project.join(".git").exists(), "git metadata should exist");
    }

    #[test]
    fn exec_git_command_runs_status_inside_project() {
        let workspace = unique_temp_dir("kaisha-git-exec");
        fs::create_dir_all(&workspace).expect("failed to create workspace");
        init_repository(&workspace, "demo-project").expect("init should succeed");

        let args = vec!["status".to_string(), "--short".to_string()];
        let output = exec_git_command(&workspace, "demo-project", &args).expect("command should succeed");
        assert!(output.is_empty(), "new repo should have clean short status");
    }

    #[test]
    fn init_repository_rejects_invalid_project_name() {
        let workspace = unique_temp_dir("kaisha-git-invalid-project");
        fs::create_dir_all(&workspace).expect("failed to create workspace");

        let result = init_repository(&workspace, "../bad");
        assert!(result.is_err(), "invalid project should fail");
    }
}
