use std::path::{Path, PathBuf};

const WINDOWS_INVALID: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
pub(crate) const WINDOWS_RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9", "CLOCK$",
];

/// Validate a single file or folder name using the union of Windows and
/// POSIX constraints, so a blueprint behaves the same on every platform.
pub fn validate_file_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("file name is empty".into());
    }
    if name == "." || name == ".." {
        return Err(format!("invalid file name: {name}"));
    }
    if name.len() > 255 {
        return Err(format!("file name is too long: {name}"));
    }
    if name.ends_with('.') || name.ends_with(' ') {
        return Err(format!("file name cannot end with a dot or space: {name}"));
    }
    if name.chars().any(char::is_control) {
        return Err(format!("file name contains control characters: {name}"));
    }
    if name.chars().any(|ch| WINDOWS_INVALID.contains(&ch)) {
        return Err(format!("file name contains invalid characters: {name}"));
    }

    let stem = name
        .split('.')
        .next()
        .unwrap_or(name)
        .trim()
        .to_ascii_uppercase();
    if WINDOWS_RESERVED.contains(&stem.as_str()) {
        return Err(format!("file name is reserved on Windows: {name}"));
    }

    Ok(())
}

/// Validate a path relative to a blueprint root.
pub fn validate_relative_path(path: &str) -> Result<(), String> {
    let normalized = path.trim().replace('\\', "/");
    if normalized.starts_with('/') || Path::new(&normalized).is_absolute() {
        return Err(format!("path must be relative: {path}"));
    }
    let normalized = normalized.trim_matches('/');
    if normalized.is_empty() {
        return Err("relative path is empty".into());
    }
    if normalized.len() > 4096 {
        return Err("relative path is too long".into());
    }

    for component in normalized.split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            return Err(format!("relative path escapes the root: {path}"));
        }
        validate_file_name(component)?;
    }

    Ok(())
}

pub fn normalize_path(path: &Path) -> PathBuf {
    dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

pub fn is_path_under_root(path: &Path, root: &Path) -> bool {
    let path = normalize_path(path);
    let root = normalize_path(root);

    if path == root {
        return true;
    }

    let mut prefix = root.as_os_str().to_owned();
    let sep = std::path::MAIN_SEPARATOR;
    let root_str = root.to_string_lossy();
    if !root_str.ends_with(sep) {
        prefix.push(sep.to_string());
    }

    path.as_os_str()
        .to_string_lossy()
        .starts_with(prefix.as_os_str().to_string_lossy().as_ref())
}

pub fn is_path_authorized(path: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| is_path_under_root(path, root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("autoflow-path-{name}-{id}"))
    }

    #[test]
    fn rejects_prefix_collision() {
        let base = temp_dir("base");
        let sibling = temp_dir("sibling");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&sibling).unwrap();

        assert!(is_path_under_root(&base, &base));
        assert!(!is_path_under_root(&sibling, &base));

        let _ = fs::remove_dir_all(base);
        let _ = fs::remove_dir_all(sibling);
    }

    #[test]
    fn validates_file_names_and_reserved_windows_names() {
        assert!(validate_file_name("report.pdf").is_ok());
        assert!(validate_file_name("report..v2.pdf").is_ok());
        assert!(validate_file_name("CON").is_err());
        assert!(validate_file_name("con.txt").is_err());
        assert!(validate_file_name("bad:name").is_err());
        assert!(validate_file_name("trailing.").is_err());
        assert!(validate_file_name("trailing ").is_err());
    }

    #[test]
    fn validates_relative_paths_without_rejecting_double_dots_in_names() {
        assert!(validate_relative_path("docs/report..v2.pdf").is_ok());
        assert!(validate_relative_path("docs/../escape").is_err());
        assert!(validate_relative_path("/absolute").is_err());
        assert!(validate_relative_path("C:/windows").is_err());
        assert!(validate_relative_path("").is_err());
    }
}
