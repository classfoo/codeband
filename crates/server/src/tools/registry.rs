use crate::tools::{
    driver::CodingToolDriver,
    drivers::{
        claude_code::ClaudeCodeDriver, codex::CodexDriver, cursor_cli::CursorCliDriver,
        kimi_cli::KimiCliDriver, qoder_cli::QoderCliDriver, qwen_code::QwenCodeDriver,
    },
    model::{ToolCatalogItem, ToolKind},
};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct ToolRegistry {
    drivers: HashMap<ToolKind, Arc<dyn CodingToolDriver>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut drivers: HashMap<ToolKind, Arc<dyn CodingToolDriver>> = HashMap::new();
        for driver in [
            Arc::new(ClaudeCodeDriver) as Arc<dyn CodingToolDriver>,
            Arc::new(QwenCodeDriver),
            Arc::new(QoderCliDriver),
            Arc::new(CursorCliDriver),
            Arc::new(KimiCliDriver),
            Arc::new(CodexDriver),
        ] {
            drivers.insert(driver.kind(), driver);
        }
        Self { drivers }
    }

    pub fn get(&self, kind: &ToolKind) -> Option<Arc<dyn CodingToolDriver>> {
        self.drivers.get(kind).cloned()
    }

    pub fn catalog(&self) -> Vec<ToolCatalogItem> {
        let mut out: Vec<_> = self
            .drivers
            .values()
            .map(|driver| ToolCatalogItem {
                kind: driver.kind(),
                display_name: driver.display_name().to_string(),
                schema: driver.schema(),
            })
            .collect();
        out.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        out
    }
}
