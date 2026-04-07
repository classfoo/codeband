use crate::tools::{
    model::{CreateToolInstanceRequest, ToolCatalogItem, ToolInstance, UpdateToolInstanceRequest},
    registry::ToolRegistry,
    store::ToolStore,
};
use std::{
    collections::BTreeMap,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
pub struct ToolManager {
    registry: ToolRegistry,
    store: ToolStore,
    instances: BTreeMap<String, ToolInstance>,
}

impl ToolManager {
    pub fn new(workspace: Option<&Path>) -> anyhow::Result<Self> {
        let registry = ToolRegistry::new();
        let store = ToolStore::new(workspace)?;
        let mut manager = Self {
            registry,
            store,
            instances: BTreeMap::new(),
        };
        manager.instances = manager.store.load()?;
        Ok(manager)
    }

    pub fn reload(&mut self, workspace: Option<&Path>) -> anyhow::Result<()> {
        self.store = ToolStore::new(workspace)?;
        self.instances = self.store.load()?;
        Ok(())
    }

    pub fn catalog(&self) -> Vec<ToolCatalogItem> {
        self.registry.catalog()
    }

    pub fn list(&self) -> Vec<ToolInstance> {
        self.instances.values().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<ToolInstance> {
        self.instances.get(id).cloned()
    }

    pub fn create(&mut self, req: CreateToolInstanceRequest) -> anyhow::Result<ToolInstance> {
        let driver = self
            .registry
            .get(&req.kind)
            .ok_or_else(|| anyhow::anyhow!("unsupported tool kind"))?;
        let id = format!(
            "tool_{}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
        );
        let config = driver.default_config();
        driver.validate(&config)?;

        let instance = ToolInstance {
            id: id.clone(),
            kind: req.kind,
            name: req
                .name
                .unwrap_or_else(|| format!("{} instance", driver.display_name())),
            enabled: true,
            version: 1,
            config,
        };
        self.instances.insert(id, instance.clone());
        self.store.save(&self.instances)?;
        Ok(instance)
    }

    pub fn update(&mut self, id: &str, req: UpdateToolInstanceRequest) -> anyhow::Result<ToolInstance> {
        let existing = self
            .instances
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("tool not found"))?
            .clone();
        let driver = self
            .registry
            .get(&existing.kind)
            .ok_or_else(|| anyhow::anyhow!("driver not found"))?;
        driver.validate(&req.config)?;

        let updated = ToolInstance {
            id: existing.id,
            kind: existing.kind,
            name: req.name,
            enabled: req.enabled,
            version: existing.version + 1,
            config: req.config,
        };
        self.instances.insert(id.to_string(), updated.clone());
        self.store.save(&self.instances)?;
        Ok(updated)
    }
}
