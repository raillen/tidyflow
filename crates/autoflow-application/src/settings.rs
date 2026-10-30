use autoflow_domain::{AppSettings, DomainError};

use crate::ports::SettingsStore;

pub fn get_settings(store: &dyn SettingsStore) -> AppSettings {
    store.get()
}

pub fn update_settings(
    store: &dyn SettingsStore,
    settings: AppSettings,
) -> Result<AppSettings, DomainError> {
    settings.validate()?;
    store.update(settings.clone())?;
    Ok(store.get())
}

#[cfg(test)]
mod tests {
    use super::*;
    use autoflow_domain::ThemeMode;
    use std::sync::RwLock;

    struct MemoryStore(RwLock<AppSettings>);

    impl SettingsStore for MemoryStore {
        fn get(&self) -> AppSettings {
            self.0.read().unwrap().clone()
        }

        fn update(&self, settings: AppSettings) -> Result<(), DomainError> {
            *self.0.write().unwrap() = settings;
            Ok(())
        }
    }

    #[test]
    fn rejects_invalid_parallelism() {
        let store = MemoryStore(RwLock::new(AppSettings::default()));
        let bad = AppSettings {
            max_parallel_files: 0,
            ..AppSettings::default()
        };
        assert!(update_settings(&store, bad).is_err());
    }

    #[test]
    fn updates_theme_and_accent() {
        let store = MemoryStore(RwLock::new(AppSettings::default()));
        let next = AppSettings {
            theme: ThemeMode::Dark,
            accent_color: "#7c3aed".into(),
            ..AppSettings::default()
        };
        let saved = update_settings(&store, next).unwrap();
        assert_eq!(saved.theme, ThemeMode::Dark);
        assert_eq!(saved.accent_color, "#7c3aed");
    }

    #[test]
    fn rejects_invalid_accent() {
        let store = MemoryStore(RwLock::new(AppSettings::default()));
        let bad = AppSettings {
            accent_color: "blue".into(),
            ..AppSettings::default()
        };
        assert!(update_settings(&store, bad).is_err());
    }
}
