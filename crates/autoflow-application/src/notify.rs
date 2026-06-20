use autoflow_domain::{Job, NotifyChannel, NotifyConfig, NotifyEvent};
use reqwest::Client;
use serde_json::json;

pub fn fire_notify_event(job: &Job, event: NotifyEvent, detail: Option<&str>) {
    if !job.notify.enabled {
        return;
    }
    if !job.notify.events.contains(&event) {
        return;
    }

    let job = job.clone();
    let detail = detail.map(str::to_owned);
    tokio::spawn(async move {
        if let Err(error) =
            send_notifications(&job.notify, &job, event, detail.as_deref()).await
        {
            tracing::warn!(job_id = %job.id, ?event, %error, "notification delivery failed");
        }
    });
}

async fn send_notifications(
    config: &NotifyConfig,
    job: &Job,
    event: NotifyEvent,
    detail: Option<&str>,
) -> Result<(), String> {
    let client = Client::new();
    for channel in &config.channels {
        if let Err(error) = send_channel(&client, channel, job, event, detail).await {
            tracing::warn!(?channel, %error, "channel notification failed");
        }
    }
    Ok(())
}

async fn send_channel(
    client: &Client,
    channel: &NotifyChannel,
    job: &Job,
    event: NotifyEvent,
    detail: Option<&str>,
) -> Result<(), String> {
    let message = format_message(job, event, detail);
    match channel {
        NotifyChannel::Discord { webhook_url } => {
            client
                .post(webhook_url)
                .json(&json!({ "content": message }))
                .send()
                .await
                .map_err(|e| e.to_string())?;
        }
        NotifyChannel::Telegram {
            bot_token,
            chat_id,
            ..
        } => {
            let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
            client
                .post(&url)
                .json(&json!({
                    "chat_id": chat_id,
                    "text": message,
                    "disable_web_page_preview": true
                }))
                .send()
                .await
                .map_err(|e| e.to_string())?;
        }
        NotifyChannel::Generic { url, headers } => {
            let mut request = client.post(url).json(&json!({
                "event": event_label(event),
                "jobId": job.id,
                "jobName": job.name,
                "message": message,
                "detail": detail
            }));
            for (key, value) in headers {
                request = request.header(key, value);
            }
            request.send().await.map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn format_message(job: &Job, event: NotifyEvent, detail: Option<&str>) -> String {
    let mut message = format!(
        "AutoFlow [{}] {} — {}",
        event_label(event),
        job.name,
        job.id
    );
    if let Some(detail) = detail.filter(|d| !d.is_empty()) {
        message.push_str("\n");
        message.push_str(detail);
    }
    message
}

fn event_label(event: NotifyEvent) -> &'static str {
    match event {
        NotifyEvent::Started => "started",
        NotifyEvent::Completed => "completed",
        NotifyEvent::Failed => "failed",
    }
}
