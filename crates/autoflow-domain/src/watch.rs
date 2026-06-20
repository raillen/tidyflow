use serde::{Deserialize, Serialize};

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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WatchDetectionMode {
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

impl Default for WatchDetectionMode {
    fn default() -> Self {
        Self::Realtime
    }
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
        match &self.detection {
            WatchDetectionMode::Realtime => Ok(()),
            WatchDetectionMode::Polling { interval_secs }
            | WatchDetectionMode::Hybrid { poll_interval_secs: interval_secs } => {
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
            WatchDetectionMode::Hybrid {
                poll_interval_secs,
            } => Some(*poll_interval_secs),
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
}
