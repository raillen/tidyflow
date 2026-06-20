use autoflow_application::script_runner::run_pre_script;
use autoflow_domain::Job;
use tempfile::TempDir;

fn write_script(dir: &TempDir, name: &str, body: &str) {
    let scripts = dir.path().join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(scripts.join(name), body).expect("write script");
}

#[tokio::test]
async fn pre_script_success_returns_result() {
    let data_dir = TempDir::new().expect("tempdir");
    #[cfg(windows)]
    write_script(&data_dir, "ok.bat", "@echo off\r\nexit /b 0\r\n");
    #[cfg(not(windows))]
    {
        write_script(&data_dir, "ok.sh", "#!/bin/sh\nexit 0\n");
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            data_dir.path().join("scripts/ok.sh"),
            std::fs::Permissions::from_mode(0o755),
        )
        .expect("chmod");
    }

    let mut job = Job::new("script-test", "C:\\src", "D:\\dst");
    #[cfg(windows)]
    {
        job.scripts.pre_script = Some("ok.bat".into());
    }
    #[cfg(not(windows))]
    {
        job.scripts.pre_script = Some("ok.sh".into());
    }

    let result = run_pre_script(data_dir.path(), &job)
        .await
        .expect("pre-script should succeed");
    assert!(result.is_some());
    assert_eq!(result.unwrap().exit_code, Some(0));
}

#[tokio::test]
async fn pre_script_failure_aborts_job() {
    let data_dir = TempDir::new().expect("tempdir");
    #[cfg(windows)]
    write_script(&data_dir, "fail.bat", "@echo off\r\nexit /b 2\r\n");
    #[cfg(not(windows))]
    {
        write_script(&data_dir, "fail.sh", "#!/bin/sh\nexit 2\n");
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            data_dir.path().join("scripts/fail.sh"),
            std::fs::Permissions::from_mode(0o755),
        )
        .expect("chmod");
    }

    let mut job = Job::new("script-test", "C:\\src", "D:\\dst");
    #[cfg(windows)]
    {
        job.scripts.pre_script = Some("fail.bat".into());
    }
    #[cfg(not(windows))]
    {
        job.scripts.pre_script = Some("fail.sh".into());
    }

    match run_pre_script(data_dir.path(), &job).await {
        Err(error) => assert!(error.to_string().contains("pre-script failed")),
        Ok(_) => panic!("pre-script should fail"),
    }
}
