use crate::tools::{
    driver::{ChatSubprocessSpec, CodingToolDriver, ToolChatMessage, ToolSession, ToolUsage},
    model::{FieldType, ToolFieldSchema, ToolFormSchema, ToolKind},
};
use serde_json::{json, Value};
use std::process::Command;

pub struct CodexDriver;

impl CodingToolDriver for CodexDriver {
    fn kind(&self) -> ToolKind {
        ToolKind::Codex
    }
    fn display_name(&self) -> &'static str {
        "Codex"
    }
    fn schema(&self) -> ToolFormSchema {
        ToolFormSchema {
            title: "Codex".to_string(),
            fields: vec![
                ToolFieldSchema {
                    key: "command".to_string(),
                    label: "Command".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    options: vec![],
                    placeholder: Some("codex".to_string()),
                },
                ToolFieldSchema {
                    key: "model".to_string(),
                    label: "Model".to_string(),
                    field_type: FieldType::Text,
                    required: false,
                    options: vec![],
                    placeholder: Some("gpt-5-codex".to_string()),
                },
            ],
        }
    }
    fn default_config(&self) -> Value {
        json!({
            "command":"codex",
            "model":"gpt-5-codex",
            "api_key_env":"OPENAI_API_KEY",
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
        let command = config.get("command").and_then(Value::as_str).unwrap_or("codex");
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
        let status = Command::new("sh")
            .arg("-c")
            .arg("npm install -g @openai/codex")
            .status()?;
        if !status.success() {
            anyhow::bail!("failed to install Codex with npm");
        }
        if !self.check_installed(config)? {
            anyhow::bail!("codex command still not found after installation");
        }
        Ok(())
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
        let command = config.get("command").and_then(Value::as_str).unwrap_or("codex");
        let status = Command::new(command).arg("--version").status();
        match status {
            Ok(s) if s.success() => Ok(()),
            Ok(_) => anyhow::bail!("codex command failed to start"),
            Err(err) => anyhow::bail!("failed to execute codex command: {err}"),
        }
    }

    fn create_session(&self, config: &Value) -> anyhow::Result<ToolSession> {
        self.start_tool(config)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis();
        Ok(ToolSession {
            id: format!("codex_session_{now}"),
            started_at_ms: now,
        })
    }

    fn chat_subprocess_spec(&self, config: &Value, messages: &[ToolChatMessage]) -> anyhow::Result<ChatSubprocessSpec> {
        self.validate(config)?;
        let command = config.get("command").and_then(Value::as_str).unwrap_or("codex");
        let prompt_mode = config
            .get("prompt_mode")
            .and_then(Value::as_str)
            .unwrap_or("arg");
        let prompt = crate::tools::driver::join_chat_prompt(messages);

        if prompt_mode == "stdin" {
            Ok(ChatSubprocessSpec {
                program: "sh".to_string(),
                args: vec![
                    "-c".to_string(),
                    format!("printf %s \"$PROMPT\" | {} chat -", command),
                ],
                env: vec![("PROMPT".to_string(), prompt)],
            })
        } else {
            Ok(ChatSubprocessSpec {
                program: command.to_string(),
                args: vec!["chat".to_string(), prompt],
                env: vec![],
            })
        }
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
                .unwrap_or("gpt-5-codex")
                .to_string(),
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        })
    }
}
