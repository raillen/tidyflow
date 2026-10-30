use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::counter::CounterConfig;
use crate::path_policy::validate_relative_path;
use crate::tokenizer::TemplatePipeline;
use crate::{ConflictStrategy, DomainError, FileFilter, WatchConfig};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Blueprint {
    pub id: Uuid,
    pub name: String,
    pub kind: BlueprintKind,
    pub root_path: String,
    #[serde(default)]
    pub search: FileFilter,
    pub routing: RoutingConfig,
    pub operation: BlueprintOperation,
    #[serde(default = "default_recursive")]
    pub recursive: bool,
    pub conflict: ConflictStrategy,
    #[serde(default)]
    pub rename_template: Option<TemplatePipeline>,
    #[serde(default)]
    pub folder_plan: Option<FolderPlan>,
    #[serde(default)]
    pub watch: Option<WatchConfig>,
    #[serde(default)]
    pub counter: CounterConfig,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
}

fn default_recursive() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BlueprintKind {
    File,
    Folder,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BlueprintOperation {
    Move,
    Copy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RoutingConfig {
    pub path_template: TemplatePipeline,
    #[serde(default = "default_create_intermediate_dirs")]
    pub create_intermediate_dirs: bool,
}

fn default_create_intermediate_dirs() -> bool {
    true
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            path_template: TemplatePipeline::default(),
            create_intermediate_dirs: default_create_intermediate_dirs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FolderPlanPreviewNode {
    pub name: String,
    pub relative_path: String,
    #[serde(default)]
    pub action: FolderAction,
    #[serde(default)]
    pub source_path: String,
    pub children: Vec<FolderPlanPreviewNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FolderPlanPreview {
    pub root_path: String,
    pub nodes: Vec<FolderPlanPreviewNode>,
    pub folder_count: u32,
    pub valid: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FolderPlan {
    #[serde(default = "default_folder_plan_version")]
    pub version: u32,
    #[serde(default)]
    pub variables: Vec<FolderPlanVariable>,
    #[serde(default)]
    pub allow_delete_non_empty: bool,
    #[serde(default)]
    pub confirm_delete_non_empty: bool,
    #[serde(default)]
    pub nodes: Vec<FolderActionNode>,
}

fn default_folder_plan_version() -> u32 {
    1
}

impl Default for FolderPlan {
    fn default() -> Self {
        Self {
            version: default_folder_plan_version(),
            variables: Vec::new(),
            allow_delete_non_empty: false,
            confirm_delete_non_empty: false,
            nodes: Vec::new(),
        }
    }
}

impl FolderPlan {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.version == 0 {
            return Err(DomainError::Validation(
                "folder plan version must be at least 1".into(),
            ));
        }

        let mut seen = BTreeSet::new();
        for variable in &self.variables {
            let name = variable.name.trim();
            if name.is_empty() {
                return Err(DomainError::Validation(
                    "folder variable name cannot be empty".into(),
                ));
            }
            let mut chars = name.chars();
            let first = chars.next().expect("non-empty name");
            if !(first.is_ascii_alphabetic() || first == '_')
                || !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                return Err(DomainError::Validation(format!(
                    "folder variable name must be an identifier: {name}"
                )));
            }
            let key: String = name
                .chars()
                .filter(|ch| ch.is_ascii_alphanumeric())
                .map(|ch| ch.to_ascii_lowercase())
                .collect();
            if !seen.insert(key) {
                return Err(DomainError::Validation(format!(
                    "duplicate folder variable name: {name}"
                )));
            }
        }

        for node in &self.nodes {
            validate_folder_action_node(node)?;
        }

        Ok(())
    }
}

fn validate_folder_action_node(node: &FolderActionNode) -> Result<(), DomainError> {
    if !node.source_path.trim().is_empty() {
        validate_relative_path(&node.source_path).map_err(DomainError::Validation)?;
    }
    if !node.target_path.trim().is_empty() {
        validate_relative_path(&node.target_path).map_err(DomainError::Validation)?;
    }
    for child in &node.children {
        validate_folder_action_node(child)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum FolderAction {
    #[default]
    Create,
    Rename,
    Move,
    Copy,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FolderActionNode {
    #[serde(default)]
    pub action: FolderAction,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub source_path: String,
    #[serde(default)]
    pub target_path: String,
    #[serde(default)]
    pub children: Vec<FolderActionNode>,
}

impl FolderActionNode {
    pub fn create(name: impl Into<String>) -> Self {
        Self {
            action: FolderAction::Create,
            name: name.into(),
            source_path: String::new(),
            target_path: String::new(),
            children: Vec::new(),
        }
    }
}

impl Default for FolderActionNode {
    fn default() -> Self {
        Self::create("")
    }
}

/// Backwards-compatible alias for the pre-folder-action model.
pub type FolderNode = FolderActionNode;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FolderPlanVariable {
    pub name: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub default_value: String,
    #[serde(default = "default_variable_required")]
    pub required: bool,
}

fn default_variable_required() -> bool {
    true
}

impl Default for FolderPlanVariable {
    fn default() -> Self {
        Self {
            name: String::new(),
            label: String::new(),
            default_value: String::new(),
            required: default_variable_required(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueprintSummary {
    pub id: Uuid,
    pub name: String,
    pub kind: BlueprintKind,
    pub root_path: String,
    pub operation: BlueprintOperation,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub watch_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlueprintSimulationReport {
    pub matched: u32,
    pub skipped: u32,
    pub plan_sample: Vec<BlueprintPlanSample>,
    pub warnings: Vec<String>,
    pub collisions: Vec<BlueprintCollision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlueprintPlanSample {
    pub source: String,
    pub target: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlueprintCollision {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlueprintApplyResult {
    pub processed: u32,
    pub failed: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplatePreview {
    pub result_path: String,
    pub result_name: String,
    pub valid: bool,
    pub warnings: Vec<String>,
}

impl Blueprint {
    pub fn new(name: impl Into<String>, kind: BlueprintKind, root_path: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind,
            root_path: root_path.into(),
            search: FileFilter::default(),
            routing: RoutingConfig::default(),
            operation: BlueprintOperation::Move,
            recursive: true,
            conflict: ConflictStrategy::Skip,
            rename_template: None,
            folder_plan: None,
            watch: None,
            counter: CounterConfig::default(),
            enabled: true,
            last_run: None,
        }
    }

    pub fn normalize(&mut self) {
        self.search.recursive = self.recursive;

        let watch_active = self.watch.as_ref().is_some_and(|w| w.enabled);
        if watch_active {
            if let Some(watch) = &mut self.watch {
                watch.enabled = true;
            }
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        let name = self.name.trim();
        if name.is_empty() || name.len() > 120 {
            return Err(DomainError::Validation(
                "name must be between 1 and 120 characters".into(),
            ));
        }
        if self.root_path.trim().is_empty() {
            return Err(DomainError::Validation("root_path is required".into()));
        }
        if self.kind == BlueprintKind::File && self.routing.path_template.segments.is_empty() {
            return Err(DomainError::Validation(
                "file blueprints require a routing template".into(),
            ));
        }
        if self.kind == BlueprintKind::Folder && self.folder_plan.is_none() {
            return Err(DomainError::Validation(
                "folder blueprints require a folder plan".into(),
            ));
        }
        if self
            .rename_template
            .as_ref()
            .is_some_and(|template| template.segments.is_empty())
        {
            return Err(DomainError::Validation(
                "rename template cannot be empty".into(),
            ));
        }
        if let Some(folder_plan) = &self.folder_plan {
            folder_plan.validate()?;
        }
        self.counter.validate()?;
        if let Some(watch) = &self.watch {
            watch.validate()?;
        }
        Ok(())
    }

    pub fn summary(&self) -> BlueprintSummary {
        BlueprintSummary {
            id: self.id,
            name: self.name.clone(),
            kind: self.kind,
            root_path: self.root_path.clone(),
            operation: self.operation,
            enabled: self.enabled,
            last_run: self.last_run,
            watch_enabled: self.watch.as_ref().is_some_and(|w| w.enabled),
        }
    }
}

impl Default for Blueprint {
    fn default() -> Self {
        Self::new("Novo blueprint", BlueprintKind::File, "")
    }
}
