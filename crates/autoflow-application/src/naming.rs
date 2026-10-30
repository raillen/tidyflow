use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use autoflow_domain::{
    evaluate, is_path_under_root, normalize_path, validate_file_name, AuditStatus, Blueprint,
    BlueprintApplyResult, BlueprintKind, DomainError, NamingPattern, NewAuditEntry,
};

use crate::blueprints::{
    build_folder_action_plan_for_nodes, execute_folder_action_item, folder_plan_variable_values,
    folder_token_context, parse_template_display_string,
};
use crate::ports::AuditStore;

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
    let normalized = normalize_variables(folder_plan, variables);
    let ctx = folder_token_context(Path::new(folder_name), 1, &normalized);

    if let Some(np) = naming_pattern {
        if let Some(ref tpl) = np.template_override {
            let resolved = evaluate(tpl, &ctx).map_err(|e| e.to_string())?;
            if np.regex.is_some() || !resolved.trim().is_empty() {
                return Ok(np.matches(folder_name, Some(&resolved)));
            }
        }
        if np.regex.is_some() {
            return Ok(np.matches(folder_name, None));
        }
    }

    let root_node = folder_plan.nodes.first();
    if let Some(root_node) = root_node {
        if !root_node.name.trim().is_empty() {
            let pipeline = parse_template_display_string(&root_node.name);
            let resolved = evaluate(&pipeline, &ctx).map_err(|e| e.to_string())?;
            if !resolved.trim().is_empty() {
                return Ok(folder_name == resolved);
            }
        }
    }

    Ok(true)
}

pub fn resolve_correct_folder_name(
    folder_name: &str,
    naming_pattern: Option<&NamingPattern>,
    folder_plan: &autoflow_domain::FolderPlan,
    node: Option<&autoflow_domain::FolderActionNode>,
    variables: &BTreeMap<String, String>,
) -> Result<String, String> {
    let normalized = normalize_variables(folder_plan, variables);
    let ctx = folder_token_context(Path::new(folder_name), 1, &normalized);

    if let Some(np) = naming_pattern {
        if let Some(ref tpl) = np.template_override {
            let resolved = evaluate(tpl, &ctx).map_err(|e| e.to_string())?;
            if !resolved.trim().is_empty() {
                return Ok(resolved);
            }
        }
    }

    let root_node = node.or_else(|| folder_plan.nodes.first());
    if let Some(root_node) = root_node {
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
    children: &[autoflow_domain::FolderActionNode],
    variables: &BTreeMap<String, String>,
) -> Result<BlueprintApplyResult, DomainError> {
    let folder_plan = blueprint
        .folder_plan
        .as_ref()
        .ok_or_else(|| DomainError::Validation("folder_plan required for auto-correct".into()))?;

    let plan = build_folder_action_plan_for_nodes(
        blueprint,
        folder_plan,
        children,
        renamed_folder,
        variables,
        false,
        false,
    )?;

    let mut processed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;

    for item in plan.items {
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

    Ok(BlueprintApplyResult {
        processed,
        failed,
        skipped,
    })
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
    let root = normalize_path(&PathBuf::from(&blueprint.root_path));
    let source = normalize_path(&PathBuf::from(folder_path));

    let parent = source.parent().unwrap_or(&root);
    if !is_path_under_root(parent, &root) {
        tracing::debug!(parent = %parent.display(), root = %root.display(), "folder not under root, skipping");
        return Ok(());
    }

    // Resolve the relative path of the parent from the blueprint root.
    let relative_parent = parent
        .strip_prefix(&root)
        .map(|p| p.to_string_lossy().replace("\\\\", "/"))
        .unwrap_or_default()
        .trim_matches('/')
        .to_string();

    // Find the folder-plan node whose resolved path matches the parent.
    let root_node = if relative_parent.is_empty() {
        folder_plan.nodes.first()
    } else {
        find_folder_plan_node(&folder_plan.nodes, &relative_parent, &variables)
    };

    let Some(root_node) = root_node else {
        tracing::debug!(parent = %parent.display(), "no matching folder plan node");
        return Err(DomainError::FolderOutOfPattern(relative_parent));
    };

    let correct_name = resolve_correct_folder_name(
        folder_name,
        watch.naming_pattern.as_ref(),
        folder_plan,
        Some(root_node),
        &variables,
    )
    .map_err(DomainError::Validation)?;

    validate_file_name(&correct_name).map_err(|error| {
        DomainError::Validation(format!(
            "auto-correct produced an invalid folder name ({correct_name}): {error}"
        ))
    })?;

    tracing::info!(%folder_name, %correct_name, "resolved folder name");

    if correct_name == folder_name {
        tracing::info!("name already correct, applying children only");
        apply_children_structure(audit, &blueprint, &source, &root_node.children, &variables)
            .await?;
        return Ok(());
    }

    let target = parent.join(&correct_name);
    if target.exists() {
        tracing::warn!(
            %folder_name,
            %correct_name,
            "auto-correct target already exists, skipping rename"
        );
        apply_children_structure(audit, &blueprint, &target, &root_node.children, &variables)
            .await?;
        return Ok(());
    }

    if let Some(parent_dir) = target.parent() {
        fs::create_dir_all(parent_dir).map_err(|e| DomainError::Database(e.to_string()))?;
    }

    let started = Instant::now();
    if fs::rename(&source, &target).is_err() {
        copy_dir_recursive(&source, &target).map_err(DomainError::Database)?;
        fs::remove_dir_all(&source).map_err(|e| DomainError::Database(e.to_string()))?;
    }

    tracing::info!(
        %folder_name,
        %correct_name,
        "auto-corrected folder name"
    );

    audit
        .append(NewAuditEntry {
            job_id: None,
            blueprint_id: Some(blueprint.id),
            job_name: format!("{} (auto-correct)", blueprint.name),
            source_path: source.display().to_string(),
            target_path: target.display().to_string(),
            status: AuditStatus::Organized,
            file_size: 0,
            duration_ms: started.elapsed().as_secs_f64() * 1000.0,
            details: Some("auto-correct rename".into()),
        })
        .await?;

    apply_children_structure(audit, &blueprint, &target, &root_node.children, &variables).await?;
    Ok(())
}

fn find_folder_plan_node<'a>(
    nodes: &'a [autoflow_domain::FolderActionNode],
    relative_parent: &str,
    variables: &BTreeMap<String, String>,
) -> Option<&'a autoflow_domain::FolderActionNode> {
    let target = relative_parent.trim_matches('/').to_string();
    if target.is_empty() {
        return nodes.first();
    }

    let components: Vec<&str> = target.split('/').filter(|s| !s.is_empty()).collect();
    let mut current_nodes = nodes;
    let mut current_node: Option<&autoflow_domain::FolderActionNode> = None;

    for component in &components {
        let mut found = false;
        for node in current_nodes {
            let ctx = folder_token_context(std::path::Path::new("."), 1, variables);
            let pipeline = parse_template_display_string(&node.name);
            let resolved = evaluate(&pipeline, &ctx)
                .ok()
                .unwrap_or_else(|| node.name.clone());
            if resolved == *component {
                current_node = Some(node);
                current_nodes = &node.children;
                found = true;
                break;
            }
        }
        if !found {
            return None;
        }
    }

    current_node
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if file_type.is_symlink() {
            return Err(format!(
                "refusing to copy symlink during auto-correct: {}",
                source_path.display()
            ));
        }
        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
