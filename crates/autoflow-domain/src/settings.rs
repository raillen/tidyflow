use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: ThemeMode,
    #[serde(default = "default_accent_color")]
    pub accent_color: String,
    pub language: String,
    pub autostart: bool,
    pub bandwidth_limit_mbps: u32,
    pub max_parallel_files: u32,
    pub log_retention_days: u32,
}

fn default_accent_color() -> String {
    "#0064ff".into()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            accent_color: default_accent_color(),
            language: "pt-BR".into(),
            autostart: false,
            bandwidth_limit_mbps: 0,
            max_parallel_files: 1,
            log_retention_days: 30,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), crate::DomainError> {
        if self.language.trim().is_empty() {
            return Err(crate::DomainError::Validation(
                "language must not be empty".into(),
            ));
        }
        if self.max_parallel_files == 0 {
            return Err(crate::DomainError::Validation(
                "max_parallel_files must be >= 1".into(),
            ));
        }
        if !Self::is_valid_hex_color(&self.accent_color) {
            return Err(crate::DomainError::Validation(
                "accent_color must be a hex color like #0064ff".into(),
            ));
        }
        Ok(())
    }

    fn is_valid_hex_color(value: &str) -> bool {
        value.len() == 7
            && value.starts_with('#')
            && value[1..].chars().all(|c| c.is_ascii_hexdigit())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub core: String,
}

impl HealthStatus {
    pub fn ok(version: &str) -> Self {
        Self {
            status: "ok".into(),
            version: version.into(),
            core: "autoflow-core".into(),
        }
    }
}
