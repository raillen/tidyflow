use std::sync::Arc;
use std::time::Duration;

use autoflow_application::ports::JobStore;
use autoflow_domain::DomainError;
use autoflow_infrastructure::{jobs::SqliteJobStore, ui_state::SqliteMissedScheduleStore};
use chrono::{Duration as ChronoDuration, Utc};
use tokio::time;

use crate::queue::JobQueue;

const MISSED_GRACE_SECS: i64 = 90;

pub struct Scheduler;

impl Scheduler {
    pub fn start(
        jobs: Arc<SqliteJobStore>,
        missed: Arc<SqliteMissedScheduleStore>,
        queue: JobQueue,
    ) {
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));
            interval.tick().await;
            loop {
                interval.tick().await;
                if let Err(error) = tick(&jobs, &missed, &queue).await {
                    tracing::warn!(%error, "scheduler tick failed");
                }
            }
        });
    }
}

async fn tick(
    jobs: &SqliteJobStore,
    missed: &SqliteMissedScheduleStore,
    queue: &JobQueue,
) -> Result<(), DomainError> {
    let summaries = jobs.list().await?;
    let now = Utc::now();

    for summary in summaries {
        if !summary.enabled || !summary.schedule_enabled {
            continue;
        }

        let Some(next_run) = summary.next_run else {
            continue;
        };
        if next_run > now {
            continue;
        }

        if now - next_run > ChronoDuration::seconds(MISSED_GRACE_SECS) {
            missed.record(summary.id, &summary.name, next_run).await?;
        }

        if let Err(error) = queue.enqueue(summary.id) {
            if !matches!(error, DomainError::AlreadyRunning) {
                tracing::warn!(job_id = %summary.id, %error, "failed to enqueue scheduled job");
            }
        }
    }

    Ok(())
}
