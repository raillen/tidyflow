mod commands;

use std::sync::Arc;
use std::time::Duration;

use autoflow_core::{AppState, ExecutionEvent};
use autoflow_infrastructure::ui_state::MissedScheduleEntry;
use tauri::{Emitter, Manager, WebviewWindow};
use tauri_plugin_notification::NotificationExt;

async fn notify_missed_schedules(window: &WebviewWindow, state: &AppState) {
    let entries: Vec<MissedScheduleEntry> = match state.list_missed_schedules().await {
        Ok(entries) => entries,
        Err(error) => {
            tracing::warn!(%error, "failed to load missed schedules");
            return;
        }
    };

    if entries.is_empty() {
        return;
    }

    let summary = if entries.len() == 1 {
        format!(
            "O fluxo \"{}\" perdeu a execução agendada.",
            entries[0].job_name
        )
    } else {
        format!("{} fluxos perderam execuções agendadas.", entries.len())
    };

    if let Err(error) = window
        .app_handle()
        .notification()
        .builder()
        .title("AutoFlow — agendamentos perdidos")
        .body(&summary)
        .show()
    {
        tracing::warn!(%error, "failed to show missed schedule notification");
    }
}

fn wire_missed_schedule_notifications(app: &tauri::App, state: AppState) {
    let handle = app.handle().clone();
    let state_for_startup = state.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        if let Some(window) = handle
            .get_webview_window("main")
            .or_else(|| handle.webview_windows().values().next().cloned())
        {
            notify_missed_schedules(&window, &state_for_startup).await;
        }
    });

    if let Some(window) = app
        .get_webview_window("main")
        .or_else(|| app.webview_windows().values().next().cloned())
    {
        let window_for_focus = window.clone();
        let state_for_focus = state;
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(true) = event {
                let window = window_for_focus.clone();
                let state = state_for_focus.clone();
                tauri::async_runtime::spawn(async move {
                    notify_missed_schedules(&window, &state).await;
                });
            }
        });
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");

            let events = Arc::new(move |event: ExecutionEvent| match event {
                ExecutionEvent::Progress(progress) => {
                    let _ = handle.emit("execution_progress", progress);
                }
                ExecutionEvent::Completed(completed) => {
                    let _ = handle.emit("execution_completed", completed);
                }
            });

            let state =
                tauri::async_runtime::block_on(async { AppState::new(data_dir, events).await })
                    .expect("failed to initialize AutoFlow application state");

            wire_missed_schedule_notifications(app, state.clone());
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health,
            commands::settings_get,
            commands::settings_update,
            commands::admin_fleet_snapshot,
            commands::admin_heartbeat_payload,
            commands::admin_signed_heartbeat_payload,
            commands::admin_dispatch_command,
            commands::admin_enqueue_command,
            commands::admin_list_commands,
            commands::admin_command_queue_summary,
            commands::admin_process_next_command,
            commands::jobs_list,
            commands::jobs_get,
            commands::jobs_create,
            commands::jobs_update,
            commands::jobs_delete,
            commands::jobs_run,
            commands::jobs_simulate,
            commands::jobs_simulate_draft,
            commands::executions_list_active,
            commands::executions_cancel,
            commands::audit_list_recent,
            commands::audit_query,
            commands::audit_export,
            commands::ui_state_get,
            commands::ui_state_save,
            commands::jobs_list_missed_schedules,
            commands::jobs_clear_missed_schedules,
            commands::blueprints_list,
            commands::blueprints_get,
            commands::blueprints_create,
            commands::blueprints_update,
            commands::blueprints_delete,
            commands::blueprints_simulate,
            commands::blueprints_apply,
            commands::blueprints_preview_template,
            commands::blueprints_preview_plan,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
