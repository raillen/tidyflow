use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScriptsConfig {
    pub pre_script: Option<String>,
    pub post_script: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u32,
}

fn default_timeout() -> u32 {
    60
}

impl ScriptsConfig {
    pub fn validate(&self) -> Result<(), crate::DomainError> {
        if self.timeout_secs < 5 || self.timeout_secs > 600 {
            return Err(crate::DomainError::Validation(
                "script timeout must be between 5 and 600 seconds".into(),
            ));
        }
        Ok(())
    }
}
