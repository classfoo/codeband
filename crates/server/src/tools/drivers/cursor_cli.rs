use crate::tools::{
    driver::CodingToolDriver,
    model::{FieldType, ToolFieldSchema, ToolFormSchema, ToolKind},
};
use serde_json::{json, Value};

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
        json!({"command":"cursor","profile":"default"})
    }
    fn validate(&self, config: &Value) -> anyhow::Result<()> {
        if config.get("command").and_then(Value::as_str).unwrap_or("").is_empty() {
            anyhow::bail!("command is required");
        }
        Ok(())
    }
}
