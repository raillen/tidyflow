use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use autoflow_domain::{
    evaluate, is_path_under_root, normalize_path, AuditStatus, Blueprint, BlueprintCollision,
    BlueprintKind, BlueprintOperation, BlueprintPlanSample, BlueprintSimulationReport,
    ConflictStrategy, DomainError, FolderPlan, FolderPlanPreview, FolderPlanPreviewNode, Job,
    NewAuditEntry, TemplatePreview, TokenContext,
};
use chrono::Utc;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::ports::AuditStore;
use crate::BlueprintStore;

pub async fn list_blueprints(store: &dyn BlueprintStore) -> Result<Vec<autoflow_domain::BlueprintSummary>, DomainError> {
    store.list().await
}

pub async fn get_blueprint(store: &dyn BlueprintStore, id: Uuid) -> Result<Blueprint, DomainError> {
    store.get(id).await
}

pub async fn create_blueprint(
    store: &dyn BlueprintStore,
    mut blueprint: Blueprint,
) -> Result<Blueprint, DomainError> {
    blueprint.normalize();
    blueprint.validate()?;
    store.save(&blueprint).await?;
    Ok(blueprint)
}

pub async fn update_blueprint(
    store: &dyn BlueprintStore,
    mut blueprint: Blueprint,
) -> Result<Blueprint, DomainError> {
    blueprint.normalize();
    blueprint.validate()?;
    store.save(&blueprint).await?;
    Ok(blueprint)
}

pub async fn delete_blueprint(store: &dyn BlueprintStore, id: Uuid) -> Result<(), DomainError> {
    store.delete(id).await
}

pub async fn simulate_blueprint(
    store: &dyn BlueprintStore,
    id: Uuid,
) -> Result<BlueprintSimulationReport, DomainError> {
    let mut blueprint = store.get(id).await?;
    blueprint.normalize();
    simulate_blueprint_data(store, &blueprint).await
}

pub async fn simulate_blueprint_data(
    store: &dyn BlueprintStore,
    blueprint: &Blueprint,
) -> Result<BlueprintSimulationReport, DomainError> {
    let root = PathBuf::from(&blueprint.root_path);
    if !root.exists() {
        return Err(DomainError::SourceNotFound(blueprint.root_path.clone()));
    }

    let plan = build_plan(store, blueprint, true).await?;
    let mut collisions = Vec::new();
    let mut sample = Vec::new();

    for (index, item) in plan.iter().enumerate() {
        if item.target.exists() && item.target != item.source {
            collisions.push(BlueprintCollision {
                source: item.source.display().to_string(),
                target: item.target.display().to_string(),
            });
        }
        if index < 25 {
            sample.push(BlueprintPlanSample {
                source: item.source.display().to_string(),
                target: item.target.display().to_string(),
                action: format!("{:?}", blueprint.operation).to_lowercase(),
            });
        }
    }

    Ok(BlueprintSimulationReport {
        matched: plan.len() as u32,
        skipped: 0,
        plan_sample: sample,
        warnings: Vec::new(),
        collisions,
    })
}

pub async fn apply_blueprint(
    store: &dyn BlueprintStore,
    audit: &dyn AuditStore,
    id: Uuid,
) -> Result<(u32, u32), DomainError> {
    let mut blueprint = store.get(id).await?;
    blueprint.normalize();

    if !blueprint.enabled {
        return Err(DomainError::Validation("blueprint is disabled".into()));
    }

    let root = PathBuf::from(&blueprint.root_path);
    if !root.exists() {
        return Err(DomainError::SourceNotFound(blueprint.root_path.clone()));
    }

    let plan = build_plan(store, &blueprint, false).await?;
    let mut processed = 0u32;
    let mut failed = 0u32;

    for item in plan {
        let started = Instant::now();
        let result = execute_plan_item(&blueprint, &item);

        let duration_ms = started.elapsed().as_secs_f64() * 1000.0;
        match result {
            Ok(Some(target)) => {
                processed += 1;
                audit
                    .append(NewAuditEntry {
                        job_id: None,
                        blueprint_id: Some(blueprint.id),
                        job_name: blueprint.name.clone(),
                        source_path: item.source.display().to_string(),
                        target_path: target.display().to_string(),
                        status: AuditStatus::Organized,
                        file_size: file_size(&item.source),
                        duration_ms,
                        details: None,
                    })
                    .await?;
            }
            Ok(None) => {
                audit
                    .append(NewAuditEntry {
                        job_id: None,
                        blueprint_id: Some(blueprint.id),
                        job_name: blueprint.name.clone(),
                        source_path: item.source.display().to_string(),
                        target_path: item.target.display().to_string(),
                        status: AuditStatus::Ignored,
                        file_size: file_size(&item.source),
                        duration_ms,
                        details: Some("skipped due to conflict strategy".into()),
                    })
                    .await?;
            }
            Err(details) => {
                failed += 1;
                audit
                    .append(NewAuditEntry {
                        job_id: None,
                        blueprint_id: Some(blueprint.id),
                        job_name: blueprint.name.clone(),
                        source_path: item.source.display().to_string(),
                        target_path: item.target.display().to_string(),
                        status: AuditStatus::Failed,
                        file_size: 0,
                        duration_ms,
                        details: Some(details),
                    })
                    .await?;
            }
        }
    }

    store.update_last_run(blueprint.id).await?;
    Ok((processed, failed))
}

pub fn preview_template(
    pipeline: autoflow_domain::TemplatePipeline,
    sample_path: String,
) -> TemplatePreview {
    let path = PathBuf::from(&sample_path);
    let ctx = TokenContext::from_path(&path, 1, 1);
    let mut warnings = Vec::new();

    match evaluate(&pipeline, &ctx) {
        Ok(result) => {
            let result_path = result.replace('\\', "/");
            let result_name = Path::new(&result_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&result_path)
                .to_string();
            let valid = !result_path.contains("..") && !Path::new(&result_path).is_absolute();
            if !valid {
                warnings.push("result contains invalid path segments".into());
            }
            TemplatePreview {
                result_path,
                result_name,
                valid,
                warnings,
            }
        }
        Err(error) => TemplatePreview {
            result_path: String::new(),
            result_name: String::new(),
            valid: false,
            warnings: vec![error.to_string()],
        },
    }
}

pub fn preview_folder_plan(root_path: String, folder_plan: FolderPlan) -> FolderPlanPreview {
    let mut warnings = Vec::new();
    let mut folder_count = 0u32;

    if root_path.trim().is_empty() {
        warnings.push("root_path is required".into());
    }

    let nodes = build_folder_plan_preview_nodes(&folder_plan.nodes, "", &mut warnings, &mut folder_count);
    let valid = warnings.is_empty();

    FolderPlanPreview {
        root_path,
        nodes,
        folder_count,
        valid,
        warnings,
    }
}

fn build_folder_plan_preview_nodes(
    nodes: &[autoflow_domain::FolderNode],
    parent_rel: &str,
    warnings: &mut Vec<String>,
    folder_count: &mut u32,
) -> Vec<FolderPlanPreviewNode> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for node in nodes {
        let name = node.name.trim();
        if name.is_empty() {
            warnings.push("folder name cannot be empty".into());
            continue;
        }
        if folder_name_invalid(name) {
            warnings.push(format!("invalid folder name: {name}"));
        }
        if !seen.insert(name.to_ascii_lowercase()) {
            warnings.push(format!("duplicate sibling folder: {name}"));
        }

        let relative_path = if parent_rel.is_empty() {
            name.to_string()
        } else {
            format!("{parent_rel}/{name}")
        };

        *folder_count += 1;
        let children = build_folder_plan_preview_nodes(
            &node.children,
            &relative_path,
            warnings,
            folder_count,
        );

        result.push(FolderPlanPreviewNode {
            name: name.to_string(),
            relative_path,
            children,
        });
    }

    result
}

fn folder_name_invalid(name: &str) -> bool {
    const INVALID: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    name.is_empty()
        || name.ends_with('.')
        || name.ends_with(' ')
        || name.chars().any(|c| INVALID.contains(&c))
}

struct PlanItem {
    source: PathBuf,
    target: PathBuf,
}

async fn build_plan(
    store: &dyn BlueprintStore,
    blueprint: &Blueprint,
    dry_run: bool,
) -> Result<Vec<PlanItem>, DomainError> {
    let root = PathBuf::from(&blueprint.root_path);
    let canonical_root = normalize_path(&root);

    let candidates = match blueprint.kind {
        BlueprintKind::File => collect_files_for_blueprint(blueprint)?,
        BlueprintKind::Folder => collect_folders_for_blueprint(blueprint)?,
    };

    let mut plan = Vec::new();
    for (index, source) in candidates.into_iter().enumerate() {
        let relative_dest =
            resolve_destination(store, blueprint, &source, index as u64, dry_run).await?;
        let target = root.join(&relative_dest);

        if !is_path_under_root(&target, &canonical_root) {
            return Err(DomainError::Validation(format!(
                "resolved target escapes root: {}",
                target.display()
            )));
        }

        if source == target {
            continue;
        }

        plan.push(PlanItem { source, target });
    }

    Ok(plan)
}

async fn resolve_destination(
    store: &dyn BlueprintStore,
    blueprint: &Blueprint,
    source: &Path,
    index: u64,
    dry_run: bool,
) -> Result<String, DomainError> {
    let root = Path::new(&blueprint.root_path);
    let parent = source
        .parent()
        .and_then(|p| p.strip_prefix(root).ok())
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let folder = source
        .parent()
        .and_then(|p| p.strip_prefix(root).ok())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let date_key = Utc::now().format("%Y-%m-%d").to_string();
    let scope_key = blueprint.counter.scope_key(parent, &folder, &date_key);
    let counter_value = store.get_counter(blueprint.id, &scope_key).await?;
    let counter = if counter_value == 0 {
        blueprint.counter.start
    } else {
        counter_value
    };

    let counter_display = blueprint.counter.format_value(counter);
    let mut ctx = TokenContext::from_path(source, index + 1, counter);
    ctx.counter_formatted = counter_display;

    let mut relative = evaluate(&blueprint.routing.path_template, &ctx)
        .map_err(|e| DomainError::Validation(e.to_string()))?;

    if let Some(rename) = &blueprint.rename_template {
        let name = evaluate(rename, &ctx).map_err(|e| DomainError::Validation(e.to_string()))?;
        if let Some(parent_dir) = Path::new(&relative).parent() {
            relative = parent_dir.join(&name).to_string_lossy().to_string();
        } else {
            relative = name;
        }
    }

    relative = relative.replace('\\', "/");
    if relative.contains("..") || Path::new(&relative).is_absolute() {
        return Err(DomainError::Validation(
            "template resolved to invalid relative path".into(),
        ));
    }

    if !dry_run {
        store
            .set_counter(blueprint.id, &scope_key, counter + 1)
            .await?;
    }

    Ok(relative)
}

fn collect_files_for_blueprint(blueprint: &Blueprint) -> Result<Vec<PathBuf>, DomainError> {
    let job = blueprint_search_job(blueprint);
    autoflow_domain::collect_files(&job).map_err(|e| DomainError::Database(e.to_string()))
}

fn collect_folders_for_blueprint(blueprint: &Blueprint) -> Result<Vec<PathBuf>, DomainError> {
    let root = PathBuf::from(&blueprint.root_path);
    if !root.exists() {
        return Ok(Vec::new());
    }

    let job = blueprint_search_job(blueprint);
    let mut folders = Vec::new();

    if blueprint.recursive {
        for entry in WalkDir::new(&root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_dir() {
                continue;
            }
            let path = entry.path().to_path_buf();
            if path == root {
                continue;
            }
            if folder_matches(&job, &root, &path)? {
                folders.push(path);
            }
        }
    } else if root.is_dir() {
        for entry in fs::read_dir(&root).map_err(|e| DomainError::Database(e.to_string()))? {
            let entry = entry.map_err(|e| DomainError::Database(e.to_string()))?;
            let path = entry.path();
            if path.is_dir() && folder_matches(&job, &root, &path)? {
                folders.push(path);
            }
        }
    }

    folders.sort();
    Ok(folders)
}

fn folder_matches(job: &Job, root: &Path, folder: &Path) -> Result<bool, DomainError> {
    let filters = &job.filters;
    let name = folder.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if !filters.include_hidden && name.starts_with('.') {
        return Ok(false);
    }

    if let Some(max_depth) = filters.max_depth {
        let depth = folder
            .strip_prefix(root)
            .map(|rel| rel.components().count() as u32)
            .unwrap_or(0);
        if depth > max_depth {
            return Ok(false);
        }
    }

    let path_str = folder.to_string_lossy();
    let mut builder = globset::GlobSetBuilder::new();
    for pattern in filters.expanded_exclude_patterns() {
        let glob = globset::Glob::new(&pattern)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        builder.add(glob);
    }
    if let Ok(exclude) = builder.build() {
        if exclude.is_match(path_str.as_ref()) {
            return Ok(false);
        }
    }

    if let Some(regex) = &filters.name_regex {
        let re = regex::Regex::new(regex).map_err(|e| DomainError::Validation(e.to_string()))?;
        if !re.is_match(name) {
            return Ok(false);
        }
    }

    if let Some(regex) = &filters.path_regex {
        let re = regex::Regex::new(regex).map_err(|e| DomainError::Validation(e.to_string()))?;
        if !re.is_match(&path_str) {
            return Ok(false);
        }
    }

    Ok(true)
}

fn blueprint_search_job(blueprint: &Blueprint) -> Job {
    let mut job = Job::new(&blueprint.name, &blueprint.root_path, &blueprint.root_path);
    job.filters = blueprint.search.clone();
    job.filters.recursive = blueprint.recursive;
    job
}

fn execute_plan_item(blueprint: &Blueprint, item: &PlanItem) -> Result<Option<PathBuf>, String> {
    let resolved = resolve_conflict(&item.target, blueprint.conflict)?;
    let Some(target) = resolved else {
        return Ok(None);
    };

    if blueprint.routing.create_intermediate_dirs {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    match blueprint.operation {
        BlueprintOperation::Copy => {
            if item.source.is_dir() {
                copy_dir_recursive(&item.source, &target)?;
            } else {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                fs::copy(&item.source, &target).map_err(|e| e.to_string())?;
            }
            Ok(Some(target))
        }
        BlueprintOperation::Move => {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::rename(&item.source, &target).map_err(|e| e.to_string())?;
            Ok(Some(target))
        }
    }
}

fn resolve_conflict(target: &Path, conflict: ConflictStrategy) -> Result<Option<PathBuf>, String> {
    if !target.exists() {
        return Ok(Some(target.to_path_buf()));
    }

    match conflict {
        ConflictStrategy::Skip => Ok(None),
        ConflictStrategy::Overwrite => Ok(Some(target.to_path_buf())),
        ConflictStrategy::Rename => {
            let parent = target.parent().unwrap_or_else(|| Path::new("."));
            let file_name = target
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("item");
            let path = Path::new(file_name);
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("item");
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();

            for i in 1..1000 {
                let candidate = parent.join(format!("{stem} ({i}){ext}"));
                if !candidate.exists() {
                    return Ok(Some(candidate));
                }
            }
            Err("could not resolve unique target name".into())
        }
    }
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src = entry.path();
        let dst = target.join(entry.file_name());
        if src.is_dir() {
            copy_dir_recursive(&src, &dst)?;
        } else {
            fs::copy(&src, &dst).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn file_size(path: &Path) -> i64 {
    fs::metadata(path).map(|m| m.len() as i64).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use autoflow_domain::FolderNode;

    #[test]
    fn preview_folder_plan_builds_relative_tree() {
        let preview = preview_folder_plan(
            "C:\\root".into(),
            FolderPlan {
                nodes: vec![FolderNode {
                    name: "2026".into(),
                    children: vec![FolderNode {
                        name: "Janeiro".into(),
                        children: vec![],
                    }],
                }],
            },
        );

        assert!(preview.valid);
        assert_eq!(preview.folder_count, 2);
        assert_eq!(preview.nodes[0].relative_path, "2026");
        assert_eq!(preview.nodes[0].children[0].relative_path, "2026/Janeiro");
    }

    #[test]
    fn preview_folder_plan_flags_invalid_names() {
        let preview = preview_folder_plan(
            "C:\\root".into(),
            FolderPlan {
                nodes: vec![FolderNode {
                    name: "bad<name".into(),
                    children: vec![],
                }],
            },
        );

        assert!(!preview.valid);
        assert!(!preview.warnings.is_empty());
    }
}
