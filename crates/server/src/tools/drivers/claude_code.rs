use crate::tools::{
    driver::CodingToolDriver,
    model::{FieldType, ToolFieldSchema, ToolFormSchema, ToolKind},
};
use serde_json::{json, Value};

pub struct ClaudeCodeDriver;

impl CodingToolDriver for ClaudeCodeDriver {
    fn kind(&self) -> ToolKind {
        ToolKind::ClaudeCode
    }

    fn display_name(&self) -> &'static str {
        "Claude Code"
    }

    fn schema(&self) -> ToolFormSchema {
        ToolFormSchema {
            title: "Claude Code".to_string(),
            fields: vec![
                ToolFieldSchema {
                    key: "command".to_string(),
                    label: "Command".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    options: vec![],
                    placeholder: Some("claude".to_string()),
                },
                ToolFieldSchema {
                    key: "model".to_string(),
                    label: "Model".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    options: vec![],
                    placeholder: Some("claude-sonnet-4".to_string()),
                },
            ],
        }
    }

    fn default_config(&self) -> Value {
        json!({"command":"claude","model":"claude-sonnet-4"})
    }

    fn validate(&self, config: &Value) -> anyhow::Result<()> {
        if config.get("command").and_then(Value::as_str).unwrap_or("").is_empty() {
            anyhow::bail!("command is required");
        }
        Ok(())
    }
}
