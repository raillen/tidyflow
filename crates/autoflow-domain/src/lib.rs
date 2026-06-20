pub mod audit;
pub mod blueprint;
pub mod counter;
pub mod error;
pub mod execution;
pub mod file_filter;
pub mod filter;
pub mod job;
pub mod tokenizer;
pub mod notify;
pub mod path_policy;
pub mod schedule;
pub mod scripts;
pub mod settings;
pub mod transfer;
pub mod watch;

pub use audit::{AuditEntry, AuditStatus, NewAuditEntry};
pub use blueprint::{
    Blueprint, BlueprintCollision, BlueprintKind, BlueprintOperation, BlueprintPlanSample,
    BlueprintSimulationReport, BlueprintSummary, FolderNode, FolderPlan, FolderPlanPreview,
    FolderPlanPreviewNode, RoutingConfig,
    TemplatePreview,
};
pub use counter::{CounterConfig, CounterScope};
pub use tokenizer::{evaluate, TemplatePipeline, TemplateSegment, TokenContext, TokenError};
pub use error::DomainError;
pub use execution::{
    ActiveExecution, ExecutionCompleted, ExecutionProgress, SimulationReport, SimulationSample,
};
pub use file_filter::{built_in_exclude_presets, FileFilter, SymlinkMode};
pub use filter::{collect_files, should_process_file, FilterEngine};
pub use job::{ConflictStrategy, Job, JobMode, JobSummary};
pub use notify::{NotifyChannel, NotifyConfig, NotifyEvent};
pub use path_policy::{is_path_authorized, is_path_under_root, normalize_path};
pub use schedule::{ScheduleConfig, ScheduleRule};
pub use scripts::ScriptsConfig;
pub use settings::{AppSettings, HealthStatus, ThemeMode};
pub use transfer::TransferOptions;
pub use watch::{WatchConfig, WatchDetectionMode, WatchEventKind};

pub fn file_hash(path: &std::path::Path) -> Result<String, std::io::Error> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 65536];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

pub fn files_equal(path_a: &std::path::Path, path_b: &std::path::Path, strict: bool) -> Result<bool, String> {
    let meta_a = std::fs::metadata(path_a).map_err(|e| e.to_string())?;
    let meta_b = std::fs::metadata(path_b).map_err(|e| e.to_string())?;
    if meta_a.len() != meta_b.len() {
        return Ok(false);
    }
    if !strict {
        if let (Ok(a), Ok(b)) = (meta_a.modified(), meta_b.modified()) {
            if a == b {
                return Ok(true);
            }
        }
    }
    Ok(file_hash(path_a).map_err(|e| e.to_string())? == file_hash(path_b).map_err(|e| e.to_string())?)
}

