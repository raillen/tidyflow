use std::fs;
use std::path::PathBuf;

use crate::ports::RollbackStore;
use autoflow_domain::{
    file_hash, AuditStatus, DomainError, NewAuditEntry, RollbackManifest, RollbackResult,
};
use chrono::Utc;
use uuid::Uuid;

use crate::ports::AuditStore;

pub async fn record_manifest(
    store: &dyn RollbackStore,
    manifest: RollbackManifest,
) -> Result<(), DomainError> {
    store.save(&manifest).await
}

pub async fn rollback_job(
    rollback: &dyn RollbackStore,
    audit: &dyn AuditStore,
    job_id: Uuid,
) -> Result<RollbackResult, DomainError> {
    let manifest = rollback
        .latest_for_job(job_id)
        .await?
        .ok_or_else(|| DomainError::Validation("no rollback manifest available".into()))?;

    if manifest.rolled_back_at.is_some() {
        return Err(DomainError::Validation("rollback already executed".into()));
    }

    let mut reverted = 0u32;
    let mut skipped = 0u32;

    for item in manifest.items.iter().rev() {
        match item.status {
            AuditStatus::Moved => {
                let source = PathBuf::from(&item.source_path);
                let target = PathBuf::from(&item.target_path);
                if target.exists() && !source.exists() {
                    if let Some(expected) = &item.target_hash {
                        if file_hash(&target).ok().as_ref() != Some(expected) {
                            skipped += 1;
                            continue;
                        }
                    }
                    if fs::rename(&target, &source).is_ok() {
                        reverted += 1;
                    } else {
                        skipped += 1;
                    }
                } else {
                    skipped += 1;
                }
            }
            AuditStatus::Copied => {
                let target = PathBuf::from(&item.target_path);
                if target.exists() {
                    if let Some(expected) = &item.target_hash {
                        if file_hash(&target).ok().as_ref() != Some(expected) {
                            skipped += 1;
                            continue;
                        }
                    }
                    let removed = if item.is_directory {
                        fs::remove_dir_all(&target).is_ok()
                    } else {
                        fs::remove_file(&target).is_ok()
                    };
                    if removed {
                        reverted += 1;
                    } else {
                        skipped += 1;
                    }
                } else {
                    skipped += 1;
                }
            }
            _ => skipped += 1,
        }
    }

    rollback.mark_rolled_back(manifest.id, Utc::now()).await?;

    let _ = audit
        .append(NewAuditEntry {
            job_id: Some(job_id),
            blueprint_id: manifest.blueprint_id,
            job_name: "Rollback".into(),
            source_path: format!("job://{job_id}"),
            target_path: format!("execution://{}", manifest.execution_id),
            status: AuditStatus::RolledBack,
            file_size: 0,
            duration_ms: 0.0,
            details: Some(format!(
                "reverted={reverted}, skipped={skipped}, execution={}",
                manifest.execution_id
            )),
        })
        .await;

    Ok(RollbackResult {
        success: true,
        reverted_count: reverted,
        skipped_count: skipped,
    })
}

pub async fn rollback_blueprint(
    rollback: &dyn RollbackStore,
    audit: &dyn AuditStore,
    blueprint_id: Uuid,
) -> Result<RollbackResult, DomainError> {
    let manifest = rollback
        .latest_for_blueprint(blueprint_id)
        .await?
        .ok_or_else(|| DomainError::Validation("no rollback manifest available".into()))?;

    if manifest.rolled_back_at.is_some() {
        return Err(DomainError::Validation("rollback already executed".into()));
    }

    let mut reverted = 0u32;
    let mut skipped = 0u32;

    for item in manifest.items.iter().rev() {
        match item.status {
            AuditStatus::Moved => {
                let source = PathBuf::from(&item.source_path);
                let target = PathBuf::from(&item.target_path);
                if target.exists() && !source.exists() {
                    if let Some(expected) = &item.target_hash {
                        if file_hash(&target).ok().as_ref() != Some(expected) {
                            skipped += 1;
                            continue;
                        }
                    }
                    if fs::rename(&target, &source).is_ok() {
                        reverted += 1;
                    } else {
                        skipped += 1;
                    }
                } else {
                    skipped += 1;
                }
            }
            AuditStatus::Copied => {
                let target = PathBuf::from(&item.target_path);
                if target.exists() {
                    if let Some(expected) = &item.target_hash {
                        if file_hash(&target).ok().as_ref() != Some(expected) {
                            skipped += 1;
                            continue;
                        }
                    }
                    let removed = if item.is_directory {
                        fs::remove_dir_all(&target).is_ok()
                    } else {
                        fs::remove_file(&target).is_ok()
                    };
                    if removed {
                        reverted += 1;
                    } else {
                        skipped += 1;
                    }
                } else {
                    skipped += 1;
                }
            }
            _ => skipped += 1,
        }
    }

    rollback.mark_rolled_back(manifest.id, Utc::now()).await?;

    let _ = audit
        .append(NewAuditEntry {
            job_id: None,
            blueprint_id: Some(blueprint_id),
            job_name: "Rollback".into(),
            source_path: format!("blueprint://{blueprint_id}"),
            target_path: format!("execution://{}", manifest.execution_id),
            status: AuditStatus::RolledBack,
            file_size: 0,
            duration_ms: 0.0,
            details: Some(format!(
                "reverted={reverted}, skipped={skipped}, execution={}",
                manifest.execution_id
            )),
        })
        .await;

    Ok(RollbackResult {
        success: true,
        reverted_count: reverted,
        skipped_count: skipped,
    })
}
