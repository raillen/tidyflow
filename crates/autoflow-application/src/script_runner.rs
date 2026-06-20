use std::path::{Path, PathBuf};
use std::process::Stdio;

use autoflow_domain::{DomainError, Job};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub enum ScriptPhase {
    Pre,
    Post,
}

impl ScriptPhase {
    fn label(&self) -> &'static str {
        match self {
            ScriptPhase::Pre => "pre",
            ScriptPhase::Post => "post",
        }
    }
}

pub struct ScriptResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

pub fn resolve_script_path(data_dir: &Path, name: &str) -> PathBuf {
    data_dir.join("scripts").join(name)
}

pub async fn run_script(
    data_dir: &Path,
    job: &Job,
    phase: ScriptPhase,
    script_name: &str,
) -> Result<ScriptResult, DomainError> {
    let script_path = resolve_script_path(data_dir, script_name);
    if !script_path.exists() {
        return Err(DomainError::Validation(format!(
            "{} script not found: {}",
            phase.label(),
            script_path.display()
        )));
    }

    let timeout_secs = job.scripts.timeout_secs;
    let mut command = build_command(&script_path, job);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let duration = Duration::from_secs(timeout_secs as u64);
    let child = command
        .spawn()
        .map_err(|e| DomainError::Database(format!("failed to spawn {} script: {e}", phase.label())))?;

    match timeout(duration, child.wait_with_output()).await {
        Ok(Ok(output)) => Ok(ScriptResult {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            timed_out: false,
        }),
        Ok(Err(e)) => Err(DomainError::Database(format!(
            "{} script failed: {e}",
            phase.label()
        ))),
        Err(_) => Ok(ScriptResult {
            exit_code: None,
            stdout: String::new(),
            stderr: format!("script timed out after {timeout_secs}s"),
            timed_out: true,
        }),
    }
}

pub async fn run_pre_script(
    data_dir: &Path,
    job: &Job,
) -> Result<Option<ScriptResult>, DomainError> {
    let Some(name) = job.scripts.pre_script.as_deref() else {
        return Ok(None);
    };
    let result = run_script(data_dir, job, ScriptPhase::Pre, name).await?;
    if result.timed_out || result.exit_code.unwrap_or(1) != 0 {
        let detail = if result.timed_out {
            result.stderr.clone()
        } else {
            format!(
                "exit code {:?}: {}",
                result.exit_code,
                result.stderr.trim()
            )
        };
        return Err(DomainError::Validation(format!("pre-script failed: {detail}")));
    }
    Ok(Some(result))
}

pub async fn run_post_script(
    data_dir: &Path,
    job: &Job,
) -> Result<Option<ScriptResult>, DomainError> {
    let Some(name) = job.scripts.post_script.as_deref() else {
        return Ok(None);
    };
    run_script(data_dir, job, ScriptPhase::Post, name)
        .await
        .map(Some)
}

fn build_command(script_path: &Path, job: &Job) -> Command {
    let mut command = launch_script(script_path);
    command.env("AUTOFLOW_JOB_ID", job.id.to_string());
    command.env("AUTOFLOW_JOB_NAME", &job.name);
    command.env("AUTOFLOW_SOURCE", &job.source_path);
    command.env("AUTOFLOW_TARGET", &job.target_path);
    command
}

fn launch_script(script_path: &Path) -> Command {
    #[cfg(windows)]
    {
        let ext = script_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if ext == "bat" || ext == "cmd" {
            let mut command = Command::new("cmd");
            command.args(["/C", &script_path.to_string_lossy()]);
            return command;
        }
        if ext == "ps1" {
            let mut command = Command::new("powershell");
            command.args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                &script_path.to_string_lossy(),
            ]);
            return command;
        }
    }

    Command::new(script_path)
}
