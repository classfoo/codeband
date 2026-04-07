use crate::tools::{
    driver::CodingToolDriver,
    model::{FieldType, ToolFieldSchema, ToolFormSchema, ToolKind},
};
use serde_json::{json, Value};

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
        json!({"command":"codex","model":"gpt-5-codex"})
    }
    fn validate(&self, config: &Value) -> anyhow::Result<()> {
        if config.get("command").and_then(Value::as_str).unwrap_or("").is_empty() {
            anyhow::bail!("command is required");
        }
        Ok(())
    }
}
