use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use autoflow_domain::DomainError;

pub fn hash_pin(pin: &str) -> Result<String, DomainError> {
    if pin.trim().is_empty() {
        return Err(DomainError::Validation("PIN cannot be empty".into()));
    }
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pin.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| DomainError::Database(error.to_string()))
}

pub fn verify_pin(pin: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .ok()
        .and_then(|parsed| {
            Argon2::default()
                .verify_password(pin.as_bytes(), &parsed)
                .ok()
        })
        .is_some()
}
