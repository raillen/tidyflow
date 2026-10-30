mod commands;

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use autoflow_core::{AppState, ExecutionEvent};
use autoflow_infrastructure::ui_state::MissedScheduleEntry;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
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
        .title("TidyFlow — agendamentos perdidos")
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

/// Advisory lock guarding a single TidyFlow instance per data directory.
/// The OS releases the lock when the process exits, so a crash never leaves
/// a stale lock behind.
struct InstanceLock {
    _file: File,
}

fn acquire_instance_lock(data_dir: &std::path::Path) -> Option<InstanceLock> {
    std::fs::create_dir_all(data_dir).ok()?;
    let path = data_dir.join("tidyflow.lock");
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)
        .ok()?;
    if file.try_lock().is_err() {
        return None;
    }
    let _ = file.set_len(0);
    let mut handle = &file;
    let _ = writeln!(handle, "{}", std::process::id());
    Some(InstanceLock { _file: file })
}

/// Emits `security_lock` when the window is hidden and the user asked the PIN
/// to be required whenever the app leaves the foreground (minimize, close to
/// tray or tray "hide").
fn lock_if_configured(app: &tauri::AppHandle, state: &AppState) {
    let settings = state.get_settings_sync();
    if settings.security.pin_enabled
        && settings.security.lock_on_minimize
        && settings.security.access_pin_hash.is_some()
    {
        let _ = app.emit("security_lock", ());
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

            let instance_lock = match acquire_instance_lock(&data_dir) {
                Some(lock) => lock,
                None => {
                    tracing::warn!(
                        "another TidyFlow instance already owns the data directory; exiting"
                    );
                    app.handle().exit(0);
                    return Ok(());
                }
            };

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

            if let Some(window) = app.get_webview_window("main") {
                let state = state.clone();
                let window = window.clone();
                window.clone().on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let settings = state.get_settings_sync();
                        if settings.close_to_tray {
                            api.prevent_close();
                            let _ = window.hide();
                            lock_if_configured(window.app_handle(), &state);
                        }
                    }
                });
            }

            if let Some(window) = app.get_webview_window("main") {
                let state = state.clone();
                let handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        lock_if_configured(&handle, &state);
                    }
                });
            }
            app.manage(state);
            app.manage(instance_lock);

            let tray_menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "show", "Mostrar", true, None::<&str>)?,
                    &MenuItem::with_id(app, "hide", "Ocultar", true, None::<&str>)?,
                    &MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?,
                ],
            )?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                            lock_if_configured(app, app.state::<AppState>().inner());
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health,
            commands::settings_get,
            commands::settings_update,
            commands::security_set_pin,
            commands::security_verify_pin,
            commands::security_clear_pin,
            commands::admin_fleet_snapshot,
            commands::admin_heartbeat_payload,
            commands::admin_signed_heartbeat_payload,
            commands::admin_send_signed_heartbeat_once,
            commands::admin_agent_secret_generate,
            commands::admin_agent_secret_set,
            commands::admin_agent_secret_clear,
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
            commands::rollback_execute,
            commands::executions_list_active,
            commands::executions_cancel,
            commands::queue_pause,
            commands::queue_resume,
            commands::queue_is_paused,
            commands::queue_stop_all,
            commands::audit_list_recent,
            commands::audit_query,
            commands::audit_export,
            commands::audit_clear,
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
            commands::blueprints_simulate_with_variables,
            commands::blueprints_apply,
            commands::blueprints_apply_with_variables,
            commands::blueprints_preview_template,
            commands::blueprints_preview_plan,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
