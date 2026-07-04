use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::TemplatePipeline;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NamingPattern {
    #[serde(default)]
    pub regex: Option<String>,
    #[serde(default)]
    pub template_override: Option<TemplatePipeline>,
}

impl Default for NamingPattern {
    fn default() -> Self {
        Self {
            regex: None,
            template_override: None,
        }
    }
}

impl NamingPattern {
    pub fn compiled_regex(&self) -> Result<Option<Regex>, String> {
        match &self.regex {
            Some(pattern) => {
                let re =
                    Regex::new(pattern).map_err(|e| format!("invalid naming regex: {e}"))?;
                Ok(Some(re))
            }
            None => Ok(None),
        }
    }

    pub fn matches(&self, name: &str, resolved_template: Option<&str>) -> bool {
        if let Some(pattern) = &self.regex {
            if let Ok(re) = Regex::new(pattern) {
                return re.is_match(name);
            }
        }
        if let Some(expected) = resolved_template {
            return name == expected;
        }
        true
    }

    pub fn validate(&self) -> Result<(), crate::DomainError> {
        if let Some(pattern) = &self.regex {
            Regex::new(pattern).map_err(|e| {
                crate::DomainError::Validation(format!("invalid naming regex: {e}"))
            })?;
        }
        Ok(())
    }
}
