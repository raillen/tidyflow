use autoflow_application::pack::{create_encrypted_package, remove_files_in_dir, resolve_pack_path};
use tempfile::TempDir;

#[test]
fn creates_encrypted_package_with_files() {
    let workspace = TempDir::new().expect("tempdir");
    let source = workspace.path().join("target");
    std::fs::create_dir_all(&source).expect("mkdir");
    std::fs::write(source.join("note.txt"), b"autoflow-pack-test").expect("write");

    let output = source.join("bundle.autoflow.zip");
    let created =
        create_encrypted_package(&source, &output, "test-password-123").expect("pack");
    assert_eq!(created, output);
    assert!(output.is_file());
    assert!(std::fs::metadata(&output).expect("meta").len() > 0);
}

#[test]
fn resolve_pack_path_uses_default_name() {
    let dir = TempDir::new().expect("tempdir");
    let path = resolve_pack_path(dir.path(), None);
    assert!(path.ends_with("output.autoflow.zip"));
}

#[test]
fn remove_files_in_dir_keeps_root() {
    let workspace = TempDir::new().expect("tempdir");
    let root = workspace.path().join("target");
    std::fs::create_dir_all(root.join("nested")).expect("mkdir");
    std::fs::write(root.join("nested/a.txt"), b"x").expect("write");

    remove_files_in_dir(&root).expect("remove");
    assert!(root.is_dir());
    assert!(root.read_dir().unwrap().next().is_none());
}
