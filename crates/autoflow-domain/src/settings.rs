use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub theme: ThemeMode,
    #[serde(default = "default_accent_color")]
    pub accent_color: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_interface_font")]
    pub interface_font: String,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub close_to_tray: bool,
    #[serde(default)]
    pub start_minimized: bool,
    #[serde(default)]
    pub bandwidth_limit_mbps: u32,
    #[serde(default = "default_max_parallel_files")]
    pub max_parallel_files: u32,
    #[serde(default = "default_log_retention_days")]
    pub log_retention_days: u32,
    #[serde(default)]
    pub performance: PerformanceSettings,
    #[serde(default)]
    pub security: SecuritySettings,
    #[serde(default)]
    pub notifications: NotificationSettings,
    #[serde(default)]
    pub maintenance: MaintenanceSettings,
    #[serde(default)]
    pub support: SupportSettings,
    #[serde(default)]
    pub about: AboutSettings,
    #[serde(default)]
    pub admin: AdminSettings,
}

fn default_accent_color() -> String {
    "#0064ff".into()
}

fn default_language() -> String {
    "pt-BR".into()
}

fn default_interface_font() -> String {
    "system".into()
}

fn default_max_parallel_files() -> u32 {
    1
}

fn default_log_retention_days() -> u32 {
    30
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            accent_color: default_accent_color(),
            language: default_language(),
            interface_font: default_interface_font(),
            autostart: false,
            close_to_tray: true,
            start_minimized: false,
            bandwidth_limit_mbps: 0,
            max_parallel_files: default_max_parallel_files(),
            log_retention_days: default_log_retention_days(),
            performance: PerformanceSettings::default(),
            security: SecuritySettings::default(),
            notifications: NotificationSettings::default(),
            maintenance: MaintenanceSettings::default(),
            support: SupportSettings::default(),
            about: AboutSettings::default(),
            admin: AdminSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ProcessPriority {
    Low,
    #[default]
    Normal,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceSettings {
    #[serde(default = "default_max_threads")]
    pub max_threads: u32,
    #[serde(default)]
    pub memory_limit_mb: u32,
    #[serde(default)]
    pub process_priority: ProcessPriority,
    #[serde(default)]
    pub global_bandwidth_limit_mbps: u32,
    #[serde(default = "default_queue_poll_interval_ms")]
    pub queue_poll_interval_ms: u32,
    #[serde(default)]
    pub pause_when_on_battery: bool,
}

fn default_max_threads() -> u32 {
    2
}

fn default_queue_poll_interval_ms() -> u32 {
    500
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            max_threads: default_max_threads(),
            memory_limit_mb: 0,
            process_priority: ProcessPriority::Normal,
            global_bandwidth_limit_mbps: 0,
            queue_poll_interval_ms: default_queue_poll_interval_ms(),
            pause_when_on_battery: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SecuritySettings {
    #[serde(default)]
    pub pin_enabled: bool,
    #[serde(default)]
    pub access_pin_hash: Option<String>,
    #[serde(default)]
    pub require_pin_on_startup: bool,
    #[serde(default)]
    pub lock_on_minimize: bool,
    #[serde(default)]
    pub lock_on_tray: bool,
    #[serde(default)]
    pub encryption_enabled: bool,
    #[serde(default)]
    pub master_key_hint: String,
    #[serde(default)]
    pub mask_sensitive_paths: bool,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            pin_enabled: false,
            access_pin_hash: None,
            require_pin_on_startup: false,
            lock_on_minimize: false,
            lock_on_tray: false,
            encryption_enabled: false,
            master_key_hint: String::new(),
            mask_sensitive_paths: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub desktop_enabled: bool,
    #[serde(default)]
    pub webhook_enabled: bool,
    #[serde(default)]
    pub webhooks: Vec<WebhookSettings>,
    #[serde(default)]
    pub smtp: SmtpSettings,
    #[serde(default)]
    pub admin_panel_enabled: bool,
    #[serde(default)]
    pub notify_on_success: bool,
    #[serde(default = "default_true")]
    pub notify_on_failure: bool,
}

fn default_true() -> bool {
    true
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            desktop_enabled: true,
            webhook_enabled: false,
            webhooks: Vec::new(),
            smtp: SmtpSettings::default(),
            admin_panel_enabled: false,
            notify_on_success: false,
            notify_on_failure: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WebhookSettings {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub secret_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SmtpSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub from_address: String,
    #[serde(default = "default_true")]
    pub use_tls: bool,
    #[serde(default)]
    pub password_configured: bool,
}

fn default_smtp_port() -> u16 {
    587
}

impl Default for SmtpSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            port: default_smtp_port(),
            username: String::new(),
            from_address: String::new(),
            use_tls: true,
            password_configured: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceSettings {
    #[serde(default = "default_log_retention_days")]
    pub log_retention_days: u32,
    #[serde(default = "default_true")]
    pub auto_compact_database: bool,
    #[serde(default)]
    pub backup_enabled: bool,
    #[serde(default)]
    pub backup_directory: String,
    #[serde(default = "default_backup_interval_hours")]
    pub backup_interval_hours: u32,
    #[serde(default = "default_backup_retention_count")]
    pub backup_retention_count: u32,
    #[serde(default)]
    pub optimize_after_cleanup: bool,
}

fn default_backup_interval_hours() -> u32 {
    24
}

fn default_backup_retention_count() -> u32 {
    7
}

impl Default for MaintenanceSettings {
    fn default() -> Self {
        Self {
            log_retention_days: default_log_retention_days(),
            auto_compact_database: true,
            backup_enabled: false,
            backup_directory: String::new(),
            backup_interval_hours: default_backup_interval_hours(),
            backup_retention_count: default_backup_retention_count(),
            optimize_after_cleanup: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SupportSettings {
    #[serde(default)]
    pub support_email: String,
    #[serde(default)]
    pub pix_key: String,
    #[serde(default)]
    pub bank_deposit_info: String,
    #[serde(default)]
    pub buy_me_coffee_url: String,
    #[serde(default)]
    pub donations_enabled: bool,
}

impl Default for SupportSettings {
    fn default() -> Self {
        Self {
            support_email: "suporte@autoflow.local".into(),
            pix_key: String::new(),
            bank_deposit_info: String::new(),
            buy_me_coffee_url: String::new(),
            donations_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AboutSettings {
    #[serde(default = "default_project_name")]
    pub project_name: String,
    #[serde(default = "default_project_description")]
    pub project_description: String,
    #[serde(default = "default_creator_name")]
    pub creator_name: String,
    #[serde(default = "default_creator_bio")]
    pub creator_bio: String,
    #[serde(default)]
    pub website_url: String,
    #[serde(default)]
    pub github_url: String,
    #[serde(default)]
    pub linkedin_url: String,
}

fn default_project_name() -> String {
    "TidyFlow".into()
}

fn default_project_description() -> String {
    "Automação local para organizar, copiar e mover arquivos com auditoria clara.".into()
}

fn default_creator_name() -> String {
    "Raillen Santos".into()
}

fn default_creator_bio() -> String {
    "Criador do TidyFlow e projetos Zenith.".into()
}

impl Default for AboutSettings {
    fn default() -> Self {
        Self {
            project_name: default_project_name(),
            project_description: default_project_description(),
            creator_name: default_creator_name(),
            creator_bio: default_creator_bio(),
            website_url: String::new(),
            github_url: "https://github.com/raillen".into(),
            linkedin_url: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum AdminAgentMode {
    #[default]
    LocalOnly,
    ManagedAgent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub mode: AdminAgentMode,
    #[serde(default)]
    pub instance_id: Option<String>,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub server_url: String,
    #[serde(default)]
    pub enrollment_token_configured: bool,
    #[serde(default)]
    pub allow_remote_commands: bool,
    #[serde(default)]
    pub allow_batch_commands: bool,
    #[serde(default = "default_heartbeat_interval_secs")]
    pub heartbeat_interval_secs: u32,
    #[serde(default = "default_inventory_interval_secs")]
    pub inventory_interval_secs: u32,
    #[serde(default)]
    pub last_registered_at: Option<DateTime<Utc>>,
}

fn default_heartbeat_interval_secs() -> u32 {
    30
}

fn default_inventory_interval_secs() -> u32 {
    300
}

impl Default for AdminSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: AdminAgentMode::LocalOnly,
            instance_id: None,
            display_name: String::new(),
            server_url: String::new(),
            enrollment_token_configured: false,
            allow_remote_commands: false,
            allow_batch_commands: false,
            heartbeat_interval_secs: default_heartbeat_interval_secs(),
            inventory_interval_secs: default_inventory_interval_secs(),
            last_registered_at: None,
        }
    }
}

impl AppSettings {
    pub fn normalized(mut self) -> Self {
        if self.performance.global_bandwidth_limit_mbps == 0 && self.bandwidth_limit_mbps > 0 {
            self.performance.global_bandwidth_limit_mbps = self.bandwidth_limit_mbps;
        }
        if self.maintenance.log_retention_days == default_log_retention_days()
            && self.log_retention_days != default_log_retention_days()
        {
            self.maintenance.log_retention_days = self.log_retention_days;
        }

        self.bandwidth_limit_mbps = self.performance.global_bandwidth_limit_mbps;
        self.log_retention_days = self.maintenance.log_retention_days;
        self.admin.display_name = self.admin.display_name.trim().to_string();
        self.admin.server_url = self
            .admin
            .server_url
            .trim()
            .trim_end_matches('/')
            .to_string();
        self.admin.instance_id = self
            .admin
            .instance_id
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        self
    }

    pub fn validate(&self) -> Result<(), crate::DomainError> {
        if self.language.trim().is_empty() {
            return Err(crate::DomainError::Validation(
                "language must not be empty".into(),
            ));
        }
        if self.interface_font.trim().is_empty() {
            return Err(crate::DomainError::Validation(
                "interface_font must not be empty".into(),
            ));
        }
        if self.max_parallel_files == 0 {
            return Err(crate::DomainError::Validation(
                "max_parallel_files must be >= 1".into(),
            ));
        }
        if self.performance.max_threads == 0 || self.performance.max_threads > 128 {
            return Err(crate::DomainError::Validation(
                "performance.max_threads must be between 1 and 128".into(),
            ));
        }
        if self.performance.queue_poll_interval_ms < 100 {
            return Err(crate::DomainError::Validation(
                "performance.queue_poll_interval_ms must be >= 100".into(),
            ));
        }
        if self.maintenance.backup_enabled && self.maintenance.backup_directory.trim().is_empty() {
            return Err(crate::DomainError::Validation(
                "maintenance.backup_directory is required when backups are enabled".into(),
            ));
        }
        if self.maintenance.backup_interval_hours == 0 {
            return Err(crate::DomainError::Validation(
                "maintenance.backup_interval_hours must be >= 1".into(),
            ));
        }
        if self.security.pin_enabled && self.security.access_pin_hash.is_none() {
            return Err(crate::DomainError::Validation(
                "security.access_pin_hash is required when PIN is enabled".into(),
            ));
        }
        if !Self::is_valid_hex_color(&self.accent_color) {
            return Err(crate::DomainError::Validation(
                "accent_color must be a hex color like #0064ff".into(),
            ));
        }
        for webhook in &self.notifications.webhooks {
            if webhook.enabled && !is_http_url(&webhook.url) {
                return Err(crate::DomainError::Validation(format!(
                    "webhook URL must start with http:// or https://: {}",
                    webhook.name
                )));
            }
        }
        if self.notifications.smtp.enabled && self.notifications.smtp.host.trim().is_empty() {
            return Err(crate::DomainError::Validation(
                "notifications.smtp.host is required when SMTP is enabled".into(),
            ));
        }
        if self.admin.display_name.len() > 120 {
            return Err(crate::DomainError::Validation(
                "admin.display_name must be <= 120 characters".into(),
            ));
        }
        if self.admin.heartbeat_interval_secs < 10 || self.admin.heartbeat_interval_secs > 3600 {
            return Err(crate::DomainError::Validation(
                "admin.heartbeat_interval_secs must be between 10 and 3600".into(),
            ));
        }
        if self.admin.inventory_interval_secs < 60 || self.admin.inventory_interval_secs > 86_400 {
            return Err(crate::DomainError::Validation(
                "admin.inventory_interval_secs must be between 60 and 86400".into(),
            ));
        }
        if !self.admin.server_url.is_empty() && !is_http_url(&self.admin.server_url) {
            return Err(crate::DomainError::Validation(
                "admin.server_url must start with http:// or https://".into(),
            ));
        }
        if self.admin.enabled
            && self.admin.mode == AdminAgentMode::ManagedAgent
            && self.admin.server_url.is_empty()
        {
            return Err(crate::DomainError::Validation(
                "admin.server_url is required in managed agent mode".into(),
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

fn is_http_url(value: &str) -> bool {
    let lower = value.trim().to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_legacy_flat_settings_into_nested_sections() {
        let mut settings = AppSettings::default();
        settings.bandwidth_limit_mbps = 25;
        settings.log_retention_days = 90;

        let normalized = settings.normalized();

        assert_eq!(normalized.performance.global_bandwidth_limit_mbps, 25);
        assert_eq!(normalized.maintenance.log_retention_days, 90);
    }

    #[test]
    fn normalized_settings_keep_flat_fields_in_sync() {
        let mut settings = AppSettings::default();
        settings.performance.global_bandwidth_limit_mbps = 15;
        settings.maintenance.log_retention_days = 45;

        let normalized = settings.normalized();

        assert_eq!(normalized.bandwidth_limit_mbps, 15);
        assert_eq!(normalized.log_retention_days, 45);
    }

    #[test]
    fn validates_admin_agent_intervals() {
        let mut settings = AppSettings::default();
        settings.admin.heartbeat_interval_secs = 5;

        assert!(settings.validate().is_err());
    }

    #[test]
    fn normalizes_admin_url_and_instance_id() {
        let mut settings = AppSettings::default();
        settings.admin.server_url = " https://admin.local/ ".into();
        settings.admin.instance_id = Some(" local-1 ".into());

        let normalized = settings.normalized();

        assert_eq!(normalized.admin.server_url, "https://admin.local");
        assert_eq!(normalized.admin.instance_id.as_deref(), Some("local-1"));
    }
}
