use crate::tools::model::{ToolFormSchema, ToolKind};
use serde_json::Value;

pub trait CodingToolDriver: Send + Sync {
    fn kind(&self) -> ToolKind;
    fn display_name(&self) -> &'static str;
    fn schema(&self) -> ToolFormSchema;
    fn default_config(&self) -> Value;
    fn validate(&self, config: &Value) -> anyhow::Result<()>;
}
