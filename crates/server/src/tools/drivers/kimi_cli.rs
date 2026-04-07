use crate::tools::{
    driver::CodingToolDriver,
    model::{FieldType, ToolFieldSchema, ToolFormSchema, ToolKind},
};
use serde_json::{json, Value};

pub struct KimiCliDriver;

impl CodingToolDriver for KimiCliDriver {
    fn kind(&self) -> ToolKind {
        ToolKind::KimiCli
    }
    fn display_name(&self) -> &'static str {
        "Kimi CLI"
    }
    fn schema(&self) -> ToolFormSchema {
        ToolFormSchema {
            title: "Kimi CLI".to_string(),
            fields: vec![
                ToolFieldSchema {
                    key: "command".to_string(),
                    label: "Command".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    options: vec![],
                    placeholder: Some("kimi".to_string()),
                },
                ToolFieldSchema {
                    key: "model".to_string(),
                    label: "Model".to_string(),
                    field_type: FieldType::Text,
                    required: false,
                    options: vec![],
                    placeholder: Some("moonshot-v1-32k".to_string()),
                },
            ],
        }
    }
    fn default_config(&self) -> Value {
        json!({"command":"kimi","model":"moonshot-v1-32k"})
    }
    fn validate(&self, config: &Value) -> anyhow::Result<()> {
        if config.get("command").and_then(Value::as_str).unwrap_or("").is_empty() {
            anyhow::bail!("command is required");
        }
        Ok(())
    }
}
