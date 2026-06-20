use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionProgress {
    pub execution_id: Uuid,
    pub job_id: Uuid,
    pub job_name: String,
    pub current_file: String,
    pub percent: f64,
    pub bytes_per_sec: f64,
    pub recent_log: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionCompleted {
    pub execution_id: Uuid,
    pub job_id: Uuid,
    pub success: bool,
    pub processed: u32,
    pub failed: u32,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveExecution {
    pub execution_id: Uuid,
    pub job_id: Uuid,
    pub job_name: String,
    pub current_file: String,
    pub percent: f64,
    pub bytes_per_sec: f64,
    pub recent_log: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationReport {
    pub files_matched: u32,
    pub files_skipped: u32,
    pub sample: Vec<SimulationSample>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationSample {
    pub source: String,
    pub target: String,
    pub action: String,
}
