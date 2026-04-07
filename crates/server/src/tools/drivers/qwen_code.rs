use crate::tools::{
    driver::CodingToolDriver,
    model::{FieldType, ToolFieldSchema, ToolFormSchema, ToolKind},
};
use serde_json::{json, Value};

pub struct QwenCodeDriver;

impl CodingToolDriver for QwenCodeDriver {
    fn kind(&self) -> ToolKind {
        ToolKind::QwenCode
    }
    fn display_name(&self) -> &'static str {
        "Qwen Code"
    }
    fn schema(&self) -> ToolFormSchema {
        ToolFormSchema {
            title: "Qwen Code".to_string(),
            fields: vec![
                ToolFieldSchema {
                    key: "command".to_string(),
                    label: "Command".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    options: vec![],
                    placeholder: Some("qwen".to_string()),
                },
                ToolFieldSchema {
                    key: "endpoint".to_string(),
                    label: "Endpoint".to_string(),
                    field_type: FieldType::Text,
                    required: false,
                    options: vec![],
                    placeholder: Some("https://dashscope.aliyuncs.com".to_string()),
                },
            ],
        }
    }
    fn default_config(&self) -> Value {
        json!({"command":"qwen","endpoint":"https://dashscope.aliyuncs.com"})
    }
    fn validate(&self, config: &Value) -> anyhow::Result<()> {
        if config.get("command").and_then(Value::as_str).unwrap_or("").is_empty() {
            anyhow::bail!("command is required");
        }
        Ok(())
    }
}
