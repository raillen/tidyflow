using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.IO;
using System.Text.Json;
using FolderFlow.Application.Interfaces;

namespace FolderFlow.Infrastructure.Localization;

public class JsonLocalizationService : ILocalizationService, INotifyPropertyChanged
{
    public event PropertyChangedEventHandler? PropertyChanged;

    private Dictionary<string, string> _translations = new();
    private string _currentCulture = "pt-BR";
    private readonly string _basePath;

    public string this[string key] => GetString(key);

    public JsonLocalizationService()
    {
        _basePath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Assets", "i18n");
        if (!Directory.Exists(_basePath)) Directory.CreateDirectory(_basePath);
        
        LoadTranslations();
    }

    public string GetString(string key)
    {
        return _translations.TryGetValue(key, out var value) ? value : key;
    }

    public void SetLanguage(string cultureCode)
    {
        if (string.IsNullOrEmpty(cultureCode)) cultureCode = "pt-BR";
        _currentCulture = cultureCode;
        LoadTranslations();
        // O indexador  uma propriedade especial chamada "Item"
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs("Item"));
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs("Item[]"));
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(null)); // Notifica tudo
    }

    private void LoadTranslations()
    {
        var filePath = Path.Combine(_basePath, $"{_currentCulture}.json");
        var defaults = GetDefaults(_currentCulture);

        if (File.Exists(filePath))
        {
            try
            {
                var json = File.ReadAllText(filePath);
                var loaded = JsonSerializer.Deserialize<Dictionary<string, string>>(json) ?? new();
                
                // Merge loaded with defaults to ensure new keys are present
                _translations = new Dictionary<string, string>(defaults);
                foreach (var kvp in loaded)
                {
                    _translations[kvp.Key] = kvp.Value;
                }
            }
            catch
            {
                _translations = defaults;
            }
        }
        else
        {
            _translations = defaults;
            SaveTranslations(filePath, _translations);
        }
    }

    private void SaveTranslations(string path, Dictionary<string, string> dict)
    {
        try
        {
            var json = JsonSerializer.Serialize(dict, new JsonSerializerOptions { WriteIndented = true });
            File.WriteAllText(path, json);
        }
        catch { }
    }

    private Dictionary<string, string> GetDefaults(string culture)
    {
        return culture switch
        {
            "en-US" => GetEnUsDefaults(),
            "es-ES" => GetEsEsDefaults(),
            "ja-JP" => GetJaJpDefaults(),
            "ru-RU" => GetRuRuDefaults(),
            _ => GetPtBrDefaults()
        };
    }

    private Dictionary<string, string> GetEnUsDefaults() => new() {
        ["Dashboard"] = "Dashboard", ["Jobs"] = "Queue", ["History"] = "History", ["Settings"] = "Settings", 
        ["Greeting"] = "Good evening!", ["ActiveAuto"] = "You have {0} active automations.",
        ["Total"] = "Total", ["Running"] = "Running", ["Completed"] = "Completed", ["Failed"] = "Failed",
        ["RecentActivity"] = "Recent Activity", ["QuickActions"] = "Quick Actions", ["ErrorAudit"] = "Error Audit",
        ["NewJob"] = "New Task", ["NewAuto"] = "New Automation", ["ViewHistory"] = "View History",
        ["NoErrors"] = "No errors recorded.", ["OpenErrorFolder"] = "Open Error Folder", ["ClearLogs"] = "Clear Logs",
        ["SaveSettings"] = "Save Settings", ["Appearance"] = "Appearance", ["SystemTheme"] = "Theme", ["Language"] = "Language",
        ["Behavior"] = "Behavior", ["ShowNotifications"] = "Notifications", ["StartAtStartup"] = "Auto-start",
        ["Support"] = "Support", ["SupportDesc"] = "If FolderFlow is useful, support us!", ["BuyCoffee"] = "Buy Me a Coffee",
        ["Add"] = "Add", ["Hotfolder"] = "Hotfolder", ["Run"] = "Run", ["StopAll"] = "Stop All", ["Delete"] = "Delete",
        ["QueueManagement"] = "MANAGE OPERATION QUEUE", ["OpLog"] = "Operation Log", ["ManualOp"] = "Manual Operations", ["Automation"] = "Automations",
        ["HistoryHeader"] = "AUTOMATION HISTORY", ["Update"] = "Update", ["SearchHistory"] = "Search files or destinations...",
        ["NoLogs"] = "No logs found.",
        ["Copy"] = "Copy", ["Move"] = "Move", ["Skip"] = "Skip", ["Overwrite"] = "Overwrite", ["Rename"] = "Rename",
        ["None"] = "None", ["Interval"] = "Interval", ["Daily"] = "Daily", ["Weekly"] = "Weekly",
        ["RealTime"] = "Real-Time", ["Polling"] = "Polling", ["OnSuccess"] = "On Success", ["OnError"] = "On Error", ["OnBoth"] = "On Both",
        ["ControlPanel"] = "Control Panel", ["SystemOverview"] = "System overview and performance metrics",
        ["Type"] = "Type:", ["Task"] = "Task:", ["SystemHealth"] = "SYSTEM HEALTH", ["TotalVolume"] = "TOTAL VOLUME",
        ["ProcessedData"] = "Processed data", ["TimeSaved"] = "TIME SAVED", ["AutomatedWork"] = "Automated work",
        ["Files"] = "FILES", ["DirectCopy"] = "Direct Copy", ["WatchFolder"] = "Watch Folder", ["PauseAll"] = "Pause All",
        ["RecentActivities"] = "RECENT ACTIVITIES", ["UpcomingRadar"] = "SCHEDULE RADAR", ["SystemMonitor"] = "SYSTEM MONITOR",
        ["CopySpeed"] = "Copy Speed", ["Processor"] = "Processor (CPU)", ["Ram"] = "RAM Memory", ["CloudSync"] = "Cloud Sync",
        ["All"] = "All", ["Excellent"] = "Excellent", ["Good"] = "Good", ["Attention"] = "Attention", ["Critical"] = "Critical",
        ["NoData"] = "No data", ["Downloading"] = "Downloading...", ["Synced"] = "Synced", ["GlobalPaused"] = "Global processing paused.", ["GlobalResumed"] = "Global processing resumed.",
        ["AdvancedSettings"] = "Advanced Settings", ["SettingsDesc"] = "Full control over performance, integration and visuals",
        ["EnginePerformance"] = "ENGINE & PERFORMANCE", ["Parallelism"] = "Parallelism (Threads)", ["ParallelismDesc"] = "Number of files processed simultaneously",
        ["ProcessPriority"] = "Process Priority", ["PriorityDesc"] = "Software impact on Windows performance",
        ["GlobalBandwidth"] = "Global Bandwidth Limit", ["BandwidthDesc"] = "Limit copy speed to not saturate network (0 = Unlimited)",
        ["SecurityPrivacy"] = "SECURITY & PRIVACY", ["AccessPin"] = "Access PIN", ["PinWatermark"] = "Set a numeric PIN",
        ["PinDesc"] = "Protect interface with a password", ["LockOnMinimize"] = "Lock on minimize",
        ["EncryptionKey"] = "Master Encryption Key", ["EncryptionKeyWatermark"] = "Your global key", ["EncryptionDesc"] = "Suggested as default for new secure jobs",
        ["ExternalNotifications"] = "EXTERNAL NOTIFICATIONS (WEBHOOKS & SMTP)", ["WebhookUrl"] = "Webhook URL",
        ["WebhookUrlWatermark"] = "https://discord.com/api/webhooks/...", ["SmtpServer"] = "SMTP Server",
        ["SmtpHostWatermark"] = "smtp.gmail.com", ["Port"] = "Port", ["DestEmail"] = "Destination Email",
        ["DestEmailWatermark"] = "alerts@email.com", ["EnableSmtp"] = "Enable Email", ["NotifyOnSuccess"] = "Notify Success", ["NotifyOnError"] = "Notify Errors",
        ["AutomationSystem"] = "AUTOMATION & SYSTEM", ["StartWithWindows"] = "Start with Windows", ["StartWithWindowsDesc"] = "Open automatically on computer startup",
        ["CloseToTray"] = "Close to Tray (System Tray)", ["CloseToTrayDesc"] = "Keep FolderFlow running when closing main window",
        ["MaintenanceData"] = "MAINTENANCE & DATA", ["LogRetention"] = "Log Retention", ["LogRetentionDesc"] = "Days to keep records in history",
        ["DatabaseSize"] = "Size: {0}", ["ClearNow"] = "Clear Now", ["OptimizeDatabase"] = "Optimize Database",
        ["VisualLanguage"] = "VISUAL & LANGUAGE", ["InterfaceTheme"] = "Interface Theme", ["SystemLanguage"] = "System Language",
        ["GlassOpacity"] = "Glass Effect Opacity (Mica/Acrylic)", ["DiscardChanges"] = "Discard Changes",
        ["ConfigureAutomation"] = "Configure Automation", ["AutomationRules"] = "Define rules, filters and destinations",
        ["General"] = "General", ["Filters"] = "Filters", ["Schedule"] = "Schedule", ["Security"] = "Security", ["Advanced"] = "Advanced",
        ["JobName"] = "Task Name", ["SourcePath"] = "Source Path", ["DestPath"] = "Destination Path", ["WorkMode"] = "Work Mode",
        ["FileConflicts"] = "File Conflicts", ["IncludeExtensions"] = "Include Extensions", ["IgnorePatterns"] = "Ignore Patterns",
        ["RegexExpression"] = "Regex Expression", ["WatcherConfigs"] = "WATCHER CONFIGS", ["DetectionMethod"] = "Detection Method",
        ["StabilizationTime"] = "Stabilization Time", ["ScheduleType"] = "Schedule Type", ["RepeatOnDays"] = "Repeat on days:",
        ["Sun"] = "S", ["Mon"] = "M", ["Tue"] = "T", ["Wed"] = "W", ["Thu"] = "T", ["Fri"] = "F", ["Sat"] = "S",
        ["ExecutionTime"] = "Execution Time", ["DataProtection"] = "DATA PROTECTION", ["EncryptionKeyLabel"] = "Encryption Key",
        ["JobWebhooks"] = "TASK WEBHOOKS", ["SimulationSummary"] = "SIMULATION SUMMARY", ["FilesIdentified"] = "Files Identified",
        ["DataVolume"] = "Data Volume", ["SimulationSuccess"] = "Simulation completed successfully.",
        ["Simulate"] = "Simulate", ["NewDirectCopyTitle"] = "New Direct Copy", ["NewWatchFolderTitle"] = "New Watch Folder",
        ["Save"] = "Save", ["Cancel"] = "Cancel", ["ProcessSubfolders"] = "Process Subfolders", ["SmartSyncLabel"] = "Smart Sync",
        ["VerifyHashLabel"] = "Verify Hash", ["ClearPreview"] = "Clear Preview", ["SaveAutomation"] = "Save Automation",
        ["AuditHistory"] = "Audit & History", ["ForenseTracking"] = "Forensic tracking of all operations",
        ["ExportCsv"] = "Export CSV", ["OpDetails"] = "OPERATION DETAILS", ["File"] = "File", ["Destination"] = "Destination",
        ["Size"] = "Size", ["Duration"] = "Duration", ["EngineLogs"] = "Engine Logs", ["SelectEntry"] = "Select an entry",
        ["SearchWatermark"] = "Search...", ["Time"] = "Time", ["Origin"] = "Origin", ["OpenSourceFolder"] = "Open Source Folder",
        ["OpenDestFolder"] = "Open Destination Folder", ["ClearHistory"] = "Clear History", ["Copied"] = "COPIED",
        ["Moved"] = "MOVED", ["Ignored"] = "IGNORED", ["FailedStatus"] = "FAILED", ["Zipped"] = "ZIPPED", ["Cancelled"] = "CANCELLED",
        ["SidebarAutomation"] = "Automation", ["SidebarAudit"] = "Audit", ["AutomationHub"] = "Automation Hub",
        ["ActiveBadge"] = "{0} Active", ["QueueReady"] = "Queue Ready", ["AutomationDesc"] = "Manage your data flows",
        ["StopAllTip"] = "Stop All", ["Active"] = "Active", ["Pending"] = "Pending", ["Watchers"] = "Watchers",
        ["SearchJobsWatermark"] = "Search tasks...", ["SelectedCount"] = "{0} selected", ["RealTimeProcessing"] = "REAL-TIME",
        ["ConfiguredTasks"] = "YOUR CONFIGURED TASKS", ["RunTip"] = "Run", ["StopTip"] = "Stop", ["EditTip"] = "Edit", ["DeleteTip"] = "Delete",
        ["OriginLabel"] = "ORIGIN:", ["DestLabel"] = "DEST:", ["Frequency"] = "FREQ", ["SpeedLabel"] = "SPEED", ["LiveTerminal"] = "TERMINAL",
        ["Processing"] = "Processing...", ["Idle"] = "Idle", ["Queued"] = "Queued...", ["NewDirectCopy"] = "New Direct Copy",
        ["NewWatchFolder"] = "New Watch Folder", ["JobNameWatermark"] = "e.g. Backup", ["Browse"] = "Browse",
        ["ExtensionsWatermark"] = ".jpg, .png", ["IgnoreWatermark"] = "temp, node_modules", ["RegexWatermark"] = "^[0-9].*",
        ["OpenApp"] = "Open FolderFlow", ["Exit"] = "Exit", ["SuccessBadge"] = "OK", ["IgnoredBadge"] = "IGN", ["ErrorBadge"] = "ERR", ["SmtpEmailHeader"] = "SMTP / Email",
        ["JobStarted"] = "Task '{0}' started.", ["RetryMode"] = " [RETRY]", ["PreScriptFailed"] = "Pre-script failed.",
        ["SourceNotFound"] = "Source not found: {0}", ["JobError"] = "Job Error", ["JobFailedSourceNotFound"] = "Task '{0}' failed: Source missing.",
        ["StartingFile"] = "Starting: {0}", ["SuccessFile"] = "Success: {0}", ["ErrorFile"] = "ERROR: {0}",
        ["FilterIgnored"] = "Filter ignored.", ["JobFinished"] = "Job Finished: {0}", ["JobCompleted"] = "Job Completed",
        ["FilesFailed"] = "{0} files failed.", ["UserCancelled"] = "Task '{0}' cancelled.", ["UserCancelledOp"] = "Cancelled.",
        ["CriticalError"] = "Critical Error", ["CriticalErrorJob"] = "Error in '{0}': {1}", ["PostScriptFailed"] = "Post-script failed.",
        ["IntegrityFailed"] = "Integrity failed.", ["ZipCreationFailed"] = "ZIP failed: {0}",
        ["RetentionOldDeleted"] = "Retention: Deleted {0}", ["RetentionExpiredDeleted"] = "Retention: Expired {0}",
        ["RetentionFailed"] = "Retention failed: {0}", ["SmartSyncDetail"] = "SmartSync", ["StartingJobLog"] = "Starting Job: {0}",
        ["PreviewErrorSourceNotFound"] = "Source missing: {0}", ["PreviewIgnored"] = "[SKIP] {0}",
        ["PreviewIgnoredSmartSync"] = "[SYNC] {0}", ["PreviewIgnoredConflict"] = "[CONFLICT] {0}",
        ["PreviewOverwrite"] = "[OVERWRITE] {0}", ["PreviewRename"] = "[RENAME] {0}", ["PreviewCopy"] = "[COPY] {0}", ["PreviewMove"] = "[MOVE] {0}",
        ["RamUsageFormat"] = "{0} GB / {1} GB", ["TimeSavedFormat"] = "{0}h {1}m", ["SchedulerLoopError"] = "Scheduler error: {0}",
        ["DailyMaintenanceSuccess"] = "Maintenance OK.", ["DailyMaintenanceError"] = "Maintenance error: {0}",
        ["DailySummaryName"] = "Daily Summary", ["InXDays"] = "in {0} days", ["InXHours"] = "in {0}h", ["InXMinutes"] = "in {0}min",
        ["SupportTitle"] = "Support the Developer", ["SupportPixLabel"] = "Pix Key (Brazil)", ["SupportCoffeeLabel"] = "Buy Me a Coffee", ["SupportCopyPix"] = "Copy Key",
        ["Donate"] = "Donate"
    };

    private Dictionary<string, string> GetEsEsDefaults() => new() {
        ["Dashboard"] = "Panel", ["Jobs"] = "Cola", ["History"] = "Historial", ["Settings"] = "Ajustes", 
        ["Greeting"] = "¡Buenas noches!", ["ActiveAuto"] = "Tienes {0} automatizaciones activas.",
        ["Total"] = "Total", ["Running"] = "Ejecutando", ["Completed"] = "Completado", ["Failed"] = "Fallido",
        ["RecentActivity"] = "Actividad Reciente", ["QuickActions"] = "Acciones Rápidas", ["ErrorAudit"] = "Auditoría",
        ["NewJob"] = "Nueva Tarea", ["NewAuto"] = "Nueva Automatización", ["ViewHistory"] = "Ver Historial",
        ["NoErrors"] = "Sin errores registrados.", ["OpenErrorFolder"] = "Abrir Carpeta", ["ClearLogs"] = "Limpiar Logs",
        ["SaveSettings"] = "Guardar Ajustes", ["Appearance"] = "Apariencia", ["SystemTheme"] = "Tema", ["Language"] = "Idioma",
        ["Behavior"] = "Comportamiento", ["ShowNotifications"] = "Notificaciones", ["StartAtStartup"] = "Auto-inicio",
        ["Support"] = "Soporte", ["SupportDesc"] = "¡Apóyanos!", ["BuyCoffee"] = "Invítame a un café",
        ["Add"] = "Añadir", ["Hotfolder"] = "Hotfolder", ["Run"] = "Ejecutar", ["StopAll"] = "Parar Todo", ["Delete"] = "Eliminar",
        ["QueueManagement"] = "GESTIÓN DE COLA", ["OpLog"] = "Log de Operações", ["ManualOp"] = "Operaciones Manuales", ["Automation"] = "Automatizaciones",
        ["HistoryHeader"] = "HISTORIAL DE AUTOMATIZACIÓN", ["Update"] = "Actualizar", ["SearchHistory"] = "Buscar...",
        ["NoLogs"] = "No hay logs.",
        ["Copy"] = "Copiar", ["Move"] = "Mover", ["Skip"] = "Omitir", ["Overwrite"] = "Sobrescribir", ["Rename"] = "Renombrar",
        ["None"] = "Ninguno", ["Interval"] = "Intervalo", ["Daily"] = "Diario", ["Weekly"] = "Semanal",
        ["RealTime"] = "Tiempo Real", ["Polling"] = "Sondeo", ["OnSuccess"] = "Al Éxito", ["OnError"] = "Al Error", ["OnBoth"] = "En Ambos",
        ["ControlPanel"] = "Panel de Controle", ["SystemOverview"] = "Visión general del sistema",
        ["Type"] = "Tipo:", ["Task"] = "Tarea:", ["SystemHealth"] = "SALUD DEL SISTEMA", ["TotalVolume"] = "VOLUMEN TOTAL",
        ["ProcessedData"] = "Datos processados", ["TimeSaved"] = "TIEMPO AHORRADO", ["AutomatedWork"] = "Trabajo automatizado",
        ["Files"] = "ARCHIVOS", ["DirectCopy"] = "Copia Directa", ["WatchFolder"] = "Watch Folder", ["PauseAll"] = "Pausar Todo",
        ["RecentActivities"] = "ACTIVIDADES RECENTES", ["UpcomingRadar"] = "RADAR DE TAREAS", ["SystemMonitor"] = "MONITOR DEL SISTEMA",
        ["CopySpeed"] = "Velocidad", ["Processor"] = "Procesador", ["Ram"] = "Memoria RAM", ["CloudSync"] = "Sincro Nube",
        ["All"] = "Todos", ["Excellent"] = "Excelente", ["Good"] = "Bueno", ["Attention"] = "Atención", ["Critical"] = "Crítico",
        ["NoData"] = "Sin datos", ["Downloading"] = "Descargando...", ["Synced"] = "Sincronizado", ["GlobalPaused"] = "Procesamiento pausado.", ["GlobalResumed"] = "Procesamiento reanudado.",
        ["AdvancedSettings"] = "Ajustes Avanzados", ["SettingsDesc"] = "Control total del sistema",
        ["EnginePerformance"] = "MOTOR Y RENDIMIENTO", ["Parallelism"] = "Paralelismo", ["ParallelismDesc"] = "Archivos simultáneos",
        ["ProcessPriority"] = "Prioridad", ["PriorityDesc"] = "Impacto en el sistema",
        ["GlobalBandwidth"] = "Límite de Banda", ["BandwidthDesc"] = "0 = Ilimitado",
        ["SecurityPrivacy"] = "SEGURIDAD Y PRIVACIDAD", ["AccessPin"] = "PIN de Acceso", ["PinWatermark"] = "Introduce un PIN",
        ["PinDesc"] = "Protege la interfaz", ["LockOnMinimize"] = "Bloquear al minimizar",
        ["EncryptionKey"] = "Clave Maestra", ["EncryptionKeyWatermark"] = "Tu clave global", ["EncryptionDesc"] = "Por defecto para nuevos jobs",
        ["ExternalNotifications"] = "NOTIFICACIONES (WEBHOOK & SMTP)", ["WebhookUrl"] = "URL Webhook",
        ["WebhookUrlWatermark"] = "https://...", ["SmtpServer"] = "Servidor SMTP",
        ["SmtpHostWatermark"] = "smtp.ejemplo.com", ["Port"] = "Puerto", ["DestEmail"] = "Email Destino",
        ["DestEmailWatermark"] = "alertas@email.com", ["EnableSmtp"] = "Activar Email", ["NotifyOnSuccess"] = "Notificar Éxito", ["NotifyOnError"] = "Notificar Error",
        ["AutomationSystem"] = "SISTEMA", ["StartWithWindows"] = "Inicio con Windows", ["StartWithWindowsDesc"] = "Arrancar al encender",
        ["CloseToTray"] = "Cerrar a la Bandeja", ["CloseToTrayDesc"] = "Mantener en segundo plano",
        ["MaintenanceData"] = "MANTENIMIENTO", ["LogRetention"] = "Retención de Logs", ["LogRetentionDesc"] = "Días de historial",
        ["DatabaseSize"] = "Tamaño: {0}", ["ClearNow"] = "Limpiar Ahora", ["OptimizeDatabase"] = "Optimizar BD",
        ["VisualLanguage"] = "VISUAL E IDIOMA", ["InterfaceTheme"] = "Tema", ["SystemLanguage"] = "Idioma",
        ["GlassOpacity"] = "Opacidade Cristal", ["DiscardChanges"] = "Descartar",
        ["ConfigureAutomation"] = "Configurar Tarea", ["AutomationRules"] = "Define reglas y destinos",
        ["General"] = "General", ["Filters"] = "Filtros", ["Schedule"] = "Agenda", ["Security"] = "Seguridad", ["Advanced"] = "Avanzado",
        ["JobName"] = "Nombre", ["SourcePath"] = "Origen", ["DestPath"] = "Destino", ["WorkMode"] = "Modo",
        ["FileConflicts"] = "Conflictos", ["IncludeExtensions"] = "Extensiones", ["IgnorePatterns"] = "Ignorar",
        ["RegexExpression"] = "Regex", ["WatcherConfigs"] = "WATCHER", ["DetectionMethod"] = "Método",
        ["StabilizationTime"] = "Estabilización", ["ScheduleType"] = "Tipo Agenda", ["RepeatOnDays"] = "Días:",
        ["Sun"] = "D", ["Mon"] = "L", ["Tue"] = "M", ["Wed"] = "M", ["Thu"] = "J", ["Fri"] = "V", ["Sat"] = "S",
        ["ExecutionTime"] = "Hora", ["DataProtection"] = "PROTECCIÓN", ["EncryptionKeyLabel"] = "Clave",
        ["JobWebhooks"] = "WEBHOOKS", ["SimulationSummary"] = "RESUMEN", ["FilesIdentified"] = "Archivos",
        ["DataVolume"] = "Volumen", ["SimulationSuccess"] = "Simulación OK.",
        ["Simulate"] = "Simular", ["NewDirectCopyTitle"] = "Nueva Copia", ["NewWatchFolderTitle"] = "Nueva Watch",
        ["Save"] = "Guardar", ["Cancel"] = "Cancelar", ["ProcessSubfolders"] = "Subcarpetas", ["SmartSyncLabel"] = "Smart Sync",
        ["VerifyHashLabel"] = "Verificar Hash", ["ClearPreview"] = "Limpiar", ["SaveAutomation"] = "Guardar Tarea",
        ["AuditHistory"] = "Auditoría", ["ForenseTracking"] = "Rastreo de operaciones",
        ["ExportCsv"] = "Exportar CSV", ["OpDetails"] = "DETALLES", ["File"] = "Archivo", ["Destination"] = "Destino",
        ["Size"] = "Tamaño", ["Duration"] = "Duración", ["EngineLogs"] = "Logs Motor", ["SelectEntry"] = "Selecciona una entrada",
        ["SearchWatermark"] = "Buscar...", ["Time"] = "Hora", ["Origin"] = "Origen", ["OpenSourceFolder"] = "Abrir Origen",
        ["OpenDestFolder"] = "Abrir Destino", ["ClearHistory"] = "Limpar Historial", ["Copied"] = "COPIADO",
        ["Moved"] = "MOVIDO", ["Ignored"] = "IGNORADO", ["FailedStatus"] = "FALLIDO", ["Zipped"] = "ZIPADO", ["Cancelled"] = "CANCELADO",
        ["SidebarAutomation"] = "Automatización", ["SidebarAudit"] = "Auditoría", ["AutomationHub"] = "Central",
        ["ActiveBadge"] = "{0} Activos", ["QueueReady"] = "Cola Lista", ["AutomationDesc"] = "Gestiona tus flujos",
        ["StopAllTip"] = "Parar Todo", ["Active"] = "Activos", ["Pending"] = "Pendentes", ["Watchers"] = "Watchers",
        ["SearchJobsWatermark"] = "Buscar...", ["SelectedCount"] = "{0} seleccionados", ["RealTimeProcessing"] = "TIEMPO REAL",
        ["ConfiguredTasks"] = "TAREAS", ["RunTip"] = "Arrancar", ["StopTip"] = "Parar", ["EditTip"] = "Editar", ["DeleteTip"] = "Borrar",
        ["OriginLabel"] = "ORIGEN:", ["DestLabel"] = "DEST:", ["Frequency"] = "FREQ", ["SpeedLabel"] = "VEL", ["LiveTerminal"] = "TERMINAL",
        ["Processing"] = "Ejecutando...", ["Idle"] = "Inactivo", ["Queued"] = "En cola...", ["NewDirectCopy"] = "Nueva Copia",
        ["NewWatchFolder"] = "Nueva Watch", ["JobNameWatermark"] = "Ej: Backup", ["Browse"] = "Buscar",
        ["ExtensionsWatermark"] = ".jpg", ["IgnoreWatermark"] = "temp", ["RegexWatermark"] = "^[0-9].*",
        ["OpenApp"] = "Abrir", ["Exit"] = "Salir", ["SuccessBadge"] = "OK", ["IgnoredBadge"] = "IGN", ["ErrorBadge"] = "ERR", ["SmtpEmailHeader"] = "SMTP",
        ["JobStarted"] = "Tarea '{0}' iniciada.", ["RetryMode"] = " [REINTENTO]", ["PreScriptFailed"] = "Script previo falló.",
        ["SourceNotFound"] = "Origen no encontrado: {0}", ["JobError"] = "Error", ["JobFailedSourceNotFound"] = "Error: Falta origen.",
        ["StartingFile"] = "Iniciando: {0}", ["SuccessFile"] = "Éxito: {0}", ["ErrorFile"] = "ERROR: {0}",
        ["FilterIgnored"] = "Filtro aplicado.", ["JobFinished"] = "Finalizado: {0}", ["JobCompleted"] = "Tarea Completada",
        ["FilesFailed"] = "{0} fallidos.", ["UserCancelled"] = "Cancelado por usuario.", ["UserCancelledOp"] = "Cancelado.",
        ["CriticalError"] = "Error Crítico", ["CriticalErrorJob"] = "Error en '{0}': {1}", ["PostScriptFailed"] = "Script post falló.",
        ["IntegrityFailed"] = "Error integridad.", ["ZipCreationFailed"] = "Error ZIP: {0}",
        ["RetentionOldDeleted"] = "Retención: Borrado {0}", ["RetentionExpiredDeleted"] = "Retención: Expirado {0}",
        ["RetentionFailed"] = "Error retención: {0}", ["SmartSyncDetail"] = "SmartSync", ["StartingJobLog"] = "Iniciando: {0}",
        ["PreviewErrorSourceNotFound"] = "Sin origen: {0}", ["PreviewIgnored"] = "[IGN] {0}",
        ["PreviewIgnoredSmartSync"] = "[SYNC] {0}", ["PreviewIgnoredConflict"] = "[CONF] {0}",
        ["PreviewOverwrite"] = "[SOB] {0}", ["PreviewRename"] = "[REN] {0}", ["PreviewCopy"] = "[COP] {0}", ["PreviewMove"] = "[MOV] {0}",
        ["RamUsageFormat"] = "{0} GB / {1} GB", ["TimeSavedFormat"] = "{0}h {1}m", ["SchedulerLoopError"] = "Error agenda: {0}",
        ["DailyMaintenanceSuccess"] = "Mantenimiento OK.", ["DailyMaintenanceError"] = "Error mantenimiento: {0}",
        ["DailySummaryName"] = "Resumen Diario", ["InXDays"] = "en {0} días", ["InXHours"] = "en {0}h", ["InXMinutes"] = "en {0}min",
        ["SupportTitle"] = "Apoya al Desarrollador", ["SupportPixLabel"] = "Clave Pix (Brasil)", ["SupportCoffeeLabel"] = "Buy Me a Coffee", ["SupportCopyPix"] = "Copiar Clave",
        ["Donate"] = "Donar"
    };

    private Dictionary<string, string> GetJaJpDefaults() => new() {
        ["Dashboard"] = "ダッシュボード", ["Jobs"] = "キュー", ["History"] = "履歴", ["Settings"] = "設定", 
        ["Greeting"] = "こんにちは！", ["ActiveAuto"] = "{0} 個の自動化が有効です。",
        ["Total"] = "合計", ["Running"] = "実行中", ["Completed"] = "完了", ["Failed"] = "失敗",
        ["RecentActivity"] = "最近の活動", ["QuickActions"] = "アクション", ["ErrorAudit"] = "監査",
        ["NewJob"] = "新規タスク", ["NewAuto"] = "新規自動化", ["ViewHistory"] = "履歴表示",
        ["NoErrors"] = "エラーなし", ["OpenErrorFolder"] = "フォルダを開く", ["ClearLogs"] = "ログ削除",
        ["SaveSettings"] = "設定保存", ["Appearance"] = "外観", ["SystemTheme"] = "テーマ", ["Language"] = "言語",
        ["Behavior"] = "動作", ["ShowNotifications"] = "通知を表示", ["StartAtStartup"] = "自動起動",
        ["Support"] = "サポート", ["SupportDesc"] = "開発者を支援する", ["BuyCoffee"] = "コーヒーを買う",
        ["Add"] = "追加", ["Hotfolder"] = "ホットフォルダ", ["Run"] = "開始", ["StopAll"] = "全停止", ["Delete"] = "削除",
        ["QueueManagement"] = "キュー管理", ["OpLog"] = "操作ログ", ["ManualOp"] = "手動操作", ["Automation"] = "自動化",
        ["HistoryHeader"] = "自動化履歴", ["Update"] = "更新", ["SearchHistory"] = "検索...",
        ["NoLogs"] = "ログなし",
        ["Copy"] = "コピー", ["Move"] = "移動", ["Skip"] = "スキップ", ["Overwrite"] = "上書き", ["Rename"] = "名前変更",
        ["None"] = "なし", ["Interval"] = "間隔", ["Daily"] = "毎日", ["Weekly"] = "毎週",
        ["RealTime"] = "リアルタイム", ["Polling"] = "ポーリング", ["OnSuccess"] = "成功時", ["OnError"] = "エラー時", ["OnBoth"] = "両方",
        ["ControlPanel"] = "コントロールパネル", ["SystemOverview"] = "システム概要",
        ["Type"] = "タイプ:", ["Task"] = "タスク:", ["SystemHealth"] = "システム状態", ["TotalVolume"] = "総ボリューム",
        ["ProcessedData"] = "処理済みデータ", ["TimeSaved"] = "節約された時間", ["AutomatedWork"] = "自動化された作業",
        ["Files"] = "ファイル", ["DirectCopy"] = "直接コピー", ["WatchFolder"] = "監視フォルダ", ["PauseAll"] = "全一時停止",
        ["RecentActivities"] = "最近の活動", ["UpcomingRadar"] = "スケジュール予定", ["SystemMonitor"] = "システムモニター",
        ["CopySpeed"] = "コピー速度", ["Processor"] = "CPU使用率", ["Ram"] = "メモリ", ["CloudSync"] = "クラウド同期",
        ["All"] = "すべて", ["Excellent"] = "良好", ["Good"] = "普通", ["Attention"] = "注意", ["Critical"] = "危険",
        ["NoData"] = "データなし", ["Downloading"] = "ダウンロード中...", ["Synced"] = "同期済み", ["GlobalPaused"] = "一時停止中", ["GlobalResumed"] = "再開済み",
        ["AdvancedSettings"] = "詳細設定", ["SettingsDesc"] = "パフォーマンスと統合の設定",
        ["EnginePerformance"] = "エンジンとパフォーマンス", ["Parallelism"] = "並列処理", ["ParallelismDesc"] = "同時処理数",
        ["ProcessPriority"] = "優先度", ["PriorityDesc"] = "システムへの影響",
        ["GlobalBandwidth"] = "帯域幅制限", ["BandwidthDesc"] = "0 = 無制限",
        ["SecurityPrivacy"] = "セキュリティ", ["AccessPin"] = "アクセスPIN", ["PinWatermark"] = "PINを入力",
        ["PinDesc"] = "パスワード保護", ["LockOnMinimize"] = "最小化時にロック",
        ["EncryptionKey"] = "暗号化キー", ["EncryptionKeyWatermark"] = "グローバルキー", ["EncryptionDesc"] = "新規ジョブのデフォルト",
        ["ExternalNotifications"] = "外部通知", ["WebhookUrl"] = "Webhook URL",
        ["WebhookUrlWatermark"] = "https://...", ["SmtpServer"] = "SMTPサーバー",
        ["SmtpHostWatermark"] = "smtp.gmail.com", ["Port"] = "ポート", ["DestEmail"] = "宛先メール",
        ["DestEmailWatermark"] = "alerts@email.com", ["EnableSmtp"] = "メール有効化", ["NotifyOnSuccess"] = "成功を通知", ["NotifyOnError"] = "エラーを通知",
        ["AutomationSystem"] = "システム設定", ["StartWithWindows"] = "Windows起動時に開始", ["StartWithWindowsDesc"] = "自動実行",
        ["CloseToTray"] = "トレイに閉じる", ["CloseToTrayDesc"] = "バックグラウンド実行",
        ["MaintenanceData"] = "メンテナンス", ["LogRetention"] = "ログ保持期間", ["LogRetentionDesc"] = "保存日数",
        ["DatabaseSize"] = "サイズ: {0}", ["ClearNow"] = "今すぐ削除", ["OptimizeDatabase"] = "最適化",
        ["VisualLanguage"] = "表示と言語", ["InterfaceTheme"] = "テーマ", ["SystemLanguage"] = "システム言語",
        ["GlassOpacity"] = "不透明度", ["DiscardChanges"] = "破棄",
        ["ConfigureAutomation"] = "自動化設定", ["AutomationRules"] = "ルールと宛先の設定",
        ["General"] = "一般", ["Filters"] = "フィルタ", ["Schedule"] = "スケジュール", ["Security"] = "安全", ["Advanced"] = "詳細",
        ["JobName"] = "タスク名", ["SourcePath"] = "元のパス", ["DestPath"] = "保存先パス", ["WorkMode"] = "動作モード",
        ["FileConflicts"] = "競合解決", ["IncludeExtensions"] = "拡張子指定", ["IgnorePatterns"] = "除外パターン",
        ["RegexExpression"] = "正規表現", ["WatcherConfigs"] = "監視設定", ["DetectionMethod"] = "検出方法",
        ["StabilizationTime"] = "安定化時間", ["ScheduleType"] = "スケジュール形式", ["RepeatOnDays"] = "実行日:",
        ["Sun"] = "日", ["Mon"] = "月", ["Tue"] = "火", ["Wed"] = "水", ["Thu"] = "木", ["Fri"] = "金", ["Sat"] = "土",
        ["ExecutionTime"] = "実行時間", ["DataProtection"] = "データ保護", ["EncryptionKeyLabel"] = "暗号化キー",
        ["JobWebhooks"] = "Webhook設定", ["SimulationSummary"] = "シミュレーション結果", ["FilesIdentified"] = "特定ファイル",
        ["DataVolume"] = "データ量", ["SimulationSuccess"] = "シミュレーション成功",
        ["Simulate"] = "プレビュー", ["NewDirectCopyTitle"] = "新規コピー", ["NewWatchFolderTitle"] = "新規監視",
        ["Save"] = "保存", ["Cancel"] = "キャンセル", ["ProcessSubfolders"] = "サブフォルダを含む", ["SmartSyncLabel"] = "スマート同期",
        ["VerifyHashLabel"] = "整合性確認", ["ClearPreview"] = "クリア", ["SaveAutomation"] = "タスク保存",
        ["AuditHistory"] = "監査と履歴", ["ForenseTracking"] = "全操作の追跡",
        ["ExportCsv"] = "CSV出力", ["OpDetails"] = "詳細情報", ["File"] = "ファイル", ["Destination"] = "宛先",
        ["Size"] = "サイズ", ["Duration"] = "所要時間", ["EngineLogs"] = "エンジンログ", ["SelectEntry"] = "項目を選択",
        ["SearchWatermark"] = "検索中...", ["Time"] = "時間", ["Origin"] = "元", ["OpenSourceFolder"] = "元フォルダを開く",
        ["OpenDestFolder"] = "先フォルダを開く", ["ClearHistory"] = "履歴をクリア", ["Copied"] = "コピー済み",
        ["Moved"] = "移動済み", ["Ignored"] = "無視", ["FailedStatus"] = "失敗", ["Zipped"] = "圧縮済み", ["Cancelled"] = "キャンセル",
        ["SidebarAutomation"] = "自動化", ["SidebarAudit"] = "監査", ["AutomationHub"] = "自動化センター",
        ["ActiveBadge"] = "{0} 実行中", ["QueueReady"] = "準備完了", ["AutomationDesc"] = "データフローの管理",
        ["StopAllTip"] = "すべて停止", ["Active"] = "有効", ["Pending"] = "保留中", ["Watchers"] = "監視中",
        ["SearchJobsWatermark"] = "タスクを検索...", ["SelectedCount"] = "{0}件選択", ["RealTimeProcessing"] = "リアルタイム処理",
        ["ConfiguredTasks"] = "設定済みタスク", ["RunTip"] = "実行", ["StopTip"] = "停止", ["EditTip"] = "編集", ["DeleteTip"] = "削除",
        ["OriginLabel"] = "元:", ["DestLabel"] = "先:", ["Frequency"] = "頻度", ["SpeedLabel"] = "速度", ["LiveTerminal"] = "ターミナル",
        ["Processing"] = "処理中...", ["Idle"] = "待機中", ["Queued"] = "待機中...", ["NewDirectCopy"] = "新規コピー",
        ["NewWatchFolder"] = "新規監視", ["JobNameWatermark"] = "例: バックアップ", ["Browse"] = "参照",
        ["ExtensionsWatermark"] = ".jpg, .pdf", ["IgnoreWatermark"] = "temp", ["RegexWatermark"] = "^[0-9].*",
        ["OpenApp"] = "アプリを開く", ["Exit"] = "終了", ["SuccessBadge"] = "OK", ["IgnoredBadge"] = "無視", ["ErrorBadge"] = "エラー", ["SmtpEmailHeader"] = "SMTPメール",
        ["JobStarted"] = "タスク '{0}' を開始しました。", ["RetryMode"] = " [再試行]", ["PreScriptFailed"] = "前処理失敗。",
        ["SourceNotFound"] = "元フォルダが見つかりません: {0}", ["JobError"] = "ジョブエラー", ["JobFailedSourceNotFound"] = "タスク失敗: 元なし",
        ["StartingFile"] = "開始: {0}", ["SuccessFile"] = "成功: {0}", ["ErrorFile"] = "エラー: {0}",
        ["FilterIgnored"] = "フィルタで除外されました。", ["JobFinished"] = "終了: {0}", ["JobCompleted"] = "タスク完了",
        ["FilesFailed"] = "{0}個のファイルが失敗しました。", ["UserCancelled"] = "ユーザーにより中止。", ["UserCancelledOp"] = "中止済み。",
        ["CriticalError"] = "致命的なエラー", ["CriticalErrorJob"] = "エラー '{0}': {1}", ["PostScriptFailed"] = "後処理失敗。",
        ["IntegrityFailed"] = "整合性確認失敗。", ["ZipCreationFailed"] = "ZIP作成失敗: {0}",
        ["RetentionOldDeleted"] = "保持期限: 削除 {0}", ["RetentionExpiredDeleted"] = "保持期限: 期限切れ {0}",
        ["RetentionFailed"] = "保持期限適用失敗: {0}", ["SmartSyncDetail"] = "スマート同期", ["StartingJobLog"] = "ジョブ開始: {0}",
        ["PreviewErrorSourceNotFound"] = "元なし: {0}", ["PreviewIgnored"] = "[無視] {0}",
        ["PreviewIgnoredSmartSync"] = "[同期] {0}", ["PreviewIgnoredConflict"] = "[競合] {0}",
        ["PreviewOverwrite"] = "[上書き] {0}", ["PreviewRename"] = "[名前変更] {0}", ["PreviewCopy"] = "[コピー] {0}", ["PreviewMove"] = "[移動] {0}",
        ["RamUsageFormat"] = "{0} GB / {1} GB", ["TimeSavedFormat"] = "{0}時間 {1}分", ["SchedulerLoopError"] = "スケジュールエラー: {0}",
        ["DailyMaintenanceSuccess"] = "メンテナンス完了。", ["DailyMaintenanceError"] = "メンテナンスエラー: {0}",
        ["DailySummaryName"] = "デイリーサマリー", ["InXDays"] = "{0}日後", ["InXHours"] = "{0}時間後", ["InXMinutes"] = "{0}分後",
        ["SupportTitle"] = "開発者を支援", ["SupportPixLabel"] = "Pixキー (ブラジル)", ["SupportCoffeeLabel"] = "コーヒーを買う", ["SupportCopyPix"] = "キーをコピー",
        ["Donate"] = "寄付"
    };

    private Dictionary<string, string> GetRuRuDefaults() => new() {
        ["Dashboard"] = "Панель", ["Jobs"] = "Очередь", ["History"] = "История", ["Settings"] = "Настройки", 
        ["Greeting"] = "Добрый вечер!", ["ActiveAuto"] = "Активных автоматизаций: {0}.",
        ["Total"] = "Всего", ["Running"] = "Запущено", ["Completed"] = "Готово", ["Failed"] = "Ошибка",
        ["RecentActivity"] = "Активность", ["QuickActions"] = "Действия", ["ErrorAudit"] = "Аудит",
        ["NewJob"] = "Новая задача", ["NewAuto"] = "Автоматизация", ["ViewHistory"] = "История",
        ["NoErrors"] = "Ошибок нет.", ["OpenErrorFolder"] = "Открыть папку", ["ClearLogs"] = "Очистить",
        ["SaveSettings"] = "Сохранить", ["Appearance"] = "Внешний вид", ["SystemTheme"] = "Тема", ["Language"] = "Язык",
        ["Behavior"] = "Поведение", ["ShowNotifications"] = "Уведомления", ["StartAtStartup"] = "Автозагрузка",
        ["Support"] = "Поддержка", ["SupportDesc"] = "Поддержите нас!", ["BuyCoffee"] = "Купить кофе",
        ["Add"] = "Добавить", ["Hotfolder"] = "Хотфолдер", ["Run"] = "Пуск", ["StopAll"] = "Стоп", ["Delete"] = "Удалить",
        ["QueueManagement"] = "УПРАВЛЕНИЕ ОЧЕРЕДЬЮ", ["OpLog"] = "Лог операций", ["ManualOp"] = "Вручную", ["Automation"] = "Автоматика",
        ["HistoryHeader"] = "ИСТОРИЯ", ["Update"] = "Обновить", ["SearchHistory"] = "Поиск...",
        ["NoLogs"] = "Логов нет.",
        ["Copy"] = "Копировать", ["Move"] = "Переместить", ["Skip"] = "Пропустить", ["Overwrite"] = "Заменить", ["Rename"] = "Переименовать",
        ["None"] = "Нет", ["Interval"] = "Интервал", ["Daily"] = "Ежедневно", ["Weekly"] = "Еженедельно",
        ["RealTime"] = "В реальном времени", ["Polling"] = "Опрос", ["OnSuccess"] = "При успехе", ["OnError"] = "При ошибке", ["OnBoth"] = "Всегда",
        ["ControlPanel"] = "Панель управления", ["SystemOverview"] = "Обзор системы",
        ["Type"] = "Тип:", ["Task"] = "Задача:", ["SystemHealth"] = "СОСТОЯНИЕ СИСТЕМЫ", ["TotalVolume"] = "ОБЪЕМ",
        ["ProcessedData"] = "Обработано", ["TimeSaved"] = "ВРЕМЯ СЭКОНОМЛЕНО", ["AutomatedWork"] = "Автоматизировано",
        ["Files"] = "ФАЙЛЫ", ["DirectCopy"] = "Копирование", ["WatchFolder"] = "Слежение", ["PauseAll"] = "Пауза",
        ["RecentActivities"] = "АКТИВНОСТЬ", ["UpcomingRadar"] = "РАСПИСАНИЕ", ["SystemMonitor"] = "МОНИТОРИНГ",
        ["CopySpeed"] = "Скорость", ["Processor"] = "Процессор", ["Ram"] = "Память RAM", ["CloudSync"] = "Облако",
        ["All"] = "Все", ["Excellent"] = "Отлично", ["Good"] = "Хорошо", ["Attention"] = "Внимание", ["Critical"] = "Критично",
        ["NoData"] = "Нет данных", ["Downloading"] = "Загрузка...", ["Synced"] = "Синхронизировано", ["GlobalPaused"] = "Пауза.", ["GlobalResumed"] = "Возобновлено.",
        ["AdvancedSettings"] = "Настройки", ["SettingsDesc"] = "Производительность и вид",
        ["EnginePerformance"] = "ДВИЖОК", ["Parallelism"] = "Потоки", ["ParallelismDesc"] = "Одновременные файлы",
        ["ProcessPriority"] = "Приоритет", ["PriorityDesc"] = "Влияние на Windows",
        ["GlobalBandwidth"] = "Лимит скорости", ["BandwidthDesc"] = "0 = Без лимита",
        ["SecurityPrivacy"] = "БЕЗОПАСНОСТЬ", ["AccessPin"] = "PIN-код", ["PinWatermark"] = "Введите PIN",
        ["PinDesc"] = "Защита паролем", ["LockOnMinimize"] = "Блок. при сворачивании",
        ["EncryptionKey"] = "Ключ шифрования", ["EncryptionKeyWatermark"] = "Глобальный ключ", ["EncryptionDesc"] = "Для новых задач",
        ["ExternalNotifications"] = "УВЕДОМЛЕНИЯ", ["WebhookUrl"] = "Webhook URL",
        ["WebhookUrlWatermark"] = "https://...", ["SmtpServer"] = "SMTP сервер",
        ["SmtpHostWatermark"] = "smtp.yandex.ru", ["Port"] = "Порт", ["DestEmail"] = "Email получателя",
        ["DestEmailWatermark"] = "admin@mail.ru", ["EnableSmtp"] = "Включить Email", ["NotifyOnSuccess"] = "При успехе", ["NotifyOnError"] = "При ошибке",
        ["AutomationSystem"] = "СИСТЕМА", ["StartWithWindows"] = "Автозапуск", ["StartWithWindowsDesc"] = "При входе в Windows",
        ["CloseToTray"] = "В трей", ["CloseToTrayDesc"] = "Работа в фоне",
        ["MaintenanceData"] = "ДАННЫЕ", ["LogRetention"] = "Хранение логов", ["LogRetentionDesc"] = "Дней в истории",
        ["DatabaseSize"] = "Размер: {0}", ["ClearNow"] = "Очистить сейчас", ["OptimizeDatabase"] = "Оптимизировать",
        ["VisualLanguage"] = "ВИД И ЯЗЫК", ["InterfaceTheme"] = "Тема", ["SystemLanguage"] = "Язык",
        ["GlassOpacity"] = "Прозрачность", ["DiscardChanges"] = "Сбросить",
        ["ConfigureAutomation"] = "Настройка", ["AutomationRules"] = "Правила и пути",
        ["General"] = "Общие", ["Filters"] = "Фильтры", ["Schedule"] = "График", ["Security"] = "Защита", ["Advanced"] = "Доп.",
        ["JobName"] = "Название", ["SourcePath"] = "Источник", ["DestPath"] = "Назначение", ["WorkMode"] = "Режим",
        ["FileConflicts"] = "Конфликты", ["IncludeExtensions"] = "Расширения", ["IgnorePatterns"] = "Игнорировать",
        ["RegexExpression"] = "Regex", ["WatcherConfigs"] = "WATCHER", ["DetectionMethod"] = "Метод",
        ["StabilizationTime"] = "Стабилизация", ["ScheduleType"] = "Тип графика", ["RepeatOnDays"] = "Дни:",
        ["Sun"] = "В", ["Mon"] = "П", ["Tue"] = "В", ["Wed"] = "С", ["Thu"] = "Ч", ["Fri"] = "П", ["Sat"] = "С",
        ["ExecutionTime"] = "Время", ["DataProtection"] = "ЗАЩИТА ДАННЫХ", ["EncryptionKeyLabel"] = "Ключ",
        ["JobWebhooks"] = "WEBHOOKS", ["SimulationSummary"] = "ПРЕВЬЮ", ["FilesIdentified"] = "Файлов",
        ["DataVolume"] = "Объем", ["SimulationSuccess"] = "Успешно.",
        ["Simulate"] = "Тест", ["NewDirectCopyTitle"] = "Новая копия", ["NewWatchFolderTitle"] = "Новая Watch",
        ["Save"] = "ОК", ["Cancel"] = "Отмена", ["ProcessSubfolders"] = "Включая подпапки", ["SmartSyncLabel"] = "Smart Sync",
        ["VerifyHashLabel"] = "Проверка Hash", ["ClearPreview"] = "Очистить", ["SaveAutomation"] = "Сохранить",
        ["AuditHistory"] = "Аудит", ["ForenseTracking"] = "История всех операций",
        ["ExportCsv"] = "В CSV", ["OpDetails"] = "ДЕТАЛИ", ["File"] = "Файл", ["Destination"] = "Куда",
        ["Size"] = "Размер", ["Duration"] = "Время", ["EngineLogs"] = "Лог движка", ["SelectEntry"] = "Выберите запись",
        ["SearchWatermark"] = "Поиск...", ["Time"] = "Время", ["Origin"] = "Откуда", ["OpenSourceFolder"] = "Открыть источник",
        ["OpenDestFolder"] = "Открыть цель", ["ClearHistory"] = "Очистить историю", ["Copied"] = "ОК: КОПИЯ",
        ["Moved"] = "ОК: ПЕРЕНОС", ["Ignored"] = "ПРОПУЩЕНО", ["FailedStatus"] = "ОШИБКА", ["Zipped"] = "В АРХИВЕ", ["Cancelled"] = "ОТМЕНЕНО",
        ["SidebarAutomation"] = "Автоматика", ["SidebarAudit"] = "Аудит", ["AutomationHub"] = "Центр",
        ["ActiveBadge"] = "{0} Активно", ["QueueReady"] = "Готов", ["AutomationDesc"] = "Управление потоками",
        ["StopAllTip"] = "Стоп все", ["Active"] = "Активны", ["Pending"] = "Ожидают", ["Watchers"] = "Watchers",
        ["SearchJobsWatermark"] = "Поиск...", ["SelectedCount"] = "{0} выбрано", ["RealTimeProcessing"] = "LIVE РЕЖИМ",
        ["ConfiguredTasks"] = "ЗАДАЧИ", ["RunTip"] = "Пуск", ["StopTip"] = "Стоп", ["EditTip"] = "Правка", ["DeleteTip"] = "Удалить",
        ["OriginLabel"] = "ОТКУДА:", ["DestLabel"] = "КУДА:", ["Frequency"] = "ГРАФИК", ["SpeedLabel"] = "СКОРОСТЬ", ["LiveTerminal"] = "ТЕРМИНАЛ",
        ["Processing"] = "В работе...", ["Idle"] = "Спит", ["Queued"] = "В очереди...", ["NewDirectCopy"] = "Копия",
        ["NewWatchFolder"] = "Watch", ["JobNameWatermark"] = "Напр. Бэкап", ["Browse"] = "Обзор",
        ["ExtensionsWatermark"] = ".jpg, .zip", ["IgnoreWatermark"] = "temp", ["RegexWatermark"] = "^[0-9].*",
        ["OpenApp"] = "Открыть", ["Exit"] = "Выход", ["SuccessBadge"] = "OK", ["IgnoredBadge"] = "IGN", ["ErrorBadge"] = "ERR", ["SmtpEmailHeader"] = "SMTP / Email",
        ["JobStarted"] = "Задача '{0}' запущена.", ["RetryMode"] = " [RETRY]", ["PreScriptFailed"] = "Pre-script ошибка.",
        ["SourceNotFound"] = "Не найден путь: {0}", ["JobError"] = "Ошибка задачи", ["JobFailedSourceNotFound"] = "Ошибка: Путь отсутствует.",
        ["StartingFile"] = "Старт: {0}", ["SuccessFile"] = "ОК: {0}", ["ErrorFile"] = "ОШИБКА: {0}",
        ["FilterIgnored"] = "Фильтр.", ["JobFinished"] = "Готово: {0}", ["JobCompleted"] = "Успех",
        ["FilesFailed"] = "Ошибок: {0}.", ["UserCancelled"] = "Отмена пользователем.", ["UserCancelledOp"] = "Отменено.",
        ["CriticalError"] = "Крит. ошибка", ["CriticalErrorJob"] = "Ошибка в '{0}': {1}", ["PostScriptFailed"] = "Post-script ошибка.",
        ["IntegrityFailed"] = "Ошибка целостности.", ["ZipCreationFailed"] = "Ошибка ZIP: {0}",
        ["RetentionOldDeleted"] = "Удалено по сроку {0}", ["RetentionExpiredDeleted"] = "Удалено: истек срок {0}",
        ["RetentionFailed"] = "Ошибка очистки: {0}", ["SmartSyncDetail"] = "SmartSync", ["StartingJobLog"] = "Старт: {0}",
        ["PreviewErrorSourceNotFound"] = "Нет пути: {0}", ["PreviewIgnored"] = "[SKIP] {0}",
        ["PreviewIgnoredSmartSync"] = "[SYNC] {0}", ["PreviewIgnoredConflict"] = "[CONF] {0}",
        ["PreviewOverwrite"] = "[OVER] {0}", ["PreviewRename"] = "[REN] {0}", ["PreviewCopy"] = "[COPY] {0}", ["PreviewMove"] = "[MOVE] {0}",
        ["RamUsageFormat"] = "{0} ГБ / {1} GB", ["TimeSavedFormat"] = "{0}ч {1}м", ["SchedulerLoopError"] = "Ошибка планировщика: {0}",
        ["DailyMaintenanceSuccess"] = "Обслуживание завершено.", ["DailyMaintenanceError"] = "Ошибка обслуживания: {0}",
        ["DailySummaryName"] = "Отчет за день", ["InXDays"] = "через {0} д.", ["InXHours"] = "через {0} ч.", ["InXMinutes"] = "через {0} мин.",
        ["SupportTitle"] = "Поддержка разработчика", ["SupportPixLabel"] = "Pix ключ (Бразилия)", ["SupportCoffeeLabel"] = "Купить кофе", ["SupportCopyPix"] = "Копировать",
        ["Donate"] = "Донат"
    };

    private Dictionary<string, string> GetPtBrDefaults() => new() {
        ["Dashboard"] = "Painel", ["Jobs"] = "Fila", ["History"] = "Histórico", ["Settings"] = "Configurações", 
        ["Greeting"] = "Boa noite!", ["ActiveAuto"] = "Você tem {0} automações ativas.",
        ["Total"] = "Total", ["Running"] = "Rodando", ["Completed"] = "Concluído", ["Failed"] = "Falhou",
        ["RecentActivity"] = "Atividade Recente", ["QuickActions"] = "Ações Rápidas", ["ErrorAudit"] = "Auditoria de Erros",
        ["NewJob"] = "Nova Tarefa", ["NewAuto"] = "Nova Automatização", ["ViewHistory"] = "Ver Histórico",
        ["NoErrors"] = "Nenhum erro registrado no sistema.", ["OpenErrorFolder"] = "Abrir Pasta de Erros", ["ClearLogs"] = "Limpar Logs",
        ["SaveSettings"] = "Salvar Configurações", ["Appearance"] = "Aparência e Localização", ["SystemTheme"] = "Tema do Aplicativo", ["Language"] = "Idioma (Cultura)",
        ["Behavior"] = "Comportamento do Sistema", ["ShowNotifications"] = "Exibir notificações na área de trabalho (Toasts)", ["StartAtStartup"] = "Iniciar FolderFlow automaticamente com o sistema (Background)",
        ["Support"] = "Suporte e Doação", ["SupportDesc"] = "Se o FolderFlow tem sido útil para você, considere apoiar o desenvolvedor!", ["BuyCoffee"] = "Apoie o desenvolvedor",
        ["Add"] = "Adicionar", ["Hotfolder"] = "Hotfolder", ["Run"] = "Rodar", ["StopAll"] = "Parar Tudo", ["Delete"] = "Excluir",
        ["QueueManagement"] = "GERENCIAR FILA DE OPERAÇÕES", ["OpLog"] = "Log de Operações", ["ManualOp"] = "Operações Manuais", ["Automation"] = "Automatações",
        ["HistoryHeader"] = "HISTÓRICO DE AUTOMAÇÃO", ["Update"] = "Atualizar", ["SearchHistory"] = "Pesquisar arquivo ou destino...",
        ["NoLogs"] = "Nenhum log encontrado.",
        ["Copy"] = "Copiar", ["Move"] = "Mover", ["Skip"] = "Pular", ["Overwrite"] = "Sobrescrever", ["Rename"] = "Renomear",
        ["None"] = "Nenhum", ["Interval"] = "Intervalo", ["Daily"] = "Diário", ["Weekly"] = "Semanal",
        ["RealTime"] = "Tempo Real", ["Polling"] = "Varredura (Polling)", ["OnSuccess"] = "No Sucesso", ["OnError"] = "No Erro", ["OnBoth"] = "Em Ambos",
        ["ControlPanel"] = "Painel de Controle", ["SystemOverview"] = "Visão geral do sistema e métricas de desempenho",
        ["Type"] = "Tipo:", ["Task"] = "Tarefa:", ["SystemHealth"] = "SAÚDE DO SISTEMA", ["TotalVolume"] = "VOLUME TOTAL",
        ["ProcessedData"] = "Dados processados", ["TimeSaved"] = "TEMPO ECONOMIZADO", ["AutomatedWork"] = "Trabalho automatizado",
        ["Files"] = "ARQUIVOS", ["DirectCopy"] = "Cópia Direta", ["WatchFolder"] = "Watch Folder", ["PauseAll"] = "Pausar Tudo",
        ["RecentActivities"] = "ATIVIDADES RECENTES", ["UpcomingRadar"] = "RADAR DE AGENDAMENTOS", ["SystemMonitor"] = "MONITOR DO SISTEMA",
        ["CopySpeed"] = "Velocidade de Cópia", ["Processor"] = "Processador (CPU)", ["Ram"] = "Memória RAM", ["CloudSync"] = "Sincronização Nuvem",
        ["All"] = "Todos", ["Excellent"] = "Excelente", ["Good"] = "Bom", ["Attention"] = "Atenção", ["Critical"] = "Crítico",
        ["NoData"] = "Sem dados", ["Downloading"] = "Baixando...", ["Synced"] = "Sincronizado", ["GlobalPaused"] = "Processamento global pausado.", ["GlobalResumed"] = "Processamento global retomado.",
        ["AdvancedSettings"] = "Configurações Avançadas", ["SettingsDesc"] = "Controle total sobre performance, integração e visual",
        ["EnginePerformance"] = "MOTOR E PERFORMANCE", ["Parallelism"] = "Paralelismo (Threads)", ["ParallelismDesc"] = "Número de arquivos processados simultaneamente",
        ["ProcessPriority"] = "Prioridade do Processo", ["PriorityDesc"] = "Impacto do software no desempenho do Windows",
        ["GlobalBandwidth"] = "Limite de Banda Global", ["BandwidthDesc"] = "Limite a velocidade de cópia para não saturar a rede (0 = Ilimitado)",
        ["SecurityPrivacy"] = "SEGURANÇA E PRIVACIDADE", ["AccessPin"] = "PIN de Acesso", ["PinWatermark"] = "Defina um PIN numérico",
        ["PinDesc"] = "Proteja a interface com uma senha", ["LockOnMinimize"] = "Bloquear ao minimizar",
        ["EncryptionKey"] = "Chave de Criptografia Mestra", ["EncryptionKeyWatermark"] = "Sua chave global", ["EncryptionDesc"] = "Sugerida como padrão para novos jobs seguros",
        ["ExternalNotifications"] = "NOTIFICAÇÕES EXTERNAS (WEBHOOKS & SMTP)", ["WebhookUrl"] = "Webhook URL",
        ["WebhookUrlWatermark"] = "https://discord.com/api/webhooks/...", ["SmtpServer"] = "Servidor SMTP",
        ["SmtpHostWatermark"] = "smtp.gmail.com", ["Port"] = "Porta", ["DestEmail"] = "E-mail Destino",
        ["DestEmailWatermark"] = "alertas@email.com", ["EnableSmtp"] = "Ativar E-mail", ["NotifyOnSuccess"] = "Notificar Sucessos", ["NotifyOnError"] = "Notificar Erros",
        ["AutomationSystem"] = "AUTOMAÇÃO E SISTEMA", ["StartWithWindows"] = "Iniciar com o Windows", ["StartWithWindowsDesc"] = "Abrir automaticamente ao ligar o computador",
        ["CloseToTray"] = "Fechar para a Bandeja (System Tray)", ["CloseToTrayDesc"] = "Manter o FolderFlow rodando ao fechar a janela principal",
        ["MaintenanceData"] = "MANUTENÇÃO E DADOS", ["LogRetention"] = "Retenção de Logs", ["LogRetentionDesc"] = "Dias para manter os registros no histórico",
        ["DatabaseSize"] = "Tamanho: {0}", ["ClearNow"] = "Limpar Agora", ["OptimizeDatabase"] = "Otimizar Banco",
        ["VisualLanguage"] = "VISUAL E IDIOMA", ["InterfaceTheme"] = "Tema da Interface", ["SystemLanguage"] = "Idioma do Sistema",
        ["GlassOpacity"] = "Opacidade do Efeito Vidro (Mica/Acrylic)", ["DiscardChanges"] = "Descartar Alterações",
        ["ConfigureAutomation"] = "Configurar Automação", ["AutomationRules"] = "Defina as regras, filtros e destinos para o fluxo de dados",
        ["General"] = "Geral", ["Filters"] = "Filtros", ["Schedule"] = "Agenda", ["Security"] = "Segurança", ["Advanced"] = "Avançado",
        ["JobName"] = "Nome da Tarefa", ["SourcePath"] = "Caminho de Origem", ["DestPath"] = "Caminho de Destino", ["WorkMode"] = "Modo de Trabalho",
        ["FileConflicts"] = "Conflitos de Arquivo", ["IncludeExtensions"] = "Extensões (Inclusão)", ["IgnorePatterns"] = "Padrões a Ignorar",
        ["RegexExpression"] = "Expressão Regular (REGEX)", ["WatcherConfigs"] = "CONFIGURAÇÕES DO WATCHER", ["DetectionMethod"] = "Método de Detecção",
        ["StabilizationTime"] = "Tempo de Estabilização", ["ScheduleType"] = "Tipo de Agendamento", ["RepeatOnDays"] = "Repetir nos dias:",
        ["Sun"] = "D", ["Mon"] = "S", ["Tue"] = "T", ["Wed"] = "Q", ["Thu"] = "Q", ["Fri"] = "S", ["Sat"] = "S",
        ["ExecutionTime"] = "Horário de Execução", ["DataProtection"] = "PROTEÇÃO DE DADOS (AES-256)", ["EncryptionKeyLabel"] = "Chave de Criptografia",
        ["JobWebhooks"] = "WEBHOOKS DE TAREFA", ["SimulationSummary"] = "RESUMO DA SIMULAÇÃO", ["FilesIdentified"] = "Arquivos Identificados",
        ["DataVolume"] = "Volume de Dados", ["SimulationSuccess"] = "A simulação foi concluída com sucesso.",
        ["Simulate"] = "Simular (Preview)", ["NewDirectCopyTitle"] = "Nova Cópia Direta", ["NewWatchFolderTitle"] = "Nova Watch Folder",
        ["Save"] = "Salvar", ["Cancel"] = "Cancelar", ["ProcessSubfolders"] = "Processar Subpastas", ["SmartSyncLabel"] = "Smart Sync (Pular idênticos)",
        ["VerifyHashLabel"] = "Validar Integridade Pós-Cópia (Hash)", ["ClearPreview"] = "Limpar Preview", ["SaveAutomation"] = "Salvar Automação",
        ["AuditHistory"] = "Auditoria e Histórico", ["ForenseTracking"] = "Rastreamento forense de todas as operações de arquivos",
        ["ExportCsv"] = "Exportar CSV", ["OpDetails"] = "DETALHES DA OPERAÇÃO", ["File"] = "Arquivo", ["Destination"] = "Destino",
        ["Size"] = "Tamanho", ["Duration"] = "Duração", ["EngineLogs"] = "Logs do Engine", ["SelectEntry"] = "Selecione uma entrada para ver os detalhes",
        ["SearchWatermark"] = "Buscar arquivo ou detalhe...", ["Time"] = "Hora", ["Origin"] = "Origem", ["OpenSourceFolder"] = "Abrir Pasta de Origem",
        ["OpenDestFolder"] = "Abrir Pasta de Destino", ["ClearHistory"] = "Limpar Todo o Histórico", ["Copied"] = "COPIADO",
        ["Moved"] = "MOVIDO", ["Ignored"] = "IGNORADO", ["FailedStatus"] = "FALHA", ["Zipped"] = "ZIPADO", ["Cancelled"] = "CANCELADO",
        ["SidebarAutomation"] = "Automação", ["SidebarAudit"] = "Auditoria", ["AutomationHub"] = "Central de Automação",
        ["ActiveBadge"] = "{0} Ativos", ["QueueReady"] = "Fila Pronta", ["AutomationDesc"] = "Orquestre, monitore e gerencie seus fluxos de dados em um único lugar",
        ["StopAllTip"] = "Encerrar Todas as Atividades", ["Active"] = "Ativos", ["Pending"] = "Pendentes", ["Watchers"] = "Watchers",
        ["SearchJobsWatermark"] = "Buscar por nome ou caminho...", ["SelectedCount"] = "{0} selecionados", ["RealTimeProcessing"] = "PROCESSAMENTO EM TEMPO REAL",
        ["ConfiguredTasks"] = "SUAS TAREFAS CONFIGURADAS", ["RunTip"] = "Iniciar", ["StopTip"] = "Parar", ["EditTip"] = "Editar", ["DeleteTip"] = "Excluir",
        ["OriginLabel"] = "ORIGEM:", ["DestLabel"] = "DESTINO:", ["Frequency"] = "FREQUÊNCIA", ["SpeedLabel"] = "VELOCIDADE", ["LiveTerminal"] = "TERMINAL LIVE",
        ["Processing"] = "Processando...", ["Idle"] = "Ocioso", ["Queued"] = "Na Fila...", ["NewDirectCopy"] = "Nova Cópia Direta",
        ["NewWatchFolder"] = "Nova Watch Folder", ["JobNameWatermark"] = "Ex: Backup de Documentos", ["Browse"] = "Procurar",
        ["ExtensionsWatermark"] = ".jpg, .png, .pdf", ["IgnoreWatermark"] = "node_modules, temp", ["RegexWatermark"] = "Ex: ^[0-9]{4}.*",
        ["OpenApp"] = "Abrir FolderFlow", ["Exit"] = "Sair", ["SuccessBadge"] = "OK", ["IgnoredBadge"] = "IGN", ["ErrorBadge"] = "ERR", ["SmtpEmailHeader"] = "SMTP / E-mail",
        ["JobStarted"] = "Tarefa '{0}' iniciada.", ["RetryMode"] = " [MODO RETRY]", ["PreScriptFailed"] = "Pre-script falhou. A tarefa continuará, mas verifique os logs.",
        ["SourceNotFound"] = "Pasta de origem não encontrada: {0}", ["JobError"] = "Erro no Job", ["JobFailedSourceNotFound"] = "Tarefa '{0}' falhou: Origem não encontrada.",
        ["StartingFile"] = "Iniciando: {0}", ["SuccessFile"] = "Sucesso: {0}", ["ErrorFile"] = "ERRO: {0}",
        ["FilterIgnored"] = "Filtro de exclusão ou critérios de data/tamanho.", ["JobFinished"] = "Job Finalizado: {0}", ["JobCompleted"] = "Job Concluído",
        ["FilesFailed"] = "{0} arquivos falharam.", ["UserCancelled"] = "Tarefa '{0}' cancelada pelo usuário.", ["UserCancelledOp"] = "Operação cancelada pelo usuário.",
        ["CriticalError"] = "Erro Crítico", ["CriticalErrorJob"] = "Erro crítico na tarefa '{0}': {1}", ["PostScriptFailed"] = "Post-script falhou.",
        ["IntegrityFailed"] = "Falha na verificação de integridade (Hash mismatch).", ["ZipCreationFailed"] = "Falha ao criar arquivo ZIP: {0}",
        ["RetentionOldDeleted"] = "Retenção: Arquivo antigo excluído '{0}'.", ["RetentionExpiredDeleted"] = "Retenção: Arquivo expirado excluído '{0}'.",
        ["RetentionFailed"] = "Falha ao aplicar política de retenção: {0}", ["SmartSyncDetail"] = "SmartSync", ["StartingJobLog"] = "Iniciando Job: {0}",
        ["PreviewErrorSourceNotFound"] = "[ERRO] Diretório de origem não encontrado: {0}", ["PreviewIgnored"] = "[IGNORADO] {0}",
        ["PreviewIgnoredSmartSync"] = "[IGNORADO - SMART SYNC] {0}", ["PreviewIgnoredConflict"] = "[IGNORADO - CONFLITO] {0}",
        ["PreviewOverwrite"] = "[SOBRESCREVER] {0}", ["PreviewRename"] = "[RENOMEAR] {0}", ["PreviewCopy"] = "[COPIAR] {0}", ["PreviewMove"] = "[MOVER] {0}",
        ["RamUsageFormat"] = "{0} GB / {1} GB", ["TimeSavedFormat"] = "{0}h {1}m", ["SchedulerLoopError"] = "Erro no loop do agendador: {0}",
        ["DailyMaintenanceSuccess"] = "Manutenção diária concluída com sucesso.", ["DailyMaintenanceError"] = "Erro na manutenção diária: {0}",
        ["DailySummaryName"] = "Resumo Diário", ["InXDays"] = "em {0} dias", ["InXHours"] = "em {0}h", ["InXMinutes"] = "em {0}min",
        ["SupportTitle"] = "Apoie o Desenvolvedor", ["SupportPixLabel"] = "Chave Pix (Brasil)", ["SupportCoffeeLabel"] = "Buy Me a Coffee", ["SupportCopyPix"] = "Copiar Chave",
        ["Donate"] = "Doar"
    };

    private void CreateDefaultFile(string path, string culture)
    {
        var dict = GetDefaults(culture);
        SaveTranslations(path, dict);
    }
}
