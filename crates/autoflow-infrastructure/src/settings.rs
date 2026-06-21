use autoflow_application::ports::SettingsStore;
use autoflow_domain::{AppSettings, DomainError};
use sqlx::SqlitePool;

pub struct SqliteSettingsStore {
    pool: SqlitePool,
}

impl SqliteSettingsStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn ensure_default(&self) -> Result<(), DomainError> {
        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM settings WHERE id = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        if exists.0 == 0 {
            let default = AppSettings::default();
            self.save(&default).await?;
        }
        Ok(())
    }

    async fn save(&self, settings: &AppSettings) -> Result<(), DomainError> {
        let payload =
            serde_json::to_string(settings).map_err(|e| DomainError::Database(e.to_string()))?;
        sqlx::query(
            "INSERT INTO settings (id, payload) VALUES (1, ?) ON CONFLICT(id) DO UPDATE SET payload = excluded.payload",
        )
        .bind(payload)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}

impl SettingsStore for SqliteSettingsStore {
    fn get(&self) -> AppSettings {
        // Sync bridge for trait — prefer async methods on store directly in new code.
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let row: (String,) = sqlx::query_as("SELECT payload FROM settings WHERE id = 1")
                    .fetch_one(&self.pool)
                    .await
                    .unwrap_or_else(|_| (serde_json::to_string(&AppSettings::default()).unwrap(),));
                serde_json::from_str::<AppSettings>(&row.0)
                    .unwrap_or_default()
                    .normalized()
            })
        })
    }

    fn update(&self, settings: AppSettings) -> Result<(), DomainError> {
        let settings = settings.normalized();
        settings.validate()?;
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { self.save(&settings).await })
        })
    }
}

impl SqliteSettingsStore {
    pub async fn get_async(&self) -> Result<AppSettings, DomainError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT payload FROM settings WHERE id = 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some((payload,)) => serde_json::from_str::<AppSettings>(&payload)
                .map(AppSettings::normalized)
                .map_err(|e| DomainError::Database(e.to_string())),
            None => Ok(AppSettings::default()),
        }
    }

    pub async fn update_async(&self, settings: AppSettings) -> Result<(), DomainError> {
        let settings = settings.normalized();
        settings.validate()?;
        self.save(&settings).await
    }
}
