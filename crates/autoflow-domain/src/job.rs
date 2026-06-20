use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

use uuid::Uuid;



use crate::{

    DomainError, FileFilter, NotifyConfig, ScheduleConfig, ScriptsConfig, TransferOptions,
    WatchConfig,
};



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

#[serde(rename_all = "camelCase")]

pub struct Job {

    pub id: Uuid,

    pub name: String,

    pub source_path: String,

    pub target_path: String,

    pub mode: JobMode,

    pub conflict: ConflictStrategy,

    #[serde(default)]

    pub filters: FileFilter,

    #[serde(default)]

    pub options: TransferOptions,

    #[serde(default)]
    pub schedule: Option<ScheduleConfig>,

    #[serde(default)]
    pub watch: Option<WatchConfig>,

    #[serde(default)]
    pub scripts: ScriptsConfig,

    #[serde(default)]

    pub notify: NotifyConfig,

    pub enabled: bool,

    pub last_run: Option<DateTime<Utc>>,

    pub next_run: Option<DateTime<Utc>>,

    /// Legacy field migrated into `filters` by `normalize()`.

    #[serde(default, skip_serializing)]

    include_extensions: Vec<String>,

    #[serde(default, skip_serializing)]

    recursive: Option<bool>,

}



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

#[serde(rename_all = "camelCase")]

pub struct JobSummary {

    pub id: Uuid,

    pub name: String,

    pub source_path: String,

    pub target_path: String,

    pub mode: JobMode,

    pub enabled: bool,

    pub last_run: Option<DateTime<Utc>>,

    pub next_run: Option<DateTime<Utc>>,

    pub schedule_enabled: bool,

    pub watch_enabled: bool,

}



#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]

#[serde(rename_all = "lowercase")]

pub enum JobMode {

    Copy,

    Move,

}



#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]

#[serde(rename_all = "lowercase")]

pub enum ConflictStrategy {

    Skip,

    Overwrite,

    Rename,

}



impl Job {

    pub fn new(name: impl Into<String>, source_path: impl Into<String>, target_path: impl Into<String>) -> Self {

        Self {

            id: Uuid::new_v4(),

            name: name.into(),

            source_path: source_path.into(),

            target_path: target_path.into(),

            mode: JobMode::Copy,

            conflict: ConflictStrategy::Skip,

            filters: FileFilter::default(),

            options: TransferOptions::default(),

            schedule: None,

            watch: None,

            scripts: ScriptsConfig::default(),

            notify: NotifyConfig::default(),

            enabled: true,

            last_run: None,

            next_run: None,

            include_extensions: Vec::new(),

            recursive: None,

        }

    }



    pub fn normalize(&mut self) {

        if self.filters.include_extensions.is_empty() && !self.include_extensions.is_empty() {

            self.filters.include_extensions = std::mem::take(&mut self.include_extensions);

        }

        if let Some(recursive) = self.recursive {

            self.filters.recursive = recursive;

        }

        self.recursive = None;

        self.include_extensions.clear();

        let watch_active = self.watch.as_ref().is_some_and(|w| w.enabled);

        if watch_active {
            if let Some(schedule) = &mut self.schedule {
                schedule.enabled = false;
            }
        } else if self.schedule.as_ref().is_some_and(|s| s.enabled) {
            if let Some(watch) = &mut self.watch {
                watch.enabled = false;
            }
        }

        if watch_active {
            self.next_run = None;
        } else if let Some(schedule) = &self.schedule {

            if schedule.enabled {

                self.next_run = schedule.compute_next_run(Utc::now());

            } else {

                self.next_run = None;

            }

        } else {

            self.next_run = None;

        }

    }



    pub fn validate(&self) -> Result<(), DomainError> {

        let name = self.name.trim();

        if name.is_empty() || name.len() > 120 {

            return Err(DomainError::Validation(

                "name must be between 1 and 120 characters".into(),

            ));

        }

        if self.source_path.trim().is_empty() {

            return Err(DomainError::Validation("source_path is required".into()));

        }

        if self.target_path.trim().is_empty() {

            return Err(DomainError::Validation("target_path is required".into()));

        }

        if let Some(schedule) = &self.schedule {

            schedule.validate()?;

        }

        if let Some(watch) = &self.watch {

            watch.validate()?;

        }

        self.scripts.validate()?;

        Ok(())

    }



    pub fn summary(&self) -> JobSummary {

        JobSummary {

            id: self.id,

            name: self.name.clone(),

            source_path: self.source_path.clone(),

            target_path: self.target_path.clone(),

            mode: self.mode,

            enabled: self.enabled,

            last_run: self.last_run,

            next_run: self.next_run,

            schedule_enabled: self.schedule.as_ref().is_some_and(|s| s.enabled),

            watch_enabled: self.watch.as_ref().is_some_and(|w| w.enabled),

        }

    }

}



impl Default for Job {

    fn default() -> Self {

        Self::new("Novo fluxo", "", "")

    }

}


