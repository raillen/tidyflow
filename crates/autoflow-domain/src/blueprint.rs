use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::counter::CounterConfig;
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
pub struct FolderPlan {
    pub nodes: Vec<FolderNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FolderNode {
    pub name: String,
    #[serde(default)]
    pub children: Vec<FolderNode>,
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
        if self.kind == BlueprintKind::Folder && self.folder_plan.is_none() {
            // folder_plan is optional scaffolding; routing still required via path_template
        }
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
