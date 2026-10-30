use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::naming::NamingPattern;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WatchConfig {
    pub enabled: bool,
    #[serde(default = "default_settle_seconds")]
    pub settle_seconds: u32,
    #[serde(default)]
    pub detection: WatchDetectionMode,
    #[serde(default = "default_events")]
    pub events: Vec<WatchEventKind>,
    #[serde(default)]
    pub auto_correct: bool,
    #[serde(default)]
    pub naming_pattern: Option<NamingPattern>,
}

fn default_settle_seconds() -> u32 {
    2
}

fn default_events() -> Vec<WatchEventKind> {
    vec![WatchEventKind::Create]
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            settle_seconds: default_settle_seconds(),
            detection: WatchDetectionMode::default(),
            events: default_events(),
            auto_correct: false,
            naming_pattern: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WatchDetectionMode {
    #[default]
    Realtime,
    Polling {
        #[serde(default = "default_poll_interval")]
        interval_secs: u32,
    },
    Hybrid {
        #[serde(default = "default_poll_interval")]
        poll_interval_secs: u32,
    },
}

fn default_poll_interval() -> u32 {
    30
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WatchEventKind {
    Create,
    Modify,
    Remove,
    Rename,
}

impl WatchConfig {
    pub fn validate(&self) -> Result<(), crate::DomainError> {
        if !self.enabled {
            return Ok(());
        }
        if self.settle_seconds == 0 || self.settle_seconds > 60 {
            return Err(crate::DomainError::Validation(
                "watch settle_seconds must be between 1 and 60".into(),
            ));
        }
        if self.events.is_empty() {
            return Err(crate::DomainError::Validation(
                "watch must include at least one event kind".into(),
            ));
        }
        if let Some(ref np) = self.naming_pattern {
            np.validate()?;
        }
        match &self.detection {
            WatchDetectionMode::Realtime => Ok(()),
            WatchDetectionMode::Polling { interval_secs }
            | WatchDetectionMode::Hybrid {
                poll_interval_secs: interval_secs,
            } => {
                if *interval_secs < 5 || *interval_secs > 3600 {
                    Err(crate::DomainError::Validation(
                        "watch polling interval must be between 5 and 3600 seconds".into(),
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn poll_interval_secs(&self) -> Option<u32> {
        match &self.detection {
            WatchDetectionMode::Realtime => None,
            WatchDetectionMode::Polling { interval_secs } => Some(*interval_secs),
            WatchDetectionMode::Hybrid { poll_interval_secs } => Some(*poll_interval_secs),
        }
    }

    pub fn uses_realtime(&self) -> bool {
        matches!(
            self.detection,
            WatchDetectionMode::Realtime | WatchDetectionMode::Hybrid { .. }
        )
    }

    pub fn uses_polling(&self) -> bool {
        matches!(
            self.detection,
            WatchDetectionMode::Polling { .. } | WatchDetectionMode::Hybrid { .. }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NormalizedWatchEvent {
    FolderCreated {
        blueprint_id: Uuid,
        path: String,
        folder_name: String,
    },
    BlueprintTriggered {
        blueprint_id: Uuid,
    },
    JobTriggered {
        job_id: Uuid,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_events_when_enabled() {
        let config = WatchConfig {
            enabled: true,
            events: vec![],
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn polling_requires_minimum_interval() {
        let config = WatchConfig {
            enabled: true,
            detection: WatchDetectionMode::Polling { interval_secs: 2 },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_invalid_naming_regex() {
        let config = WatchConfig {
            enabled: true,
            naming_pattern: Some(NamingPattern {
                regex: Some("[invalid".to_string()),
                template_override: None,
            }),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn accepts_valid_naming_regex() {
        let config = WatchConfig {
            enabled: true,
            naming_pattern: Some(NamingPattern {
                regex: Some("^\\d{4}.*".to_string()),
                template_override: None,
            }),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn auto_correct_defaults_to_false() {
        let config = WatchConfig::default();
        assert!(!config.auto_correct);
        assert!(config.naming_pattern.is_none());
    }
}
