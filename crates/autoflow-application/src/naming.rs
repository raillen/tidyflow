use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use autoflow_domain::{
    evaluate, Blueprint, BlueprintApplyResult, BlueprintKind, DomainError, FolderAction,
    NamingPattern, NewAuditEntry, AuditStatus,
};

use crate::ports::AuditStore;
use crate::blueprints::{
    execute_folder_action_item, folder_token_context, folder_plan_variable_values,
    parse_template_display_string, FolderActionPlanItem,
};

fn normalize_variables(
    folder_plan: &autoflow_domain::FolderPlan,
    variables: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut warnings = Vec::new();
    folder_plan_variable_values(folder_plan, variables, false, &mut warnings)
}

pub fn folder_name_matches_pattern(
    folder_name: &str,
    naming_pattern: Option<&NamingPattern>,
    folder_plan: &autoflow_domain::FolderPlan,
    variables: &BTreeMap<String, String>,
) -> Result<bool, String> {
    let root = Path::new(".");
    let normalized = normalize_variables(folder_plan, variables);
    let ctx = folder_token_context(root, 1, &normalized);

    if let Some(np) = naming_pattern {
        if let Some(ref tpl) = np.template_override {
            let resolved = evaluate(tpl, &ctx).map_err(|e| e.to_string())?;
            return Ok(np.matches(folder_name, Some(&resolved)));
        }
        if np.regex.is_some() {
            return Ok(np.matches(folder_name, None));
        }
    }

    if let Some(root_node) = folder_plan.nodes.first() {
        if !root_node.name.trim().is_empty() {
            let pipeline = parse_template_display_string(&root_node.name);
            let resolved = evaluate(&pipeline, &ctx).map_err(|e| e.to_string())?;
            return Ok(folder_name == resolved);
        }
    }

    Ok(true)
}

pub fn resolve_correct_folder_name(
    folder_name: &str,
    naming_pattern: Option<&NamingPattern>,
    folder_plan: &autoflow_domain::FolderPlan,
    variables: &BTreeMap<String, String>,
) -> Result<String, String> {
    let root = Path::new(".");
    let normalized = normalize_variables(folder_plan, variables);
    let ctx = folder_token_context(root, 1, &normalized);

    if let Some(np) = naming_pattern {
        if let Some(ref tpl) = np.template_override {
            let resolved = evaluate(tpl, &ctx).map_err(|e| e.to_string())?;
            if !resolved.trim().is_empty() {
                return Ok(resolved);
            }
        }
    }

    if let Some(root_node) = folder_plan.nodes.first() {
        if !root_node.name.trim().is_empty() {
            let pipeline = parse_template_display_string(&root_node.name);
            let resolved = evaluate(&pipeline, &ctx).map_err(|e| e.to_string())?;
            if !resolved.trim().is_empty() {
                return Ok(resolved);
            }
        }
    }

    Ok(folder_name.to_string())
}

pub async fn apply_children_structure(
    audit: &dyn AuditStore,
    blueprint: &Blueprint,
    renamed_folder: &Path,
    variables: &BTreeMap<String, String>,
) -> Result<BlueprintApplyResult, DomainError> {
    let folder_plan = blueprint
        .folder_plan
        .as_ref()
        .ok_or_else(|| DomainError::Validation("folder_plan required for auto-correct".into()))?;

    let root_node = folder_plan
        .nodes
        .first()
        .ok_or_else(|| DomainError::Validation("folder_plan requires at least one node".into()))?;

    let mut processed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;

    for (index, child_node) in root_node.children.iter().enumerate() {
        let child_items =
            collect_child_items(child_node, renamed_folder, index as u64, variables)?;

        for item in child_items {
            if let Some(ref target) = item.target {
                if target.is_dir() {
                    skipped += 1;
                    continue;
                }
            }

            let started = Instant::now();
            let result = execute_folder_action_item(blueprint, &item);
            let duration_ms = started.elapsed().as_secs_f64() * 1000.0;
            let source_str = item
                .source
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            let target_str = item
                .target
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();

            match result {
                Ok(Some(target)) => {
                    processed += 1;
                    audit
                        .append(NewAuditEntry {
                            job_id: None,
                            blueprint_id: Some(blueprint.id),
                            job_name: format!("{} (auto-correct)", blueprint.name),
                            source_path: source_str,
                            target_path: target.display().to_string(),
                            status: AuditStatus::Organized,
                            file_size: 0,
                            duration_ms,
                            details: Some("auto-correct structure".into()),
                        })
                        .await?;
                }
                Ok(None) => {
                    skipped += 1;
                }
                Err(details) => {
                    failed += 1;
                    audit
                        .append(NewAuditEntry {
                            job_id: None,
                            blueprint_id: Some(blueprint.id),
                            job_name: format!("{} (auto-correct)", blueprint.name),
                            source_path: source_str,
                            target_path: target_str,
                            status: AuditStatus::Failed,
                            file_size: 0,
                            duration_ms,
                            details: Some(details),
                        })
                        .await?;
                }
            }
        }
    }

    Ok(BlueprintApplyResult {
        processed,
        failed,
        skipped,
    })
}

fn collect_child_items(
    node: &autoflow_domain::FolderActionNode,
    parent: &Path,
    index: u64,
    variables: &BTreeMap<String, String>,
) -> Result<Vec<FolderActionPlanItem>, DomainError> {
    let mut items = Vec::new();
    let ctx = folder_token_context(parent, index + 1, variables);
    let pipeline = parse_template_display_string(&node.name);
    let name = evaluate(&pipeline, &ctx).map_err(|e| DomainError::Validation(e.to_string()))?;

    if name.trim().is_empty() {
        return Ok(items);
    }

    let child_path = parent.join(&name);

    match node.action {
        autoflow_domain::FolderAction::Create => {
            items.push(FolderActionPlanItem {
                action: FolderAction::Create,
                source: None,
                target: Some(child_path.clone()),
            });
            for (i, grandchild) in node.children.iter().enumerate() {
                let sub_items = collect_child_items(grandchild, &child_path, i as u64, variables)?;
                items.extend(sub_items);
            }
        }
        autoflow_domain::FolderAction::Rename => {
            let source_rel = if !node.source_path.trim().is_empty() {
                node.source_path.clone()
            } else {
                name.clone()
            };
            let source = parent.join(&source_rel);
            items.push(FolderActionPlanItem {
                action: FolderAction::Rename,
                source: Some(source),
                target: Some(child_path.clone()),
            });
            for (i, grandchild) in node.children.iter().enumerate() {
                let sub_items = collect_child_items(grandchild, &child_path, i as u64, variables)?;
                items.extend(sub_items);
            }
        }
        autoflow_domain::FolderAction::Move => {
            let source = if !node.source_path.trim().is_empty() {
                parent.join(&node.source_path)
            } else {
                return Err(DomainError::Validation(
                    "move child action requires source_path".into(),
                ));
            };
            items.push(FolderActionPlanItem {
                action: FolderAction::Move,
                source: Some(source),
                target: Some(child_path.clone()),
            });
            for (i, grandchild) in node.children.iter().enumerate() {
                let sub_items = collect_child_items(grandchild, &child_path, i as u64, variables)?;
                items.extend(sub_items);
            }
        }
        autoflow_domain::FolderAction::Copy => {
            let source = if !node.source_path.trim().is_empty() {
                parent.join(&node.source_path)
            } else {
                return Err(DomainError::Validation(
                    "copy child action requires source_path".into(),
                ));
            };
            items.push(FolderActionPlanItem {
                action: FolderAction::Copy,
                source: Some(source),
                target: Some(child_path.clone()),
            });
            for (i, grandchild) in node.children.iter().enumerate() {
                let sub_items = collect_child_items(grandchild, &child_path, i as u64, variables)?;
                items.extend(sub_items);
            }
        }
        autoflow_domain::FolderAction::Delete => {
            let source = if !node.source_path.trim().is_empty() {
                parent.join(&node.source_path)
            } else {
                child_path.clone()
            };
            items.push(FolderActionPlanItem {
                action: FolderAction::Delete,
                source: Some(source),
                target: None,
            });
        }
    }

    Ok(items)
}

pub async fn handle_folder_auto_correct(
    store: &dyn crate::BlueprintStore,
    audit: &dyn AuditStore,
    blueprint_id: uuid::Uuid,
    folder_path: &str,
    folder_name: &str,
) -> Result<(), DomainError> {
    tracing::info!(%blueprint_id, %folder_path, %folder_name, "auto-correct handler invoked");

    let blueprint = store.get(blueprint_id).await?;
    if !blueprint.enabled {
        tracing::debug!("blueprint disabled, skipping auto-correct");
        return Ok(());
    }
    if blueprint.kind != BlueprintKind::Folder {
        tracing::debug!("blueprint is not folder kind, skipping auto-correct");
        return Ok(());
    }

    let folder_plan = match blueprint.folder_plan.as_ref() {
        Some(fp) => fp,
        None => {
            tracing::debug!("no folder_plan, skipping auto-correct");
            return Ok(());
        }
    };

    let watch = match blueprint.watch.as_ref() {
        Some(w) if w.auto_correct => w,
        _ => {
            tracing::debug!("auto_correct not enabled, skipping");
            return Ok(());
        }
    };

    let variables = crate::blueprints::folder_plan_default_variables(Some(folder_plan));
    let source = PathBuf::from(folder_path);
    let root = PathBuf::from(&blueprint.root_path);

    let parent = source.parent().unwrap_or(&root);
    if parent != root {
        tracing::debug!(parent = %parent.display(), root = %root.display(), "folder not in root, skipping");
        return Ok(());
    }

    let correct_name = resolve_correct_folder_name(
        folder_name,
        watch.naming_pattern.as_ref(),
        folder_plan,
        &variables,
    )
    .map_err(|e| DomainError::Validation(e))?;

    tracing::info!(%folder_name, %correct_name, "resolved folder name");

    if correct_name == folder_name {
        tracing::info!("name already correct, applying children only");
        apply_children_structure(audit, &blueprint, &source, &variables).await?;
        return Ok(());
    }

    let target = parent.join(&correct_name);
    if target.exists() {
        tracing::warn!(
            %folder_name,
            %correct_name,
            "auto-correct target already exists, skipping rename"
        );
        apply_children_structure(audit, &blueprint, &target, &variables).await?;
        return Ok(());
    }

    if let Some(parent_dir) = target.parent() {
        fs::create_dir_all(parent_dir).map_err(|e| DomainError::Database(e.to_string()))?;
    }

    if fs::rename(&source, &target).is_err() {
        copy_dir_recursive(&source, &target)
            .map_err(|e| DomainError::Database(e))?;
        fs::remove_dir_all(&source).map_err(|e| DomainError::Database(e.to_string()))?;
    }

    tracing::info!(
        %folder_name,
        %correct_name,
        "auto-corrected folder name"
    );

    apply_children_structure(audit, &blueprint, &target, &variables).await?;
    Ok(())
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
