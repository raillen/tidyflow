use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use autoflow_domain::DomainError;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{AesMode, CompressionMethod, ZipWriter};

pub fn create_encrypted_package(
    target_dir: &Path,
    output_path: &Path,
    password: &str,
) -> Result<PathBuf, DomainError> {
    if password.is_empty() {
        return Err(DomainError::Validation(
            "encrypt password is required".into(),
        ));
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| DomainError::Database(e.to_string()))?;
    }

    let file = File::create(output_path).map_err(|e| DomainError::Database(e.to_string()))?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .with_aes_encryption(AesMode::Aes256, password);

    for entry in WalkDir::new(target_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let relative = path
            .strip_prefix(target_dir)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let name = relative
            .to_string_lossy()
            .replace('\\', "/");

        writer
            .start_file(name, options)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let mut source = File::open(path).map_err(|e| DomainError::Database(e.to_string()))?;
        let mut buffer = Vec::new();
        source
            .read_to_end(&mut buffer)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        writer
            .write_all(&buffer)
            .map_err(|e| DomainError::Database(e.to_string()))?;
    }

    writer
        .finish()
        .map_err(|e| DomainError::Database(e.to_string()))?;

    Ok(output_path.to_path_buf())
}

pub fn resolve_pack_path(target_dir: &Path, pack_filename: Option<&str>) -> PathBuf {
    let name = pack_filename
        .filter(|n| !n.trim().is_empty())
        .map(|n| n.to_string())
        .unwrap_or_else(|| "output.autoflow.zip".into());
    target_dir.join(name)
}

pub fn remove_files_in_dir(dir: &Path) -> Result<(), DomainError> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in WalkDir::new(dir)
        .contents_first(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path == dir {
            continue;
        }
        if entry.file_type().is_dir() {
            fs::remove_dir(path).map_err(|e| DomainError::Database(e.to_string()))?;
        } else {
            fs::remove_file(path).map_err(|e| DomainError::Database(e.to_string()))?;
        }
    }
    Ok(())
}

pub fn resolve_encrypt_password(
    job_id: uuid::Uuid,
    password: Option<&str>,
    remember: bool,
) -> Result<String, DomainError> {
    if let Some(password) = password.filter(|p| !p.is_empty()) {
        if remember {
            let _ = store_encrypt_password(job_id, password);
        }
        return Ok(password.to_string());
    }

    if remember {
        if let Ok(entry) = keyring::Entry::new("autoflow", &format!("encrypt-{job_id}")) {
            if let Ok(stored) = entry.get_password() {
                return Ok(stored);
            }
        }
    }

    Err(DomainError::Validation(
        "encrypt password is required for encrypted output".into(),
    ))
}

fn store_encrypt_password(job_id: uuid::Uuid, password: &str) -> Result<(), DomainError> {
    keyring::Entry::new("autoflow", &format!("encrypt-{job_id}"))
        .and_then(|entry| entry.set_password(password))
        .map_err(|e| DomainError::Database(e.to_string()))
}
