use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum SymlinkMode {
    #[default]
    Follow,
    CopyLink,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileFilter {
    pub include_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub exclude_preset_ids: Vec<String>,
    pub name_regex: Option<String>,
    pub path_regex: Option<String>,
    pub min_size_bytes: Option<u64>,
    pub max_size_bytes: Option<u64>,
    pub max_depth: Option<u32>,
    pub modified_after: Option<chrono::DateTime<chrono::Utc>>,
    pub modified_before: Option<chrono::DateTime<chrono::Utc>>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
    pub older_than_days: Option<u32>,
    pub content_contains: Option<String>,
    #[serde(default = "default_content_max_bytes")]
    pub content_max_bytes: u64,
    #[serde(default = "default_content_extensions")]
    pub content_extensions: Vec<String>,
    #[serde(default = "default_true")]
    pub recursive: bool,
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default)]
    pub symlink_mode: SymlinkMode,
    #[serde(default)]
    pub skip_empty_files: bool,
}

fn default_content_max_bytes() -> u64 {
    5 * 1024 * 1024
}

fn default_content_extensions() -> Vec<String> {
    vec![
        ".txt".into(),
        ".md".into(),
        ".csv".into(),
        ".json".into(),
        ".log".into(),
        ".xml".into(),
        ".yaml".into(),
        ".yml".into(),
    ]
}

fn default_true() -> bool {
    true
}

impl Default for FileFilter {
    fn default() -> Self {
        Self {
            include_extensions: Vec::new(),
            exclude_patterns: Vec::new(),
            exclude_preset_ids: Vec::new(),
            name_regex: None,
            path_regex: None,
            min_size_bytes: None,
            max_size_bytes: None,
            max_depth: None,
            modified_after: None,
            modified_before: None,
            created_after: None,
            created_before: None,
            older_than_days: None,
            content_contains: None,
            content_max_bytes: default_content_max_bytes(),
            content_extensions: default_content_extensions(),
            recursive: true,
            include_hidden: false,
            symlink_mode: SymlinkMode::Follow,
            skip_empty_files: false,
        }
    }
}

pub fn built_in_exclude_presets() -> &'static [(&'static str, &'static [&'static str])] {
    &[
        ("node_modules", &["**/node_modules/**"]),
        ("git", &["**/.git/**", "**/.gitignore"]),
        ("temp", &["**/temp/**", "**/tmp/**", "**/*.tmp"]),
        ("system", &["**/Thumbs.db", "**/.DS_Store", "**/desktop.ini"]),
        ("build", &["**/target/**", "**/dist/**", "**/build/**", "**/bin/**", "**/obj/**"]),
    ]
}

impl FileFilter {
    pub fn expanded_exclude_patterns(&self) -> Vec<String> {
        let mut patterns = self.exclude_patterns.clone();
        for preset in built_in_exclude_presets() {
            if self.exclude_preset_ids.iter().any(|id| id == preset.0) {
                for p in preset.1 {
                    if !patterns.iter().any(|x| x == *p) {
                        patterns.push((*p).to_string());
                    }
                }
            }
        }
        patterns
    }
}
