use crate::tools::{
    driver::CodingToolDriver,
    model::{FieldType, ToolFieldSchema, ToolFormSchema, ToolKind},
};
use serde_json::{json, Value};

pub struct QoderCliDriver;

impl CodingToolDriver for QoderCliDriver {
    fn kind(&self) -> ToolKind {
        ToolKind::QoderCli
    }
    fn display_name(&self) -> &'static str {
        "Qoder CLI"
    }
    fn schema(&self) -> ToolFormSchema {
        ToolFormSchema {
            title: "Qoder CLI".to_string(),
            fields: vec![ToolFieldSchema {
                key: "command".to_string(),
                label: "Command".to_string(),
                field_type: FieldType::Text,
                required: true,
                options: vec![],
                placeholder: Some("qoder".to_string()),
            }],
        }
    }
    fn default_config(&self) -> Value {
        json!({"command":"qoder"})
    }
    fn validate(&self, config: &Value) -> anyhow::Result<()> {
        if config.get("command").and_then(Value::as_str).unwrap_or("").is_empty() {
            anyhow::bail!("command is required");
        }
        Ok(())
    }
}
