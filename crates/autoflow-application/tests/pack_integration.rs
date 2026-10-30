use autoflow_application::pack::{
    create_encrypted_package, remove_files_in_dir, remove_files_in_dir_except, resolve_pack_path,
};
use tempfile::TempDir;

#[test]
fn creates_encrypted_package_with_files() {
    let workspace = TempDir::new().expect("tempdir");
    let source = workspace.path().join("target");
    std::fs::create_dir_all(&source).expect("mkdir");
    std::fs::write(source.join("note.txt"), b"autoflow-pack-test").expect("write");

    let output = source.join("bundle.autoflow.zip");
    let created = create_encrypted_package(&source, &output, "test-password-123").expect("pack");
    assert_eq!(created, output);
    assert!(output.is_file());
    assert!(std::fs::metadata(&output).expect("meta").len() > 0);

    let file = std::fs::File::open(&output).expect("zip");
    let mut archive = zip::ZipArchive::new(file).expect("archive");
    let names: Vec<String> = (0..archive.len())
        .map(|index| {
            archive
                .by_index_decrypt(index, b"test-password-123")
                .unwrap()
                .name()
                .to_string()
        })
        .collect();
    assert_eq!(names, vec!["note.txt"]);
}

#[test]
fn remove_files_in_dir_except_preserves_excluded_file() {
    let workspace = TempDir::new().expect("tempdir");
    let root = workspace.path().join("target");
    std::fs::create_dir_all(&root).expect("mkdir");
    std::fs::write(root.join("a.txt"), b"a").expect("write");
    let excluded = root.join("bundle.zip");
    std::fs::write(&excluded, b"bundle").expect("bundle");

    remove_files_in_dir_except(&root, Some(&excluded)).expect("remove");

    assert!(root.is_dir());
    assert!(excluded.is_file());
    assert!(!root.join("a.txt").exists());
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
