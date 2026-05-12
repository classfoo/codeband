use crate::tools::{
    driver::{CodingToolDriver, ToolChatMessage, ToolExecutionResult, ToolSession, ToolUsage},
    model::{FieldType, ToolFieldSchema, ToolFormSchema, ToolKind},
};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;

pub struct CursorCliDriver;

impl CodingToolDriver for CursorCliDriver {
    fn kind(&self) -> ToolKind {
        ToolKind::CursorCli
    }
    fn display_name(&self) -> &'static str {
        "Cursor CLI"
    }
    fn schema(&self) -> ToolFormSchema {
        ToolFormSchema {
            title: "Cursor CLI".to_string(),
            fields: vec![
                ToolFieldSchema {
                    key: "command".to_string(),
                    label: "Command".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    options: vec![],
                    placeholder: Some("cursor".to_string()),
                },
                ToolFieldSchema {
                    key: "profile".to_string(),
                    label: "Profile".to_string(),
                    field_type: FieldType::Text,
                    required: false,
                    options: vec![],
                    placeholder: Some("default".to_string()),
                },
            ],
        }
    }
    fn default_config(&self) -> Value {
        json!({
            "command":"cursor",
            "profile":"default",
            "model":"cursor-agent",
            "prompt_mode":"arg"
        })
    }
    fn validate(&self, config: &Value) -> anyhow::Result<()> {
        let command = config.get("command").and_then(Value::as_str).unwrap_or("").trim();
        if command.is_empty() {
            anyhow::bail!("command is required");
        }
        let prompt_mode = config
            .get("prompt_mode")
            .and_then(Value::as_str)
            .unwrap_or("arg");
        if prompt_mode != "stdin" && prompt_mode != "arg" {
            anyhow::bail!("prompt_mode must be stdin or arg");
        }
        Ok(())
    }

    fn check_installed(&self, config: &Value) -> anyhow::Result<bool> {
        self.validate(config)?;
        let command = config.get("command").and_then(Value::as_str).unwrap_or("cursor");
        Ok(Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {} >/dev/null 2>&1", command))
            .status()
            .map(|s| s.success())
            .unwrap_or(false))
    }

    fn install_tool(&self, config: &Value) -> anyhow::Result<()> {
        self.validate(config)?;
        if self.check_installed(config)? {
            return Ok(());
        }
        anyhow::bail!("cursor cli installation is platform-specific; please install manually")
    }

    fn configure_tool(&self, config: &Value) -> anyhow::Result<Value> {
        self.validate(config)?;
        let mut merged = self.default_config();
        if let (Some(src), Some(dst)) = (config.as_object(), merged.as_object_mut()) {
            for (k, v) in src {
                dst.insert(k.clone(), v.clone());
            }
        }
        Ok(merged)
    }

    fn start_tool(&self, config: &Value) -> anyhow::Result<()> {
        self.validate(config)?;
        let command = config.get("command").and_then(Value::as_str).unwrap_or("cursor");
        let status = Command::new(command).arg("--version").status();
        match status {
            Ok(s) if s.success() => Ok(()),
            Ok(_) => anyhow::bail!("cursor command failed to start"),
            Err(err) => anyhow::bail!("failed to execute cursor command: {err}"),
        }
    }

    fn create_session(&self, config: &Value) -> anyhow::Result<ToolSession> {
        self.start_tool(config)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis();
        Ok(ToolSession {
            id: format!("cursor_session_{now}"),
            started_at_ms: now,
        })
    }

    fn run_chat_for_code(
        &self,
        config: &Value,
        _session: &ToolSession,
        messages: &[ToolChatMessage],
        cwd: Option<&Path>,
    ) -> anyhow::Result<ToolExecutionResult> {
        self.validate(config)?;
        let command = config.get("command").and_then(Value::as_str).unwrap_or("cursor");
        let prompt_mode = config
            .get("prompt_mode")
            .and_then(Value::as_str)
            .unwrap_or("arg");
        let prompt = messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let output = if prompt_mode == "stdin" {
            let mut cmd = Command::new("sh");
            if let Some(dir) = cwd {
                cmd.current_dir(dir);
            }
            cmd.arg("-c")
                .arg(format!("printf %s \"$PROMPT\" | {} agent -", command))
                .env("PROMPT", prompt.clone())
                .output()?
        } else {
            let mut cmd = Command::new(command);
            if let Some(dir) = cwd {
                cmd.current_dir(dir);
            }
            cmd.arg("agent").arg(&prompt).output()?
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let merged = if stderr.trim().is_empty() {
            stdout
        } else {
            format!("{stdout}\n\n--- stderr ---\n{stderr}")
        };
        let usage = self.collect_usage(config, messages, &merged)?;
        Ok(ToolExecutionResult {
            output: merged,
            exit_code: output.status.code().unwrap_or(1),
            usage,
        })
    }

    fn collect_usage(
        &self,
        config: &Value,
        messages: &[ToolChatMessage],
        completion: &str,
    ) -> anyhow::Result<ToolUsage> {
        let prompt_chars: usize = messages
            .iter()
            .map(|m| m.role.chars().count() + m.content.chars().count())
            .sum();
        let completion_chars = completion.chars().count();
        let prompt_tokens = ((prompt_chars as f64) / 4.0).ceil() as u64;
        let completion_tokens = ((completion_chars as f64) / 4.0).ceil() as u64;
        Ok(ToolUsage {
            model: config
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or("cursor-agent")
                .to_string(),
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        })
    }
}
