use std::collections::BTreeMap;
use std::fs;
use tempfile::TempDir;

use autoflow_application::naming::{
    apply_children_structure, folder_name_matches_pattern, resolve_correct_folder_name,
};
use autoflow_domain::{
    FolderAction, FolderActionNode, FolderPlan, FolderPlanVariable, NamingPattern,
    TemplatePipeline, TemplateSegment,
};
use autoflow_infrastructure::audit::SqliteAuditStore;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

async fn create_test_audit(_temp_dir: &std::path::Path) -> (SqliteAuditStore, SqlitePool) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create in-memory sqlite pool");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS audit_entries (
            id TEXT PRIMARY KEY,
            job_id TEXT,
            blueprint_id TEXT,
            job_name TEXT NOT NULL,
            source_path TEXT NOT NULL,
            target_path TEXT NOT NULL,
            status TEXT NOT NULL,
            file_size INTEGER NOT NULL DEFAULT 0,
            duration_ms REAL NOT NULL DEFAULT 0,
            details TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&pool)
    .await
    .expect("create audit table");
    (SqliteAuditStore::new(pool.clone()), pool)
}

fn make_folder_plan_with_root_name(root_name: &str) -> FolderPlan {
    FolderPlan {
        version: 1,
        variables: vec![],
        allow_delete_non_empty: false,
        confirm_delete_non_empty: false,
        nodes: vec![FolderActionNode {
            action: FolderAction::Create,
            name: root_name.to_string(),
            source_path: String::new(),
            target_path: String::new(),
            children: vec![
                FolderActionNode {
                    action: FolderAction::Create,
                    name: "src".to_string(),
                    source_path: String::new(),
                    target_path: String::new(),
                    children: vec![],
                },
                FolderActionNode {
                    action: FolderAction::Create,
                    name: "docs".to_string(),
                    source_path: String::new(),
                    target_path: String::new(),
                    children: vec![],
                },
            ],
        }],
    }
}

fn make_folder_plan_with_variable() -> FolderPlan {
    FolderPlan {
        version: 1,
        variables: vec![FolderPlanVariable {
            name: "projectName".to_string(),
            label: "Project Name".to_string(),
            default_value: "default-project".to_string(),
            required: true,
        }],
        allow_delete_non_empty: false,
        confirm_delete_non_empty: false,
        nodes: vec![FolderActionNode {
            action: FolderAction::Create,
            name: "{projectName}".to_string(),
            source_path: String::new(),
            target_path: String::new(),
            children: vec![FolderActionNode {
                action: FolderAction::Create,
                name: "src".to_string(),
                source_path: String::new(),
                target_path: String::new(),
                children: vec![],
            }],
        }],
    }
}

#[test]
fn matches_static_root_name() {
    let plan = make_folder_plan_with_root_name("my-project");
    let variables = BTreeMap::new();

    assert!(folder_name_matches_pattern("my-project", None, &plan, &variables).unwrap());
    assert!(!folder_name_matches_pattern("other-name", None, &plan, &variables).unwrap());
}

#[test]
fn matches_with_regex_pattern() {
    let plan = make_folder_plan_with_root_name("any-name");
    let np = NamingPattern {
        regex: Some("^\\d{4}-.*".to_string()),
        template_override: None,
    };
    let variables = BTreeMap::new();

    assert!(folder_name_matches_pattern("2024-my-project", Some(&np), &plan, &variables).unwrap());
    assert!(!folder_name_matches_pattern("my-project", Some(&np), &plan, &variables).unwrap());
}

#[test]
fn matches_with_template_override() {
    let plan = make_folder_plan_with_root_name("ignored");
    let tpl = TemplatePipeline {
        segments: vec![TemplateSegment::Literal {
            value: "override-name".to_string(),
        }],
    };
    let np = NamingPattern {
        regex: None,
        template_override: Some(tpl),
    };
    let variables = BTreeMap::new();

    assert!(folder_name_matches_pattern("override-name", Some(&np), &plan, &variables).unwrap());
    assert!(!folder_name_matches_pattern("other", Some(&np), &plan, &variables).unwrap());
}

#[test]
fn matches_with_variable_template() {
    let plan = make_folder_plan_with_variable();
    let mut variables = BTreeMap::new();
    variables.insert("projectName".to_string(), "alpha".to_string());

    assert!(folder_name_matches_pattern("alpha", None, &plan, &variables).unwrap());
    assert!(!folder_name_matches_pattern("beta", None, &plan, &variables).unwrap());
}

#[test]
fn no_pattern_matches_anything() {
    let plan = FolderPlan {
        version: 1,
        variables: vec![],
        allow_delete_non_empty: false,
        confirm_delete_non_empty: false,
        nodes: vec![],
    };
    let variables = BTreeMap::new();

    assert!(folder_name_matches_pattern("anything", None, &plan, &variables).unwrap());
}

#[test]
fn resolve_name_from_root_node() {
    let plan = make_folder_plan_with_root_name("expected-name");
    let variables = BTreeMap::new();

    let resolved =
        resolve_correct_folder_name("wrong-name", None, &plan, None, &variables).unwrap();
    assert_eq!(resolved, "expected-name");
}

#[test]
fn resolve_name_from_template_override() {
    let plan = make_folder_plan_with_root_name("ignored");
    let tpl = TemplatePipeline {
        segments: vec![TemplateSegment::Literal {
            value: "override-result".to_string(),
        }],
    };
    let np = NamingPattern {
        regex: None,
        template_override: Some(tpl),
    };
    let variables = BTreeMap::new();

    let resolved =
        resolve_correct_folder_name("wrong", Some(&np), &plan, None, &variables).unwrap();
    assert_eq!(resolved, "override-result");
}

#[test]
fn resolve_name_with_variable() {
    let plan = make_folder_plan_with_variable();
    let mut variables = BTreeMap::new();
    variables.insert("projectName".to_string(), "gamma".to_string());

    let resolved = resolve_correct_folder_name("wrong", None, &plan, None, &variables).unwrap();
    assert_eq!(resolved, "gamma");
}

#[test]
fn resolve_fallback_to_original() {
    let plan = FolderPlan {
        version: 1,
        variables: vec![],
        allow_delete_non_empty: false,
        confirm_delete_non_empty: false,
        nodes: vec![],
    };
    let variables = BTreeMap::new();

    let resolved = resolve_correct_folder_name("keep-me", None, &plan, None, &variables).unwrap();
    assert_eq!(resolved, "keep-me");
}

#[test]
fn apply_children_creates_subfolders() {
    let workspace = TempDir::new().expect("tempdir");
    let root = workspace.path().join("my-project");
    fs::create_dir_all(&root).expect("mkdir");

    let plan = make_folder_plan_with_root_name("my-project");
    let variables = BTreeMap::new();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let (audit, _pool) = create_test_audit(workspace.path()).await;

        let blueprint = autoflow_domain::Blueprint {
            id: uuid::Uuid::new_v4(),
            name: "test".to_string(),
            kind: autoflow_domain::BlueprintKind::Folder,
            root_path: root.to_string_lossy().to_string(),
            search: autoflow_domain::FileFilter::default(),
            routing: autoflow_domain::RoutingConfig::default(),
            operation: autoflow_domain::BlueprintOperation::Move,
            recursive: true,
            conflict: autoflow_domain::ConflictStrategy::Skip,
            rename_template: None,
            folder_plan: Some(plan),
            watch: None,
            counter: autoflow_domain::CounterConfig::default(),
            enabled: true,
            last_run: None,
        };

        let children = blueprint
            .folder_plan
            .as_ref()
            .unwrap()
            .nodes
            .first()
            .unwrap()
            .children
            .as_slice();
        let result = apply_children_structure(&audit, &blueprint, &root, children, &variables)
            .await
            .unwrap();
        assert_eq!(result.processed, 2);
        assert_eq!(result.failed, 0);

        assert!(root.join("src").is_dir());
        assert!(root.join("docs").is_dir());
    });
}

#[test]
fn apply_children_skips_existing_subfolders() {
    let workspace = TempDir::new().expect("tempdir");
    let root = workspace.path().join("my-project");
    fs::create_dir_all(root.join("src")).expect("mkdir src");
    fs::write(root.join("src/existing.txt"), b"keep me").expect("write");

    let plan = make_folder_plan_with_root_name("my-project");
    let variables = BTreeMap::new();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let (audit, _pool) = create_test_audit(workspace.path()).await;

        let blueprint = autoflow_domain::Blueprint {
            id: uuid::Uuid::new_v4(),
            name: "test".to_string(),
            kind: autoflow_domain::BlueprintKind::Folder,
            root_path: root.to_string_lossy().to_string(),
            search: autoflow_domain::FileFilter::default(),
            routing: autoflow_domain::RoutingConfig::default(),
            operation: autoflow_domain::BlueprintOperation::Move,
            recursive: true,
            conflict: autoflow_domain::ConflictStrategy::Skip,
            rename_template: None,
            folder_plan: Some(plan),
            watch: None,
            counter: autoflow_domain::CounterConfig::default(),
            enabled: true,
            last_run: None,
        };

        let children = blueprint
            .folder_plan
            .as_ref()
            .unwrap()
            .nodes
            .first()
            .unwrap()
            .children
            .as_slice();
        let result = apply_children_structure(&audit, &blueprint, &root, children, &variables)
            .await
            .unwrap();
        assert_eq!(result.processed, 1);
        assert_eq!(result.skipped, 1);

        assert!(root.join("src/existing.txt").exists());
        assert!(root.join("docs").is_dir());
    });
}
