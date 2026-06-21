use autoflow_domain::DomainError;

const KEYRING_SERVICE: &str = "autoflow";
const ADMIN_AGENT_SECRET_ACCOUNT: &str = "admin-agent-signing-secret";

#[derive(Debug, Default, Clone, Copy)]
pub struct AdminSecretStore;

impl AdminSecretStore {
    pub fn new() -> Self {
        Self
    }

    pub fn get_agent_secret(&self) -> Result<Option<String>, DomainError> {
        let entry = admin_agent_secret_entry()?;
        match entry.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(DomainError::Database(error.to_string())),
        }
    }

    pub fn set_agent_secret(&self, secret: &str) -> Result<(), DomainError> {
        let secret = secret.trim();
        if secret.len() < 32 {
            return Err(DomainError::Validation(
                "admin agent secret must have at least 32 characters".into(),
            ));
        }

        admin_agent_secret_entry()?
            .set_password(secret)
            .map_err(|error| DomainError::Database(error.to_string()))
    }

    pub fn clear_agent_secret(&self) -> Result<(), DomainError> {
        let entry = admin_agent_secret_entry()?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(DomainError::Database(error.to_string())),
        }
    }
}

fn admin_agent_secret_entry() -> Result<keyring::Entry, DomainError> {
    keyring::Entry::new(KEYRING_SERVICE, ADMIN_AGENT_SECRET_ACCOUNT)
        .map_err(|error| DomainError::Database(error.to_string()))
}
