use crate::tools::model::{ToolIndexFile, ToolInstance};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct ToolStore {
    index_path: Option<PathBuf>,
}

impl ToolStore {
    pub fn new(workspace: Option<&Path>) -> anyhow::Result<Self> {
        if let Some(workspace) = workspace {
            let dir = workspace.join("settings").join("tools");
            fs::create_dir_all(&dir)?;
            let index_path = dir.join("index.yml");
            if !index_path.exists() {
                fs::write(&index_path, serde_yaml::to_string(&ToolIndexFile::default())?)?;
            }
            return Ok(Self {
                index_path: Some(index_path),
            });
        }
        Ok(Self { index_path: None })
    }

    pub fn load(&self) -> anyhow::Result<BTreeMap<String, ToolInstance>> {
        let Some(path) = self.index_path.as_ref() else {
            return Ok(BTreeMap::new());
        };
        let raw = fs::read_to_string(path)?;
        let parsed: ToolIndexFile = serde_yaml::from_str(&raw)?;
        Ok(parsed.instances)
    }

    pub fn save(&self, instances: &BTreeMap<String, ToolInstance>) -> anyhow::Result<()> {
        let Some(path) = self.index_path.as_ref() else {
            anyhow::bail!("workspace is not configured");
        };
        let payload = ToolIndexFile {
            instances: instances.clone(),
        };
        fs::write(path, serde_yaml::to_string(&payload)?)?;
        Ok(())
    }
}
