use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use autoflow_domain::{
    evaluate, is_path_under_root, normalize_path, parse_template, validate_relative_path,
    AuditStatus, Blueprint, BlueprintApplyResult, BlueprintCollision, BlueprintKind,
    BlueprintOperation, BlueprintPlanSample, BlueprintSimulationReport, ConflictStrategy,
    DomainError, FolderAction, FolderActionNode, FolderPlan, FolderPlanPreview,
    FolderPlanPreviewNode, Job, NewAuditEntry, TemplatePipeline, TemplatePreview, TokenContext,
};
use chrono::Local;
use uuid::Uuid;

use crate::ports::AuditStore;
use crate::BlueprintStore;

pub async fn list_blueprints(
    store: &dyn BlueprintStore,
) -> Result<Vec<autoflow_domain::BlueprintSummary>, DomainError> {
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
    simulate_blueprint_data(Some(store), &blueprint, &BTreeMap::new()).await
}

pub async fn simulate_blueprint_with_variables(
    store: &dyn BlueprintStore,
    id: Uuid,
    variables: BTreeMap<String, String>,
) -> Result<BlueprintSimulationReport, DomainError> {
    let mut blueprint = store.get(id).await?;
    blueprint.normalize();
    simulate_blueprint_data(Some(store), &blueprint, &variables).await
}

pub async fn simulate_blueprint_draft(
    blueprint: Blueprint,
    variables: BTreeMap<String, String>,
) -> Result<BlueprintSimulationReport, DomainError> {
    let mut blueprint = blueprint;
    blueprint.normalize();
    simulate_blueprint_data(None, &blueprint, &variables).await
}

pub async fn simulate_blueprint_data(
    store: Option<&dyn BlueprintStore>,
    blueprint: &Blueprint,
    variables: &BTreeMap<String, String>,
) -> Result<BlueprintSimulationReport, DomainError> {
    let root = PathBuf::from(&blueprint.root_path);
    if !root.exists() {
        return Err(DomainError::SourceNotFound(blueprint.root_path.clone()));
    }

    if blueprint.kind == BlueprintKind::Folder {
        let plan = build_folder_action_plan(blueprint, variables, true)?;
        return Ok(folder_action_simulation_report(&plan, blueprint.conflict));
    }

    let store = store.ok_or_else(|| {
        DomainError::Validation("file blueprint simulation requires a store".into())
    })?;
    let plan = build_plan(store, blueprint, true).await?;
    let mut collisions = Vec::new();
    let mut sample = Vec::new();
    let warnings = Vec::new();
    let mut skipped = 0u32;

    for (index, item) in plan.iter().enumerate() {
        if item.target.exists() && item.target != item.source {
            if would_skip_conflict(&item.target, blueprint.conflict) {
                skipped += 1;
            } else {
                collisions.push(BlueprintCollision {
                    source: item.source.display().to_string(),
                    target: item.target.display().to_string(),
                });
            }
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
        skipped,
        plan_sample: sample,
        warnings,
        collisions,
    })
}

pub async fn apply_blueprint(
    store: &dyn BlueprintStore,
    audit: &dyn AuditStore,
    id: Uuid,
) -> Result<BlueprintApplyResult, DomainError> {
    let blueprint = store.get(id).await?;
    let variables = folder_plan_default_variables(blueprint.folder_plan.as_ref());
    apply_blueprint_with_variables(store, audit, id, variables).await
}

pub async fn apply_blueprint_with_variables(
    store: &dyn BlueprintStore,
    audit: &dyn AuditStore,
    id: Uuid,
    variables: BTreeMap<String, String>,
) -> Result<BlueprintApplyResult, DomainError> {
    let mut blueprint = store.get(id).await?;
    blueprint.normalize();

    if !blueprint.enabled {
        return Err(DomainError::Validation("blueprint is disabled".into()));
    }

    let root = PathBuf::from(&blueprint.root_path);
    if !root.exists() {
        return Err(DomainError::SourceNotFound(blueprint.root_path.clone()));
    }

    if blueprint.kind == BlueprintKind::Folder {
        let result = apply_folder_blueprint(audit, &blueprint, &variables).await?;
        store.update_last_run(blueprint.id).await?;
        return Ok(result);
    }

    let plan = build_plan(store, &blueprint, false).await?;
    let mut processed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;

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
                skipped += 1;
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
    Ok(BlueprintApplyResult {
        processed,
        failed,
        skipped,
    })
}

pub fn preview_template(
    pipeline: autoflow_domain::TemplatePipeline,
    rename_template: Option<autoflow_domain::TemplatePipeline>,
    sample_path: String,
) -> TemplatePreview {
    let path = PathBuf::from(&sample_path);
    let ctx = TokenContext::from_path(&path, 1, 1);
    let mut warnings = Vec::new();

    let evaluated = evaluate(&pipeline, &ctx).and_then(|result| {
        if let Some(rename) = &rename_template {
            let name = evaluate(rename, &ctx)?;
            let result = if let Some(parent) = Path::new(&result).parent() {
                parent.join(&name).to_string_lossy().to_string()
            } else {
                name
            };
            Ok(result)
        } else {
            Ok(result)
        }
    });

    match evaluated {
        Ok(result) => {
            let result_path = result.replace('\\', "/");
            let result_name = Path::new(&result_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&result_path)
                .to_string();
            let valid = match validate_relative_path(&result_path) {
                Ok(()) => true,
                Err(error) => {
                    warnings.push(error);
                    false
                }
            };
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

pub fn preview_folder_plan(
    root_path: String,
    folder_plan: FolderPlan,
    variables: BTreeMap<String, String>,
) -> FolderPlanPreview {
    let mut warnings = Vec::new();
    let mut folder_count = 0u32;

    if root_path.trim().is_empty() {
        warnings.push("root_path is required".into());
    }

    let variable_values =
        folder_plan_variable_values(&folder_plan, &variables, true, &mut warnings);
    let root = PathBuf::from(&root_path);
    let nodes = build_folder_plan_preview_nodes(
        &folder_plan.nodes,
        "",
        &root,
        &variable_values,
        &mut warnings,
        &mut folder_count,
    );
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
    nodes: &[FolderActionNode],
    parent_rel: &str,
    root: &Path,
    variables: &BTreeMap<String, String>,
    warnings: &mut Vec<String>,
    folder_count: &mut u32,
) -> Vec<FolderPlanPreviewNode> {
    let mut result = Vec::new();

    for (index, node) in nodes.iter().enumerate() {
        let preview = preview_folder_node(node, parent_rel, root, variables, warnings, index);
        let relative_path = preview.relative_path.clone();

        *folder_count += 1;
        let children = if matches!(node.action, FolderAction::Delete) {
            if !node.children.is_empty() {
                warnings.push(format!(
                    "children ignored after delete action: {}",
                    display_action_path(node, &relative_path)
                ));
            }
            Vec::new()
        } else {
            build_folder_plan_preview_nodes(
                &node.children,
                &relative_path,
                root,
                variables,
                warnings,
                folder_count,
            )
        };

        result.push(FolderPlanPreviewNode {
            name: preview.name,
            relative_path,
            action: node.action,
            source_path: preview.source_path,
            children,
        });
    }

    result
}

struct FolderNodePreview {
    name: String,
    source_path: String,
    relative_path: String,
}

fn preview_folder_node(
    node: &FolderActionNode,
    parent_rel: &str,
    root: &Path,
    variables: &BTreeMap<String, String>,
    warnings: &mut Vec<String>,
    index: usize,
) -> FolderNodePreview {
    let ctx = folder_token_context(root, index as u64 + 1, variables);
    let name = evaluate_folder_template(&node.name, &ctx, warnings, "name", false)
        .unwrap_or_else(|_| node.name.clone());
    let source_path = evaluate_folder_template(&node.source_path, &ctx, warnings, "source", false)
        .unwrap_or_else(|_| node.source_path.clone());
    let target_path = evaluate_folder_template(&node.target_path, &ctx, warnings, "target", false)
        .unwrap_or_else(|_| node.target_path.clone());

    let relative_path = match node.action {
        FolderAction::Create => {
            if !target_path.trim().is_empty() {
                normalize_relative_display(&target_path)
            } else if parent_rel.is_empty() {
                normalize_relative_display(&name)
            } else if name.trim().is_empty() {
                parent_rel.to_string()
            } else {
                normalize_relative_display(&format!("{parent_rel}/{name}"))
            }
        }
        FolderAction::Rename => {
            let parent = Path::new(&source_path)
                .parent()
                .map(|path| path.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            if parent.is_empty() {
                normalize_relative_display(&name)
            } else {
                normalize_relative_display(&format!("{parent}/{name}"))
            }
        }
        FolderAction::Move | FolderAction::Copy => normalize_relative_display(&target_path),
        FolderAction::Delete => normalize_relative_display(&source_path),
    };

    if relative_path.trim().is_empty() {
        warnings.push(format!(
            "folder action target cannot be empty: {:?}",
            node.action
        ));
    }
    if relative_path_invalid(&relative_path) {
        warnings.push(format!("invalid relative folder path: {relative_path}"));
    }
    if !source_path.trim().is_empty() && relative_path_invalid(&source_path) {
        warnings.push(format!("invalid relative source path: {source_path}"));
    }

    FolderNodePreview {
        name,
        source_path: normalize_relative_display(&source_path),
        relative_path,
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FolderActionPlan {
    pub(crate) items: Vec<FolderActionPlanItem>,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct FolderActionPlanItem {
    pub(crate) action: FolderAction,
    pub(crate) source: Option<PathBuf>,
    pub(crate) target: Option<PathBuf>,
}

fn folder_action_simulation_report(
    plan: &FolderActionPlan,
    conflict: ConflictStrategy,
) -> BlueprintSimulationReport {
    let mut collisions = Vec::new();
    let mut sample = Vec::new();
    let mut warnings = plan.warnings.clone();
    let mut skipped = 0u32;

    for item in &plan.items {
        if matches!(
            item.action,
            FolderAction::Create | FolderAction::Rename | FolderAction::Move | FolderAction::Copy
        ) {
            if let Some(target) = &item.target {
                if target.exists() {
                    if item.action == FolderAction::Create
                        && conflict == ConflictStrategy::Overwrite
                    {
                        // create_dir_all is idempotent when the directory already exists
                    } else if would_skip_conflict(target, conflict) {
                        skipped += 1;
                    } else {
                        collisions.push(BlueprintCollision {
                            source: item
                                .source
                                .as_ref()
                                .map(|path| path.display().to_string())
                                .unwrap_or_default(),
                            target: target.display().to_string(),
                        });
                    }
                }
            }
        }

        if matches!(item.action, FolderAction::Delete) {
            if let Some(source) = &item.source {
                if source.exists() && directory_has_entries(source) {
                    warnings.push(format!(
                        "delete action targets a non-empty folder: {}",
                        source.display()
                    ));
                }
            }
        }

        if sample.len() < 25 {
            sample.push(BlueprintPlanSample {
                source: item
                    .source
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default(),
                target: item
                    .target
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default(),
                action: folder_action_label(item.action).to_string(),
            });
        }
    }

    BlueprintSimulationReport {
        matched: plan.items.len() as u32,
        skipped,
        plan_sample: sample,
        warnings,
        collisions,
    }
}

fn build_folder_action_plan(
    blueprint: &Blueprint,
    variables: &BTreeMap<String, String>,
    dry_run: bool,
) -> Result<FolderActionPlan, DomainError> {
    let root = PathBuf::from(&blueprint.root_path);
    let folder_plan = blueprint.folder_plan.as_ref().ok_or_else(|| {
        DomainError::Validation("folder_plan is required for folder blueprints".into())
    })?;

    let missing = missing_required_folder_variables(folder_plan, variables);
    if !dry_run && !missing.is_empty() {
        return Err(DomainError::Validation(format!(
            "required folder variables missing values: {}",
            missing.join(", ")
        )));
    }

    build_folder_action_plan_for_nodes(
        blueprint,
        folder_plan,
        &folder_plan.nodes,
        &root,
        variables,
        dry_run,
        true,
    )
}

pub(crate) fn build_folder_action_plan_for_nodes(
    blueprint: &Blueprint,
    folder_plan: &FolderPlan,
    nodes: &[FolderActionNode],
    base_root: &Path,
    variables: &BTreeMap<String, String>,
    dry_run: bool,
    enforce_required: bool,
) -> Result<FolderActionPlan, DomainError> {
    let root = PathBuf::from(&blueprint.root_path);
    let canonical_root = normalize_path(&root);
    let base = normalize_path(base_root);
    if !is_path_under_root(&base, &canonical_root) {
        return Err(DomainError::Validation(format!(
            "folder action base escapes blueprint root: {}",
            base_root.display()
        )));
    }

    if enforce_required {
        let missing = missing_required_folder_variables(folder_plan, variables);
        if !dry_run && !missing.is_empty() {
            return Err(DomainError::Validation(format!(
                "required folder variables missing values: {}",
                missing.join(", ")
            )));
        }
    }

    let mut warnings = Vec::new();
    let variable_values =
        folder_plan_variable_values(folder_plan, variables, dry_run, &mut warnings);
    let mut items = Vec::new();
    collect_folder_action_nodes(
        nodes,
        "",
        &base,
        &canonical_root,
        &variable_values,
        &mut warnings,
        &mut items,
    )?;

    let plan = FolderActionPlan { items, warnings };
    preflight_folder_action_plan(&plan, folder_plan, &root, dry_run)?;
    Ok(plan)
}

fn collect_folder_action_nodes(
    nodes: &[FolderActionNode],
    parent_rel: &str,
    base: &Path,
    canonical_root: &Path,
    variables: &BTreeMap<String, String>,
    warnings: &mut Vec<String>,
    items: &mut Vec<FolderActionPlanItem>,
) -> Result<(), DomainError> {
    for (index, node) in nodes.iter().enumerate() {
        let ctx = folder_token_context(base, index as u64 + 1, variables);
        let name = evaluate_folder_template(&node.name, &ctx, warnings, "name", true)?;
        let source_rel =
            evaluate_folder_template(&node.source_path, &ctx, warnings, "source", true)?;
        let target_rel =
            evaluate_folder_template(&node.target_path, &ctx, warnings, "target", true)?;
        let resolved =
            resolve_folder_action_paths(node, parent_rel, &name, &source_rel, &target_rel)?;

        let source = match resolved.source_rel.as_deref() {
            Some(rel) if !rel.trim().is_empty() => Some(safe_join(base, canonical_root, rel)?),
            _ => None,
        };
        let target = match resolved.target_rel.as_deref() {
            Some(rel) if !rel.trim().is_empty() => Some(safe_join(base, canonical_root, rel)?),
            _ => None,
        };

        items.push(FolderActionPlanItem {
            action: node.action,
            source,
            target,
        });

        if matches!(node.action, FolderAction::Delete) {
            if !node.children.is_empty() {
                warnings.push(format!(
                    "children ignored after delete action: {}",
                    resolved
                        .source_rel
                        .as_deref()
                        .or(resolved.target_rel.as_deref())
                        .unwrap_or("")
                ));
            }
            continue;
        }

        let child_parent = resolved
            .target_rel
            .as_deref()
            .unwrap_or(parent_rel)
            .trim_matches('/')
            .to_string();

        collect_folder_action_nodes(
            &node.children,
            &child_parent,
            base,
            canonical_root,
            variables,
            warnings,
            items,
        )?;
    }

    Ok(())
}

struct ResolvedFolderActionPaths {
    source_rel: Option<String>,
    target_rel: Option<String>,
}

fn join_relative(parent: &str, child: &str) -> String {
    normalize_relative_display(&format!("{parent}/{child}"))
}

fn resolve_folder_action_paths(
    node: &FolderActionNode,
    parent_rel: &str,
    name: &str,
    source_path: &str,
    target_path: &str,
) -> Result<ResolvedFolderActionPaths, DomainError> {
    let source = normalize_relative_display(source_path);
    let target = normalize_relative_display(target_path);
    let name = normalize_relative_display(name);

    let paths = match node.action {
        FolderAction::Create => {
            let target_rel = if !target.is_empty() {
                join_relative(parent_rel, &target)
            } else {
                join_relative(parent_rel, &name)
            };
            ResolvedFolderActionPaths {
                source_rel: None,
                target_rel: Some(target_rel),
            }
        }
        FolderAction::Rename => {
            if name.is_empty() {
                return Err(DomainError::Validation(
                    "rename folder action requires a new name".into(),
                ));
            }
            let source_rel =
                join_relative(parent_rel, if source.is_empty() { &name } else { &source });
            let target_rel = if !target.is_empty() {
                join_relative(parent_rel, &target)
            } else {
                join_relative(parent_rel, &name)
            };
            ResolvedFolderActionPaths {
                source_rel: Some(source_rel),
                target_rel: Some(target_rel),
            }
        }
        FolderAction::Move | FolderAction::Copy => {
            if source.is_empty() {
                return Err(DomainError::Validation(
                    "move/copy folder action requires source path".into(),
                ));
            }
            let target_rel = if !target.is_empty() {
                join_relative(parent_rel, &target)
            } else if !name.is_empty() {
                join_relative(parent_rel, &name)
            } else {
                return Err(DomainError::Validation(
                    "move/copy folder action requires target path or name".into(),
                ));
            };
            ResolvedFolderActionPaths {
                source_rel: Some(join_relative(parent_rel, &source)),
                target_rel: Some(target_rel),
            }
        }
        FolderAction::Delete => {
            if source.is_empty() && name.is_empty() {
                return Err(DomainError::Validation(
                    "delete folder action requires source path".into(),
                ));
            }
            ResolvedFolderActionPaths {
                source_rel: Some(join_relative(
                    parent_rel,
                    if source.is_empty() { &name } else { &source },
                )),
                target_rel: None,
            }
        }
    };

    for rel in [paths.source_rel.as_deref(), paths.target_rel.as_deref()]
        .into_iter()
        .flatten()
    {
        if relative_path_invalid(rel) {
            return Err(DomainError::Validation(format!(
                "invalid relative folder path: {rel}"
            )));
        }
    }

    Ok(paths)
}

fn preflight_folder_action_plan(
    plan: &FolderActionPlan,
    folder_plan: &FolderPlan,
    root: &Path,
    dry_run: bool,
) -> Result<(), DomainError> {
    for item in &plan.items {
        match item.action {
            FolderAction::Create => {
                if item.target.is_none() {
                    return Err(DomainError::Validation(
                        "create folder action requires a target".into(),
                    ));
                }
            }
            FolderAction::Rename | FolderAction::Move | FolderAction::Copy => {
                let Some(source) = &item.source else {
                    return Err(DomainError::Validation(
                        "folder action requires a source".into(),
                    ));
                };
                if !dry_run && !source.is_dir() {
                    if item.target.as_ref().is_some_and(|target| target.is_dir()) {
                        continue;
                    }
                    return Err(DomainError::SourceNotFound(source.display().to_string()));
                }
                if item.target.is_none() {
                    return Err(DomainError::Validation(
                        "folder action requires a target".into(),
                    ));
                }
            }
            FolderAction::Delete => {
                let Some(source) = &item.source else {
                    return Err(DomainError::Validation(
                        "delete folder action requires a source".into(),
                    ));
                };
                if normalize_path(source) == normalize_path(root) {
                    return Err(DomainError::Validation(
                        "delete folder action cannot target the root folder".into(),
                    ));
                }
                if !dry_run && !source.is_dir() {
                    return Err(DomainError::SourceNotFound(source.display().to_string()));
                }
                if source.exists()
                    && directory_has_entries(source)
                    && !(folder_plan.allow_delete_non_empty && folder_plan.confirm_delete_non_empty)
                {
                    return Err(DomainError::Validation(format!(
                        "delete non-empty folder requires both safety confirmations: {}",
                        source.display()
                    )));
                }
            }
        }
    }

    Ok(())
}

async fn apply_folder_blueprint(
    audit: &dyn AuditStore,
    blueprint: &Blueprint,
    variables: &BTreeMap<String, String>,
) -> Result<BlueprintApplyResult, DomainError> {
    let plan = build_folder_action_plan(blueprint, variables, false)?;
    let mut processed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;

    for item in plan.items {
        let started = Instant::now();
        let result = execute_folder_action_item(blueprint, &item);
        let duration_ms = started.elapsed().as_secs_f64() * 1000.0;
        let source_path = item
            .source
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| blueprint.root_path.clone());
        let target_path = item
            .target
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_default();

        match result {
            Ok(Some(target)) => {
                processed += 1;
                audit
                    .append(NewAuditEntry {
                        job_id: None,
                        blueprint_id: Some(blueprint.id),
                        job_name: blueprint.name.clone(),
                        source_path,
                        target_path: target.display().to_string(),
                        status: AuditStatus::Organized,
                        file_size: 0,
                        duration_ms,
                        details: Some(folder_action_label(item.action).to_string()),
                    })
                    .await?;
            }
            Ok(None) => {
                skipped += 1;
                audit
                    .append(NewAuditEntry {
                        job_id: None,
                        blueprint_id: Some(blueprint.id),
                        job_name: blueprint.name.clone(),
                        source_path,
                        target_path,
                        status: AuditStatus::Ignored,
                        file_size: 0,
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
                        source_path,
                        target_path,
                        status: AuditStatus::Failed,
                        file_size: 0,
                        duration_ms,
                        details: Some(details),
                    })
                    .await?;
            }
        }
    }

    Ok(BlueprintApplyResult {
        processed,
        failed,
        skipped,
    })
}

pub(crate) fn execute_folder_action_item(
    blueprint: &Blueprint,
    item: &FolderActionPlanItem,
) -> Result<Option<PathBuf>, String> {
    match item.action {
        FolderAction::Create => {
            let target = item
                .target
                .as_ref()
                .ok_or_else(|| "create target is required".to_string())?;
            if target.is_dir() {
                return match blueprint.conflict {
                    ConflictStrategy::Skip => Ok(None),
                    ConflictStrategy::Overwrite | ConflictStrategy::Rename => {
                        Ok(Some(target.clone()))
                    }
                };
            }
            let Some(target) = resolve_conflict(target, blueprint.conflict)? else {
                return Ok(None);
            };
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            Ok(Some(target))
        }
        FolderAction::Rename | FolderAction::Move => {
            let source = item
                .source
                .as_ref()
                .ok_or_else(|| "source is required".to_string())?;
            let target = item
                .target
                .as_ref()
                .ok_or_else(|| "target is required".to_string())?;
            if !source.exists() {
                if target.is_dir() {
                    return Ok(Some(target.clone()));
                }
                return Err(format!("source not found: {}", source.display()));
            }
            let Some(target) = resolve_conflict(target, blueprint.conflict)? else {
                return Ok(None);
            };
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            if fs::rename(source, &target).is_err() {
                copy_dir_recursive(source, &target)?;
                fs::remove_dir_all(source).map_err(|e| e.to_string())?;
            }
            Ok(Some(target))
        }
        FolderAction::Copy => {
            let source = item
                .source
                .as_ref()
                .ok_or_else(|| "source is required".to_string())?;
            let target = item
                .target
                .as_ref()
                .ok_or_else(|| "target is required".to_string())?;
            if !source.exists() {
                if target.is_dir() {
                    return Ok(Some(target.clone()));
                }
                return Err(format!("source not found: {}", source.display()));
            }
            let Some(target) = resolve_conflict(target, blueprint.conflict)? else {
                return Ok(None);
            };
            copy_dir_recursive(source, &target)?;
            Ok(Some(target))
        }
        FolderAction::Delete => {
            let source = item
                .source
                .as_ref()
                .ok_or_else(|| "delete source is required".to_string())?;
            if directory_has_entries(source) {
                fs::remove_dir_all(source).map_err(|e| e.to_string())?;
            } else {
                fs::remove_dir(source).map_err(|e| e.to_string())?;
            }
            Ok(Some(source.clone()))
        }
    }
}

fn safe_join(base: &Path, canonical_root: &Path, relative: &str) -> Result<PathBuf, DomainError> {
    if relative_path_invalid(relative) {
        return Err(DomainError::Validation(format!(
            "invalid relative folder path: {relative}"
        )));
    }
    let target = base.join(relative);
    if !is_path_under_root(&target, canonical_root) {
        return Err(DomainError::Validation(format!(
            "folder action escapes root: {}",
            target.display()
        )));
    }
    Ok(target)
}

fn relative_path_invalid(relative: &str) -> bool {
    validate_relative_path(relative).is_err()
}

fn normalize_relative_display(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .trim_matches('/')
        .split('/')
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("/")
}

fn evaluate_folder_template(
    input: &str,
    ctx: &TokenContext,
    warnings: &mut Vec<String>,
    label: &str,
    strict: bool,
) -> Result<String, DomainError> {
    if input.trim().is_empty() {
        return Ok(String::new());
    }
    let pipeline = parse_template_display_string(input);
    match evaluate(&pipeline, ctx) {
        Ok(value) => Ok(value),
        Err(error) if strict => Err(DomainError::Validation(format!(
            "invalid folder {label} template: {error}"
        ))),
        Err(error) => {
            warnings.push(format!("invalid folder {label} template: {error}"));
            Ok(input.to_string())
        }
    }
}

pub(crate) fn parse_template_display_string(input: &str) -> TemplatePipeline {
    parse_template(input)
}

pub(crate) fn folder_token_context(
    root: &Path,
    index: u64,
    variables: &BTreeMap<String, String>,
) -> TokenContext {
    TokenContext::for_directory(root, index, index).with_variables(variables.clone())
}

pub(crate) fn folder_plan_variable_values(
    folder_plan: &FolderPlan,
    overrides: &BTreeMap<String, String>,
    preview: bool,
    warnings: &mut Vec<String>,
) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for variable in &folder_plan.variables {
        let key = folder_variable_key(&variable.name);
        let value = overrides
            .get(&variable.name)
            .or_else(|| overrides.get(&key))
            .cloned()
            .unwrap_or_else(|| variable.default_value.clone());

        if variable.required && value.trim().is_empty() {
            warnings.push(format!(
                "required folder variable has no value: {}",
                variable.name
            ));
            if preview {
                values.insert(key, format!("{{{}}}", variable.name));
                continue;
            }
        }
        values.insert(key, value);
    }
    values
}

fn missing_required_folder_variables(
    folder_plan: &FolderPlan,
    overrides: &BTreeMap<String, String>,
) -> Vec<String> {
    folder_plan
        .variables
        .iter()
        .filter_map(|variable| {
            let key = folder_variable_key(&variable.name);
            let value = overrides
                .get(&variable.name)
                .or_else(|| overrides.get(&key))
                .map(String::as_str)
                .unwrap_or(variable.default_value.as_str());
            if variable.required && value.trim().is_empty() {
                Some(variable.name.clone())
            } else {
                None
            }
        })
        .collect()
}

fn folder_variable_key(name: &str) -> String {
    name.chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn folder_plan_default_variables(folder_plan: Option<&FolderPlan>) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    let Some(folder_plan) = folder_plan else {
        return values;
    };
    for variable in &folder_plan.variables {
        values.insert(variable.name.clone(), variable.default_value.clone());
    }
    values
}

fn would_skip_conflict(target: &Path, conflict: ConflictStrategy) -> bool {
    target.exists() && conflict == ConflictStrategy::Skip
}

fn folder_action_label(action: FolderAction) -> &'static str {
    match action {
        FolderAction::Create => "create",
        FolderAction::Rename => "rename",
        FolderAction::Move => "move",
        FolderAction::Copy => "copy",
        FolderAction::Delete => "delete",
    }
}

fn directory_has_entries(path: &Path) -> bool {
    fs::read_dir(path)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}

fn display_action_path(node: &FolderActionNode, fallback: &str) -> String {
    if !node.source_path.trim().is_empty() {
        node.source_path.clone()
    } else if !node.target_path.trim().is_empty() {
        node.target_path.clone()
    } else if !node.name.trim().is_empty() {
        node.name.clone()
    } else {
        fallback.to_string()
    }
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
    if blueprint.kind != BlueprintKind::File {
        return Err(DomainError::Validation(
            "build_plan only supports file blueprints".into(),
        ));
    }

    let root = PathBuf::from(&blueprint.root_path);
    let canonical_root = normalize_path(&root);
    let candidates = collect_files_for_blueprint(blueprint)?;

    let mut plan = Vec::new();
    let mut counters = HashMap::new();
    for (index, source) in candidates.into_iter().enumerate() {
        // Follow symlinks only when their target stays inside the blueprint root.
        if !is_path_under_root(&normalize_path(&source), &canonical_root) {
            continue;
        }
        let relative_dest = resolve_destination(
            store,
            blueprint,
            &source,
            index as u64,
            dry_run,
            &mut counters,
        )
        .await?;
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
    counters: &mut HashMap<String, u64>,
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
    let date_key = Local::now().format("%Y-%m-%d").to_string();
    let scope_key = blueprint.counter.scope_key(parent, &folder, &date_key);
    let counter = if dry_run {
        let next = match counters.get(&scope_key).copied() {
            Some(value) => value,
            None => {
                let stored = store.get_counter(blueprint.id, &scope_key).await?;
                if stored == 0 {
                    blueprint.counter.start
                } else {
                    stored
                }
            }
        };
        counters.insert(scope_key.clone(), next.saturating_add(1));
        next
    } else {
        store
            .next_counter(blueprint.id, &scope_key, blueprint.counter.start)
            .await?
    };

    let counter_display = blueprint.counter.format_value(counter);
    let mut ctx = TokenContext::from_path_with_root(source, root, index + 1, counter);
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
    validate_relative_path(&relative).map_err(DomainError::Validation)?;

    Ok(relative)
}

fn collect_files_for_blueprint(blueprint: &Blueprint) -> Result<Vec<PathBuf>, DomainError> {
    let job = blueprint_search_job(blueprint);
    autoflow_domain::collect_files(&job).map_err(|e| DomainError::Database(e.to_string()))
}

fn blueprint_search_job(blueprint: &Blueprint) -> Job {
    let mut job = Job::new(&blueprint.name, &blueprint.root_path, &blueprint.root_path);
    job.filters = blueprint.search.clone();
    job.filters.recursive = blueprint.recursive;
    job
}

fn execute_plan_item(blueprint: &Blueprint, item: &PlanItem) -> Result<Option<PathBuf>, String> {
    let same_path = item.source == item.target;
    let resolved = if same_path {
        Some(item.target.clone())
    } else {
        resolve_conflict(&item.target, blueprint.conflict)?
    };
    let Some(target) = resolved else {
        return Ok(None);
    };

    if blueprint.routing.create_intermediate_dirs {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    } else if let Some(parent) = target.parent() {
        if !parent.exists() {
            return Err(format!(
                "target parent does not exist and intermediate directories are disabled: {}",
                parent.display()
            ));
        }
    }

    if !same_path {
        match blueprint.operation {
            BlueprintOperation::Copy => {
                if item.source.is_dir() {
                    copy_dir_recursive(&item.source, &target)?;
                } else {
                    fs::copy(&item.source, &target).map_err(|e| e.to_string())?;
                }
            }
            BlueprintOperation::Move => {
                if fs::rename(&item.source, &target).is_err() {
                    if item.source.is_dir() {
                        copy_dir_recursive(&item.source, &target)?;
                        fs::remove_dir_all(&item.source).map_err(|e| e.to_string())?;
                    } else {
                        fs::copy(&item.source, &target).map_err(|e| e.to_string())?;
                        fs::remove_file(&item.source).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }

    Ok(Some(target))
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
        let metadata = fs::symlink_metadata(&src).map_err(|e| e.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "refusing to copy symlink inside blueprint root: {}",
                src.display()
            ));
        }
        if metadata.is_dir() {
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
    use autoflow_domain::{FolderActionNode, FolderPlanVariable};

    fn folder_blueprint(root: &Path, folder_plan: FolderPlan) -> Blueprint {
        let mut blueprint = Blueprint::new(
            "Estrutura de pastas",
            BlueprintKind::Folder,
            root.to_string_lossy().to_string(),
        );
        blueprint.conflict = ConflictStrategy::Overwrite;
        blueprint.folder_plan = Some(folder_plan);
        blueprint
    }

    fn folder_action(
        action: FolderAction,
        source_path: &str,
        target_path: &str,
        name: &str,
    ) -> FolderActionNode {
        FolderActionNode {
            action,
            name: name.to_string(),
            source_path: source_path.to_string(),
            target_path: target_path.to_string(),
            children: Vec::new(),
        }
    }

    fn execute_folder_plan(
        blueprint: &Blueprint,
        variables: &BTreeMap<String, String>,
    ) -> Result<(), DomainError> {
        let plan = build_folder_action_plan(blueprint, variables, false)?;
        for item in plan.items {
            execute_folder_action_item(blueprint, &item).map_err(DomainError::Database)?;
        }
        Ok(())
    }

    #[test]
    fn preview_folder_plan_builds_relative_tree() {
        let mut year = FolderActionNode::create("2026");
        year.children.push(FolderActionNode::create("Janeiro"));

        let preview = preview_folder_plan(
            "C:\\root".into(),
            FolderPlan {
                nodes: vec![year],
                ..FolderPlan::default()
            },
            BTreeMap::new(),
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
                nodes: vec![FolderActionNode::create("bad<name")],
                ..FolderPlan::default()
            },
            BTreeMap::new(),
        );

        assert!(!preview.valid);
        assert!(!preview.warnings.is_empty());
    }

    #[test]
    fn create_folder_action_builds_tree_inside_target_folder() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut entry = FolderActionNode::create("01 Entrada");
        entry.children.push(FolderActionNode::create("Briefing"));

        let blueprint = folder_blueprint(
            temp.path(),
            FolderPlan {
                nodes: vec![entry],
                ..FolderPlan::default()
            },
        );

        execute_folder_plan(&blueprint, &BTreeMap::new()).expect("folder action plan");

        assert!(temp.path().join("01 Entrada").is_dir());
        assert!(temp.path().join("01 Entrada").join("Briefing").is_dir());
    }

    #[test]
    fn folder_action_plan_creates_tree_with_variables() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut project = FolderActionNode::create("{projectName}");
        project
            .children
            .push(FolderActionNode::create("01 Briefing"));
        project
            .children
            .push(FolderActionNode::create("02 Execucao"));
        project
            .children
            .push(FolderActionNode::create("03 Entrega"));

        let blueprint = folder_blueprint(
            temp.path(),
            FolderPlan {
                variables: vec![FolderPlanVariable {
                    name: "projectName".into(),
                    label: "Nome do projeto".into(),
                    default_value: String::new(),
                    required: true,
                }],
                nodes: vec![project],
                ..FolderPlan::default()
            },
        );
        let variables = BTreeMap::from([("projectName".into(), "Projeto Alpha".into())]);

        execute_folder_plan(&blueprint, &variables).expect("execute folder plan");

        assert!(temp.path().join("Projeto Alpha").is_dir());
        assert!(temp
            .path()
            .join("Projeto Alpha")
            .join("01 Briefing")
            .is_dir());
        assert!(temp
            .path()
            .join("Projeto Alpha")
            .join("02 Execucao")
            .is_dir());
        assert!(temp
            .path()
            .join("Projeto Alpha")
            .join("03 Entrega")
            .is_dir());
    }

    #[test]
    fn folder_action_plan_renames_moves_copies_and_deletes_subfolders() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(temp.path().join("old")).expect("old");
        fs::create_dir_all(temp.path().join("to-move")).expect("to move");
        fs::create_dir_all(temp.path().join("to-copy")).expect("to copy");
        fs::write(temp.path().join("to-copy").join("file.txt"), "copy me").expect("copy file");
        fs::create_dir_all(temp.path().join("to-delete")).expect("to delete");

        let blueprint = folder_blueprint(
            temp.path(),
            FolderPlan {
                nodes: vec![
                    folder_action(FolderAction::Rename, "old", "", "renamed"),
                    folder_action(FolderAction::Move, "to-move", "moved/to-move", ""),
                    folder_action(FolderAction::Copy, "to-copy", "copied/to-copy", ""),
                    folder_action(FolderAction::Delete, "to-delete", "", ""),
                ],
                ..FolderPlan::default()
            },
        );

        execute_folder_plan(&blueprint, &BTreeMap::new()).expect("execute folder plan");

        assert!(!temp.path().join("old").exists());
        assert!(temp.path().join("renamed").is_dir());
        assert!(!temp.path().join("to-move").exists());
        assert!(temp.path().join("moved").join("to-move").is_dir());
        assert!(temp.path().join("to-copy").is_dir());
        assert!(temp
            .path()
            .join("copied")
            .join("to-copy")
            .join("file.txt")
            .is_file());
        assert!(!temp.path().join("to-delete").exists());
    }

    #[test]
    fn folder_action_plan_blocks_paths_outside_root() {
        let temp = tempfile::tempdir().expect("tempdir");
        let blueprint = folder_blueprint(
            temp.path(),
            FolderPlan {
                nodes: vec![FolderActionNode::create("../fora")],
                ..FolderPlan::default()
            },
        );

        let error = build_folder_action_plan(&blueprint, &BTreeMap::new(), false)
            .expect_err("path outside root should fail");

        assert!(matches!(error, DomainError::Validation(_)));
    }

    #[test]
    fn folder_action_plan_blocks_non_empty_delete_without_double_confirmation() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(temp.path().join("doomed")).expect("doomed");
        fs::write(temp.path().join("doomed").join("file.txt"), "important").expect("file");

        let blueprint = folder_blueprint(
            temp.path(),
            FolderPlan {
                nodes: vec![folder_action(FolderAction::Delete, "doomed", "", "")],
                ..FolderPlan::default()
            },
        );

        let error = build_folder_action_plan(&blueprint, &BTreeMap::new(), false)
            .expect_err("non-empty delete should fail");

        assert!(matches!(error, DomainError::Validation(_)));
        assert!(temp.path().join("doomed").join("file.txt").is_file());
    }

    #[test]
    fn folder_action_plan_allows_non_empty_delete_with_double_confirmation() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(temp.path().join("doomed")).expect("doomed");
        fs::write(temp.path().join("doomed").join("file.txt"), "important").expect("file");

        let blueprint = folder_blueprint(
            temp.path(),
            FolderPlan {
                allow_delete_non_empty: true,
                confirm_delete_non_empty: true,
                nodes: vec![folder_action(FolderAction::Delete, "doomed", "", "")],
                ..FolderPlan::default()
            },
        );

        execute_folder_plan(&blueprint, &BTreeMap::new()).expect("execute folder plan");

        assert!(!temp.path().join("doomed").exists());
    }

    #[test]
    fn folder_plan_deserializes_legacy_nodes_as_create_actions() {
        let legacy =
            r#"{"nodes":[{"name":"Cliente","children":[{"name":"Briefing","children":[]}]}]}"#;

        let plan: FolderPlan = serde_json::from_str(legacy).expect("legacy plan");

        assert_eq!(plan.version, 1);
        assert_eq!(plan.nodes[0].action, FolderAction::Create);
        assert_eq!(plan.nodes[0].name, "Cliente");
        assert_eq!(plan.nodes[0].children[0].action, FolderAction::Create);
        assert_eq!(plan.nodes[0].children[0].name, "Briefing");
    }

    #[test]
    fn nested_folder_actions_are_relative_to_parent() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(temp.path().join("Client").join("old")).expect("nested source");

        let mut client = FolderActionNode::create("Client");
        client
            .children
            .push(folder_action(FolderAction::Rename, "old", "", "new"));

        let blueprint = folder_blueprint(
            temp.path(),
            FolderPlan {
                nodes: vec![client],
                ..FolderPlan::default()
            },
        );

        execute_folder_plan(&blueprint, &BTreeMap::new()).expect("execute nested plan");

        assert!(!temp.path().join("Client").join("old").exists());
        assert!(temp.path().join("Client").join("new").is_dir());
    }

    #[test]
    fn create_intermediate_dirs_flag_is_respected() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("source.txt");
        fs::write(&source, b"data").expect("source");
        let target = temp
            .path()
            .join("missing")
            .join("nested")
            .join("target.txt");

        let mut blueprint = Blueprint::new(
            "Arquivos",
            BlueprintKind::File,
            temp.path().to_string_lossy().to_string(),
        );
        blueprint.operation = BlueprintOperation::Copy;
        blueprint.routing.create_intermediate_dirs = false;

        let item = PlanItem {
            source: source.clone(),
            target: target.clone(),
        };
        assert!(execute_plan_item(&blueprint, &item).is_err());
        assert!(!target.parent().expect("parent").exists());

        blueprint.routing.create_intermediate_dirs = true;
        execute_plan_item(&blueprint, &item).expect("copy with intermediate dirs");
        assert!(target.is_file());
    }

    #[test]
    fn folder_action_simulation_counts_skipped_create_conflicts() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(temp.path().join("existing")).expect("existing");

        let mut blueprint = folder_blueprint(
            temp.path(),
            FolderPlan {
                nodes: vec![FolderActionNode::create("existing")],
                ..FolderPlan::default()
            },
        );
        blueprint.conflict = ConflictStrategy::Skip;

        let plan = build_folder_action_plan(&blueprint, &BTreeMap::new(), true).expect("plan");
        let report = folder_action_simulation_report(&plan, ConflictStrategy::Skip);
        assert_eq!(report.skipped, 1);
        assert!(report.collisions.is_empty());
    }
}
