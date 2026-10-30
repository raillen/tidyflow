//! Encryption-at-rest for per-job notification secrets.
//!
//! Secrets are sealed with AES-256-GCM using a random 256-bit master key. The
//! master key lives in the OS keyring when available and falls back to a
//! `0600` file inside the application data directory on headless systems.
//!
//! Only secret fields are sealed; the rest of the job payload stays readable
//! so the database can still be inspected and queried.

use std::fs;
use std::path::Path;

use autoflow_domain::{DomainError, Job, NotifyChannel};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN};
use ring::rand::{SecureRandom, SystemRandom};

const KEYRING_SERVICE: &str = "autoflow";
const VAULT_KEY_ACCOUNT: &str = "vault-master-key";
const KEY_FILE_NAME: &str = "vault.key";
const SEALED_PREFIX: &str = "enc:v1:";

pub struct SecretVault {
    key: [u8; 32],
}

impl SecretVault {
    /// Loads the master key, creating one on first run.
    pub fn load(data_dir: &Path) -> Result<Self, DomainError> {
        let key_path = data_dir.join(KEY_FILE_NAME);

        // An explicit key file wins: it keeps portable and headless installs
        // deterministic even when a keyring happens to be available later.
        if key_path.exists() {
            let encoded = fs::read_to_string(&key_path)
                .map_err(|e| DomainError::Database(format!("failed to read vault key: {e}")))?;
            return Ok(Self {
                key: decode_key(encoded.trim())?,
            });
        }

        match keyring::Entry::new(KEYRING_SERVICE, VAULT_KEY_ACCOUNT) {
            Ok(entry) => match entry.get_password() {
                Ok(encoded) => {
                    return Ok(Self {
                        key: decode_key(&encoded)?,
                    })
                }
                Err(keyring::Error::NoEntry) => {
                    let key = random_key()?;
                    match entry.set_password(&encode_hex(&key)) {
                        Ok(()) => return Ok(Self { key }),
                        Err(error) => tracing::warn!(
                            %error,
                            "failed to store vault key in the keyring; using a key file"
                        ),
                    }
                }
                Err(error) => tracing::warn!(
                    %error,
                    "failed to read vault key from the keyring; using a key file"
                ),
            },
            Err(error) => tracing::warn!(
                %error,
                "OS keyring unavailable; using a vault key file"
            ),
        }

        let key = random_key()?;
        write_key_file(&key_path, &encode_hex(&key))?;
        Ok(Self { key })
    }

    /// Builds a vault from an explicit key. Useful for tests and tooling.
    pub fn from_key(key: [u8; 32]) -> Self {
        Self { key }
    }

    fn sealing_key(&self) -> Result<LessSafeKey, DomainError> {
        let unbound = UnboundKey::new(&AES_256_GCM, &self.key)
            .map_err(|_| DomainError::Database("invalid vault key".into()))?;
        Ok(LessSafeKey::new(unbound))
    }

    /// Seals a value, leaving empty and already-sealed values untouched.
    pub fn seal_field(&self, value: &str) -> String {
        if value.is_empty() || value.starts_with(SEALED_PREFIX) {
            return value.to_string();
        }
        match self.seal(value) {
            Ok(sealed) => format!("{SEALED_PREFIX}{}", encode_hex(&sealed)),
            Err(error) => {
                tracing::warn!(%error, "failed to seal secret; storing it as plain text");
                value.to_string()
            }
        }
    }

    /// Opens a sealed value. Plain values are returned unchanged.
    pub fn open_field(&self, value: &str) -> String {
        let Some(encoded) = value.strip_prefix(SEALED_PREFIX) else {
            return value.to_string();
        };
        let Ok(bytes) = decode_hex(encoded) else {
            tracing::warn!("sealed secret is not valid hex; dropping it");
            return String::new();
        };
        match self.open(&bytes) {
            Ok(plain) => plain,
            Err(error) => {
                tracing::warn!(%error, "failed to open sealed secret; dropping it");
                String::new()
            }
        }
    }

    fn seal(&self, plaintext: &str) -> Result<Vec<u8>, DomainError> {
        let key = self.sealing_key()?;
        let mut nonce_bytes = [0u8; NONCE_LEN];
        SystemRandom::new()
            .fill(&mut nonce_bytes)
            .map_err(|_| DomainError::Database("failed to generate nonce".into()))?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = plaintext.as_bytes().to_vec();
        key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| DomainError::Database("failed to encrypt secret".into()))?;

        let mut sealed = Vec::with_capacity(NONCE_LEN + in_out.len());
        sealed.extend_from_slice(&nonce_bytes);
        sealed.extend_from_slice(&in_out);
        Ok(sealed)
    }

    fn open(&self, sealed: &[u8]) -> Result<String, DomainError> {
        if sealed.len() <= NONCE_LEN {
            return Err(DomainError::Database("sealed secret is truncated".into()));
        }
        let (nonce_bytes, ciphertext) = sealed.split_at(NONCE_LEN);
        let nonce = Nonce::try_assume_unique_for_key(nonce_bytes)
            .map_err(|_| DomainError::Database("invalid secret nonce".into()))?;
        let key = self.sealing_key()?;
        let mut in_out = ciphertext.to_vec();
        let plaintext = key
            .open_in_place(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| DomainError::Database("failed to decrypt secret".into()))?;
        String::from_utf8(plaintext.to_vec())
            .map_err(|_| DomainError::Database("decrypted secret is not UTF-8".into()))
    }
}

/// Seals every secret carried by a job before it is persisted.
pub fn seal_job_secrets(job: &mut Job, vault: &SecretVault) {
    for channel in &mut job.notify.channels {
        match channel {
            NotifyChannel::Generic { url, headers } => {
                *url = vault.seal_field(url);
                for value in headers.values_mut() {
                    *value = vault.seal_field(value);
                }
            }
            NotifyChannel::Discord { webhook_url } => {
                *webhook_url = vault.seal_field(webhook_url);
            }
            NotifyChannel::Telegram {
                bot_token, chat_id, ..
            } => {
                *bot_token = vault.seal_field(bot_token);
                *chat_id = vault.seal_field(chat_id);
            }
        }
    }
    if let Some(password) = job.options.encrypt_password.as_mut() {
        *password = vault.seal_field(password);
    }
}

/// Opens every sealed secret carried by a job after it is loaded.
pub fn open_job_secrets(job: &mut Job, vault: &SecretVault) {
    for channel in &mut job.notify.channels {
        match channel {
            NotifyChannel::Generic { url, headers } => {
                *url = vault.open_field(url);
                for value in headers.values_mut() {
                    *value = vault.open_field(value);
                }
            }
            NotifyChannel::Discord { webhook_url } => {
                *webhook_url = vault.open_field(webhook_url);
            }
            NotifyChannel::Telegram {
                bot_token, chat_id, ..
            } => {
                *bot_token = vault.open_field(bot_token);
                *chat_id = vault.open_field(chat_id);
            }
        }
    }
    if let Some(password) = job.options.encrypt_password.as_mut() {
        *password = vault.open_field(password);
    }
}

fn random_key() -> Result<[u8; 32], DomainError> {
    let mut key = [0u8; 32];
    SystemRandom::new()
        .fill(&mut key)
        .map_err(|_| DomainError::Database("failed to generate vault key".into()))?;
    Ok(key)
}

fn decode_key(encoded: &str) -> Result<[u8; 32], DomainError> {
    let bytes = decode_hex(encoded)?;
    bytes
        .try_into()
        .map_err(|_| DomainError::Database("vault key must be 32 bytes".into()))
}

#[cfg(unix)]
fn write_key_file(path: &Path, contents: &str) -> Result<(), DomainError> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|e| DomainError::Database(format!("failed to create vault key: {e}")))?;
    file.write_all(contents.as_bytes())
        .map_err(|e| DomainError::Database(format!("failed to write vault key: {e}")))
}

#[cfg(not(unix))]
fn write_key_file(path: &Path, contents: &str) -> Result<(), DomainError> {
    fs::write(path, contents)
        .map_err(|e| DomainError::Database(format!("failed to write vault key: {e}")))
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn decode_hex(value: &str) -> Result<Vec<u8>, DomainError> {
    if !value.len().is_multiple_of(2) {
        return Err(DomainError::Database("invalid hex value".into()));
    }
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let (chunks, _) = bytes.as_chunks::<2>();
    for chunk in chunks {
        let high = hex_digit(chunk[0])?;
        let low = hex_digit(chunk[1])?;
        out.push((high << 4) | low);
    }
    Ok(out)
}

fn hex_digit(byte: u8) -> Result<u8, DomainError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(DomainError::Database("invalid hex digit".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autoflow_domain::{Job, NotifyChannel, NotifyConfig};
    use std::collections::HashMap;

    fn vault() -> SecretVault {
        SecretVault::from_key([7u8; 32])
    }

    #[test]
    fn round_trips_values() {
        let vault = vault();
        let sealed = vault.seal_field("https://discord.com/api/webhooks/123/abc");
        assert!(sealed.starts_with(SEALED_PREFIX));
        assert_ne!(sealed, "https://discord.com/api/webhooks/123/abc");
        assert_eq!(
            vault.open_field(&sealed),
            "https://discord.com/api/webhooks/123/abc"
        );
    }

    #[test]
    fn leaves_plain_and_empty_values_alone() {
        let vault = vault();
        assert_eq!(vault.open_field("plain"), "plain");
        assert_eq!(vault.seal_field(""), "");
        assert_eq!(vault.open_field(""), "");
    }

    #[test]
    fn rejects_tampered_ciphertext() {
        let vault = vault();
        let sealed = vault.seal_field("secret");
        let mut tampered = sealed.clone();
        tampered.push('0');
        assert_eq!(vault.open_field(&tampered), "");
    }

    #[test]
    fn seals_and_opens_job_secrets() {
        let vault = vault();
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());
        let mut job = Job::new("notify", "/src", "/dst");
        job.notify = NotifyConfig {
            enabled: true,
            events: vec![],
            channels: vec![
                NotifyChannel::Generic {
                    url: "https://example.com/hook".into(),
                    headers,
                },
                NotifyChannel::Discord {
                    webhook_url: "https://discord.com/api/webhooks/1/x".into(),
                },
                NotifyChannel::Telegram {
                    bot_token: "123:abc".into(),
                    chat_id: "42".into(),
                    remember_token: false,
                },
            ],
        };
        job.options.encrypt_password = Some("pack-pass".into());

        seal_job_secrets(&mut job, &vault);

        let json = serde_json::to_string(&job).unwrap();
        assert!(!json.contains("https://example.com/hook"));
        assert!(!json.contains("Bearer token"));
        assert!(!json.contains("123:abc"));
        assert!(!json.contains("pack-pass"));

        open_job_secrets(&mut job, &vault);
        match &job.notify.channels[0] {
            NotifyChannel::Generic { url, headers } => {
                assert_eq!(url, "https://example.com/hook");
                assert_eq!(headers["Authorization"], "Bearer token");
            }
            _ => panic!("unexpected channel"),
        }
        assert_eq!(job.options.encrypt_password.as_deref(), Some("pack-pass"));
    }

    #[test]
    fn loads_an_existing_key_file() {
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join(KEY_FILE_NAME);
        fs::write(&key_path, encode_hex(&[9u8; 32])).unwrap();

        let vault = SecretVault::load(dir.path()).unwrap();
        let sealed = vault.seal_field("hello");

        let reopened = SecretVault::load(dir.path()).unwrap();
        assert_eq!(reopened.open_field(&sealed), "hello");
    }

    #[cfg(unix)]
    #[test]
    fn key_file_is_private() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join(KEY_FILE_NAME);
        write_key_file(&key_path, &encode_hex(&[1u8; 32])).unwrap();
        let mode = fs::metadata(&key_path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
