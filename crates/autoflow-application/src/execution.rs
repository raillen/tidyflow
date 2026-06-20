use std::path::{Path, PathBuf};
use std::time::Instant;

use autoflow_domain::{
    collect_files, files_equal, file_hash, AuditStatus, ConflictStrategy, DomainError,
    ExecutionCompleted, ExecutionProgress, Job, JobMode, NewAuditEntry,
    NotifyEvent, SimulationReport,
};
use chrono::Utc;
use uuid::Uuid;

use crate::jobs::simulate_job_data;
use crate::notify::fire_notify_event;
use crate::pack::{
    create_encrypted_package, remove_files_in_dir, resolve_encrypt_password, resolve_pack_path,
};
use crate::ports::{AuditStore, JobStore};
use crate::script_runner::{run_post_script, run_pre_script};

pub type ProgressCallback = Box<dyn Fn(ExecutionProgress) + Send + Sync>;
pub type CompletedCallback = Box<dyn Fn(ExecutionCompleted) + Send + Sync>;

pub async fn run_job(
    data_dir: &Path,
    jobs: &dyn JobStore,
    audit: &dyn AuditStore,
    job_id: Uuid,
    execution_id: Uuid,
    on_progress: ProgressCallback,
    on_completed: CompletedCallback,
    cancel: tokio_util::sync::CancellationToken,
) -> Result<(), DomainError> {
    let mut job = jobs.get(job_id).await?;
    job.normalize();

    if !job.enabled {
        return Err(DomainError::JobDisabled);
    }

    let source = PathBuf::from(&job.source_path);
    if !source.exists() {
        return Err(DomainError::SourceNotFound(job.source_path.clone()));
    }

    std::fs::create_dir_all(&job.target_path)
        .map_err(|e| DomainError::Database(e.to_string()))?;

    fire_notify_event(&job, NotifyEvent::Started, None);

    if let Err(error) = run_pre_script(data_dir, &job).await {
        fire_notify_event(&job, NotifyEvent::Failed, Some(&error.to_string()));
        on_completed(ExecutionCompleted {
            execution_id,
            job_id,
            success: false,
            processed: 0,
            failed: 0,
            error_message: Some(error.to_string()),
        });
        return Err(error);
    }

    let files = collect_files(&job).map_err(|e| DomainError::Database(e.to_string()))?;
    let total = files.len().max(1) as f64;
    let mut recent_log = Vec::new();
    let mut processed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;
    let mut aborted = false;

    for (index, file) in files.iter().enumerate() {
        if cancel.is_cancelled() {
            break;
        }

        let started = Instant::now();
        let file_name = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        on_progress(ExecutionProgress {
            execution_id,
            job_id,
            job_name: job.name.clone(),
            current_file: file_name.clone(),
            percent: (index as f64 / total) * 100.0,
            bytes_per_sec: 0.0,
            recent_log: recent_log.clone(),
        });

        let target_dir = PathBuf::from(&job.target_path);
        let result = process_file(&job, file, &target_dir, &source);

        let duration_ms = started.elapsed().as_secs_f64() * 1000.0;
        match result {
            Ok((target_path, status)) => {
                if status == AuditStatus::Ignored {
                    skipped += 1;
                } else {
                    processed += 1;
                }
                let line = format!("{file_name} -> {}", status_label(status));
                recent_log.insert(0, line);
                if recent_log.len() > 10 {
                    recent_log.pop();
                }

                audit
                    .append(NewAuditEntry {
                        job_id: Some(job_id),
                        blueprint_id: None,
                        job_name: job.name.clone(),
                        source_path: file.display().to_string(),
                        target_path: target_path.display().to_string(),
                        status,
                        file_size: file.metadata().map(|m| m.len() as i64).unwrap_or(0),
                        duration_ms,
                        details: None,
                    })
                    .await?;
            }
            Err(details) => {
                failed += 1;
                recent_log.insert(0, format!("FALHA {file_name}"));
                if recent_log.len() > 10 {
                    recent_log.pop();
                }
                audit
                    .append(NewAuditEntry {
                        job_id: Some(job_id),
                        blueprint_id: None,
                        job_name: job.name.clone(),
                        source_path: file.display().to_string(),
                        target_path: target_dir.display().to_string(),
                        status: AuditStatus::Failed,
                        file_size: 0,
                        duration_ms,
                        details: Some(details.clone()),
                    })
                    .await?;

                if job.options.stop_on_integrity_error {
                    aborted = true;
                    break;
                }
            }
        }
    }

    if let Ok(Some(post_result)) = run_post_script(data_dir, &job).await {
        if post_result.timed_out || post_result.exit_code.unwrap_or(1) != 0 {
            let warning = if post_result.timed_out {
                post_result.stderr.clone()
            } else {
                format!(
                    "post-script exit {:?}: {}",
                    post_result.exit_code,
                    post_result.stderr.trim()
                )
            };
            recent_log.insert(0, format!("WARN post-script: {warning}"));
            audit
                .append(NewAuditEntry {
                    job_id: Some(job_id),
                    blueprint_id: None,
                    job_name: job.name.clone(),
                    source_path: job.source_path.clone(),
                    target_path: job.target_path.clone(),
                    status: AuditStatus::Failed,
                    file_size: 0,
                    duration_ms: 0.0,
                    details: Some(warning),
                })
                .await?;
        }
    }

    if job.options.encrypt_output && !cancel.is_cancelled() && failed == 0 {
        let password = resolve_encrypt_password(
            job.id,
            job.options.encrypt_password.as_deref(),
            job.options.remember_encrypt_password,
        )?;
        let pack_path = resolve_pack_path(
            Path::new(&job.target_path),
            job.options.pack_filename.as_deref(),
        );
        create_encrypted_package(Path::new(&job.target_path), &pack_path, &password)?;
        if job.options.remove_files_after_pack {
            remove_files_in_dir(Path::new(&job.target_path))?;
        }
    }

    job.last_run = Some(Utc::now());
    if let Some(schedule) = &job.schedule {
        if schedule.enabled {
            job.next_run = schedule.compute_next_run(Utc::now());
        }
    }
    jobs.save(&job).await?;

    let success = failed == 0 && !cancel.is_cancelled() && !aborted;
    let detail = if cancel.is_cancelled() {
        Some("cancelled".to_string())
    } else if aborted {
        Some("aborted on integrity error".to_string())
    } else if failed > 0 {
        Some(format!("{failed} file(s) failed, {skipped} skipped"))
    } else if skipped > 0 {
        Some(format!("{processed} processed, {skipped} skipped"))
    } else {
        None
    };

    if success {
        fire_notify_event(&job, NotifyEvent::Completed, detail.as_deref());
    } else {
        fire_notify_event(&job, NotifyEvent::Failed, detail.as_deref());
    }

    on_completed(ExecutionCompleted {
        execution_id,
        job_id,
        success,
        processed,
        failed,
        error_message: detail,
    });

    Ok(())
}

pub async fn simulate(store: &dyn JobStore, job_id: Uuid) -> Result<SimulationReport, DomainError> {
    let mut job = store.get(job_id).await?;
    job.normalize();
    simulate_job_data(&job)
}

fn process_file(
    job: &Job,
    source: &Path,
    target_dir: &Path,
    source_root: &Path,
) -> Result<(PathBuf, AuditStatus), String> {
    let relative = source
        .strip_prefix(source_root)
        .unwrap_or(source.file_name().map(Path::new).unwrap_or(source));
    let target_path = if source_root.is_file() {
        target_dir.join(
            source
                .file_name()
                .ok_or_else(|| "invalid file name".to_string())?,
        )
    } else {
        target_dir.join(relative)
    };

    if job.options.smart_sync && target_path.exists() {
        if files_equal(source, &target_path, job.options.strict_hash_sync)
            .map_err(|e| e.to_string())?
        {
            return Ok((target_path, AuditStatus::Ignored));
        }
    }

    let file_name = source
        .file_name()
        .ok_or_else(|| "invalid file name".to_string())?;
    let resolved = resolve_target(target_dir, file_name, job.conflict, source_root, source)?;

    let Some(resolved_path) = resolved else {
        return Ok((target_path, AuditStatus::Ignored));
    };

    if let Some(parent) = resolved_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    match job.mode {
        JobMode::Copy => {
            std::fs::copy(source, &resolved_path).map_err(|e| e.to_string())?;
            if job.options.verify_after_copy {
                verify_copy(source, &resolved_path)?;
            }
            Ok((resolved_path, AuditStatus::Copied))
        }
        JobMode::Move => {
            let source_hash = if job.options.verify_after_copy {
                Some(file_hash(source).map_err(|e| e.to_string())?)
            } else {
                None
            };
            std::fs::rename(source, &resolved_path).map_err(|e| e.to_string())?;
            if let Some(source_hash) = source_hash {
                let target_hash = file_hash(&resolved_path).map_err(|e| e.to_string())?;
                if source_hash != target_hash {
                    return Err(format!(
                        "integrity check failed after move: source hash {source_hash} != target hash {target_hash}"
                    ));
                }
            }
            Ok((resolved_path, AuditStatus::Moved))
        }
    }
}

fn verify_copy(source: &Path, target: &Path) -> Result<(), String> {
    let source_hash = file_hash(source).map_err(|e| e.to_string())?;
    let target_hash = file_hash(target).map_err(|e| e.to_string())?;
    if source_hash != target_hash {
        return Err(format!(
            "integrity check failed: source hash {source_hash} != target hash {target_hash}"
        ));
    }
    Ok(())
}

fn resolve_target(
    target_dir: &Path,
    file_name: &std::ffi::OsStr,
    conflict: ConflictStrategy,
    source_root: &Path,
    source: &Path,
) -> Result<Option<PathBuf>, String> {
    let relative = source
        .strip_prefix(source_root)
        .unwrap_or(Path::new(file_name));
    let mut candidate = if source_root.is_file() {
        target_dir.join(file_name)
    } else {
        target_dir.join(relative)
    };

    if !candidate.exists() {
        return Ok(Some(candidate));
    }

    match conflict {
        ConflictStrategy::Skip => Ok(None),
        ConflictStrategy::Overwrite => Ok(Some(candidate)),
        ConflictStrategy::Rename => {
            let parent = candidate.parent().unwrap_or(target_dir).to_path_buf();
            let stem = Path::new(file_name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            let ext = Path::new(file_name)
                .extension()
                .and_then(|s| s.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();

            for i in 1..1000 {
                candidate = parent.join(format!("{stem} ({i}){ext}"));
                if !candidate.exists() {
                    return Ok(Some(candidate));
                }
            }
            Err("could not resolve unique target name".into())
        }
    }
}

fn status_label(status: AuditStatus) -> &'static str {
    match status {
        AuditStatus::Copied => "COPIED",
        AuditStatus::Moved => "MOVED",
        AuditStatus::Ignored => "SKIPPED",
        AuditStatus::Failed => "FAILED",
        AuditStatus::Organized => "ORGANIZED",
    }
}
