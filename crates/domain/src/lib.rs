use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status: &'static str,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self { status: "ok" }
    }
}
