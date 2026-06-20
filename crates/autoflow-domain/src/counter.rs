use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CounterConfig {
    #[serde(default)]
    pub scope: CounterScope,
    #[serde(default = "default_start")]
    pub start: u64,
    #[serde(default = "default_padding")]
    pub padding: u32,
}

fn default_start() -> u64 {
    1
}

fn default_padding() -> u32 {
    0
}

impl Default for CounterConfig {
    fn default() -> Self {
        Self {
            scope: CounterScope::default(),
            start: default_start(),
            padding: default_padding(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum CounterScope {
    #[default]
    Global,
    PerDay,
    PerFolder,
    PerParent,
}

impl CounterConfig {
    pub fn scope_key(&self, parent: &str, folder: &str, date_key: &str) -> String {
        match self.scope {
            CounterScope::Global => "global".into(),
            CounterScope::PerDay => format!("day:{date_key}"),
            CounterScope::PerFolder => format!("folder:{folder}"),
            CounterScope::PerParent => format!("parent:{parent}"),
        }
    }

    pub fn format_value(&self, value: u64) -> String {
        if self.padding == 0 {
            return value.to_string();
        }
        format!("{value:0width$}", width = self.padding as usize)
    }
}
