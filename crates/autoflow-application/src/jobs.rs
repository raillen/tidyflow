use std::path::Path;

use autoflow_domain::{
    collect_files, DomainError, FilterEngine, Job, JobSummary, SimulationReport, SimulationSample,
};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::ports::JobStore;

pub async fn list_jobs(store: &dyn JobStore) -> Result<Vec<JobSummary>, DomainError> {
    store.list().await
}

pub async fn get_job(store: &dyn JobStore, id: Uuid) -> Result<Job, DomainError> {
    store.get(id).await
}

pub async fn create_job(store: &dyn JobStore, mut job: Job) -> Result<Job, DomainError> {
    job.normalize();
    job.validate()?;
    store.save(&job).await?;
    Ok(job)
}

pub async fn update_job(store: &dyn JobStore, mut job: Job) -> Result<Job, DomainError> {
    job.normalize();
    job.validate()?;
    store.save(&job).await?;
    Ok(job)
}

pub async fn delete_job(store: &dyn JobStore, id: Uuid) -> Result<(), DomainError> {
    store.delete(id).await
}

pub async fn simulate_job(store: &dyn JobStore, id: Uuid) -> Result<SimulationReport, DomainError> {
    let mut job = store.get(id).await?;
    job.normalize();
    simulate_job_data(&job)
}

pub fn simulate_job_draft(mut job: Job) -> Result<SimulationReport, DomainError> {
    job.normalize();
    job.validate()?;
    simulate_job_data(&job)
}

pub fn simulate_job_data(job: &Job) -> Result<SimulationReport, DomainError> {
    let source = std::path::Path::new(&job.source_path);
    if !source.exists() {
        return Err(DomainError::SourceNotFound(job.source_path.clone()));
    }

    let engine = FilterEngine::from_job(job).map_err(|e| DomainError::Validation(e))?;
    let mut warnings = Vec::new();

    if let Some(regex) = &job.filters.name_regex {
        if regex::Regex::new(regex).is_err() {
            warnings.push(format!("invalid name_regex: {regex}"));
        }
    }
    if let Some(regex) = &job.filters.path_regex {
        if regex::Regex::new(regex).is_err() {
            warnings.push(format!("invalid path_regex: {regex}"));
        }
    }

    let (_, skipped) = count_files(source, job, &engine)?;
    let matched = collect_files(job).map_err(|e| DomainError::Database(e.to_string()))?;
    let mut sample = Vec::new();

    for file in matched.iter().take(25) {
        let relative = file
            .strip_prefix(source)
            .unwrap_or(file.as_path());
        let target = std::path::Path::new(&job.target_path).join(relative);
        sample.push(SimulationSample {
            source: file.display().to_string(),
            target: target.display().to_string(),
            action: format!("{:?}", job.mode).to_lowercase(),
        });
    }

    Ok(SimulationReport {
        files_matched: matched.len() as u32,
        files_skipped: skipped as u32,
        sample,
        warnings,
    })
}

fn count_files(
    source: &Path,
    job: &Job,
    engine: &FilterEngine,
) -> Result<(usize, usize), DomainError> {
    let mut total = 0usize;
    let mut skipped = 0usize;

    let mut visit = |path: &Path| {
        if !path.is_file() {
            return;
        }
        total += 1;
        if !engine
            .should_process(path, job, source)
            .unwrap_or(false)
        {
            skipped += 1;
        }
    };

    if source.is_file() {
        visit(source);
    } else if job.filters.recursive {
        for entry in WalkDir::new(source)
            .follow_links(job.filters.symlink_mode == autoflow_domain::SymlinkMode::Follow)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_symlink()
                && job.filters.symlink_mode == autoflow_domain::SymlinkMode::Skip
            {
                continue;
            }
            visit(entry.path());
        }
    } else {
        for entry in std::fs::read_dir(source).map_err(|e| DomainError::Database(e.to_string()))? {
            let entry = entry.map_err(|e| DomainError::Database(e.to_string()))?;
            visit(&entry.path());
        }
    }

    Ok((total, skipped))
}
