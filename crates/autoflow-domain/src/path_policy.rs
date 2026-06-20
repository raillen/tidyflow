use std::path::{Path, PathBuf};

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
}
