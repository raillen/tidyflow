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
        var dict = culture switch
        {
            "en-US" => new Dictionary<string, string> { 
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

                // Enums
                ["Copy"] = "Copy", ["Move"] = "Move",
                ["Skip"] = "Skip", ["Overwrite"] = "Overwrite", ["Rename"] = "Rename",
                ["None"] = "None", ["Interval"] = "Interval", ["Daily"] = "Daily", ["Weekly"] = "Weekly",
                ["RealTime"] = "Real-Time", ["Polling"] = "Polling",
                ["OnSuccess"] = "On Success", ["OnError"] = "On Error", ["OnBoth"] = "On Both",

                // Dashboard
                ["ControlPanel"] = "Control Panel",
                ["SystemOverview"] = "System overview and performance metrics",
                ["Type"] = "Type:",
                ["Task"] = "Task:",
                ["SystemHealth"] = "SYSTEM HEALTH",
                ["TotalVolume"] = "TOTAL VOLUME",
                ["ProcessedData"] = "Processed data",
                ["TimeSaved"] = "TIME SAVED",
                ["AutomatedWork"] = "Automated work",
                ["Files"] = "FILES",
                ["DirectCopy"] = "Direct Copy",
                ["WatchFolder"] = "Watch Folder",
                ["PauseAll"] = "Pause All",
                ["RecentActivities"] = "RECENT ACTIVITIES",
                ["UpcomingRadar"] = "SCHEDULE RADAR",
                ["SystemMonitor"] = "SYSTEM MONITOR",
                ["CopySpeed"] = "Copy Speed",
                ["Processor"] = "Processor (CPU)",
                ["Ram"] = "RAM Memory",
                ["CloudSync"] = "Cloud Sync",
                ["All"] = "All",
                ["Excellent"] = "Excellent",
                ["Good"] = "Good",
                ["Attention"] = "Attention",
                ["Critical"] = "Critical",
                ["NoData"] = "No data",
                ["Downloading"] = "Downloading...",
                ["Synced"] = "Synced",
                ["GlobalPaused"] = "Global processing paused.",
                ["GlobalResumed"] = "Global processing resumed.",

                // Settings
                ["AdvancedSettings"] = "Advanced Settings",
                ["SettingsDesc"] = "Full control over performance, integration and visuals",
                ["EnginePerformance"] = "ENGINE & PERFORMANCE",
                ["Parallelism"] = "Parallelism (Threads)",
                ["ParallelismDesc"] = "Number of files processed simultaneously",
                ["ProcessPriority"] = "Process Priority",
                ["PriorityDesc"] = "Software impact on Windows performance",
                ["GlobalBandwidth"] = "Global Bandwidth Limit",
                ["BandwidthDesc"] = "Limit copy speed to not saturate network (0 = Unlimited)",
                ["SecurityPrivacy"] = "SECURITY & PRIVACY",
                ["AccessPin"] = "Access PIN",
                ["PinWatermark"] = "Set a numeric PIN",
                ["PinDesc"] = "Protect interface with a password",
                ["LockOnMinimize"] = "Lock on minimize",
                ["EncryptionKey"] = "Master Encryption Key",
                ["EncryptionKeyWatermark"] = "Your global key",
                ["EncryptionDesc"] = "Suggested as default for new secure jobs",
                ["ExternalNotifications"] = "EXTERNAL NOTIFICATIONS (WEBHOOKS & SMTP)",
                ["WebhookUrl"] = "Webhook URL",
                ["WebhookUrlWatermark"] = "https://discord.com/api/webhooks/...",
                ["Type"] = "Type",
                ["SmtpServer"] = "SMTP Server",
                ["SmtpHostWatermark"] = "smtp.gmail.com",
                ["Port"] = "Port",
                ["DestEmail"] = "Destination Email",
                ["DestEmailWatermark"] = "alerts@email.com",
                ["EnableSmtp"] = "Enable Email",
                ["NotifyOnSuccess"] = "Notify Success",
                ["NotifyOnError"] = "Notify Errors",
                ["AutomationSystem"] = "AUTOMATION & SYSTEM",
                ["StartWithWindows"] = "Start with Windows",
                ["StartWithWindowsDesc"] = "Open automatically on computer startup",
                ["CloseToTray"] = "Close to Tray (System Tray)",
                ["CloseToTrayDesc"] = "Keep FolderFlow running when closing main window",
                ["MaintenanceData"] = "MAINTENANCE & DATA",
                ["LogRetention"] = "Log Retention",
                ["LogRetentionDesc"] = "Days to keep records in history",
                ["DatabaseSize"] = "Size: {0}",
                ["ClearNow"] = "Clear Now",
                ["OptimizeDatabase"] = "Optimize Database",
                ["VisualLanguage"] = "VISUAL & LANGUAGE",
                ["InterfaceTheme"] = "Interface Theme",
                ["SystemLanguage"] = "System Language",
                ["GlassOpacity"] = "Glass Effect Opacity (Mica/Acrylic)",
                ["DiscardChanges"] = "Discard Changes",
                ["SaveSettings"] = "Save Settings",

                // Job Editor
                ["ConfigureAutomation"] = "Configure Automation",
                ["AutomationRules"] = "Define rules, filters and destinations for data flow",
                ["General"] = "General",
                ["Filters"] = "Filters",
                ["Schedule"] = "Schedule",
                ["Security"] = "Security",
                ["Advanced"] = "Advanced",
                ["JobName"] = "Task Name",
                ["SourcePath"] = "Source Path",
                ["DestPath"] = "Destination Path",
                ["WorkMode"] = "Work Mode",
                ["FileConflicts"] = "File Conflicts",
                ["IncludeExtensions"] = "Extensions (Include)",
                ["IgnorePatterns"] = "Patterns to Ignore",
                ["RegexExpression"] = "Regular Expression (REGEX)",
                ["WatcherConfigs"] = "WATCHER CONFIGS",
                ["DetectionMethod"] = "Detection Method",
                ["StabilizationTime"] = "Stabilization Time",
                ["ScheduleType"] = "Schedule Type",
                ["RepeatOnDays"] = "Repeat on days:",
                ["ExecutionTime"] = "Execution Time",
                ["DataProtection"] = "DATA PROTECTION (AES-256)",
                ["EncryptionKeyLabel"] = "Encryption Key",
                ["JobWebhooks"] = "TASK WEBHOOKS",
                ["SimulationSummary"] = "SIMULATION SUMMARY",
                ["FilesIdentified"] = "Files Identified",
                ["DataVolume"] = "Data Volume",
                ["SimulationSuccess"] = "Simulation completed successfully. Filters captured the above data.",
                ["Simulate"] = "Simulate (Preview)",
                ["NewDirectCopyTitle"] = "New Direct Copy",
                ["NewWatchFolderTitle"] = "New Watch Folder",
                ["Save"] = "Save",
                ["Cancel"] = "Cancel",

                // History / Audit
                ["AuditHistory"] = "Audit & History",
                ["ForenseTracking"] = "Forensic tracking of all file operations",
                ["ExportCsv"] = "Export CSV",
                ["OpDetails"] = "OPERATION DETAILS",
                ["File"] = "File",
                ["Destination"] = "Destination",
                ["Size"] = "Size",
                ["Duration"] = "Duration",
                ["EngineLogs"] = "Engine Logs",
                ["SelectEntry"] = "Select an entry to see details",
                ["SearchHistory"] = "Search file or destination...",
                ["SearchWatermark"] = "Search file or detail...",
                ["Time"] = "Time",
                ["Task"] = "Task",
                ["Origin"] = "Origin",
                ["OpenSourceFolder"] = "Open Source Folder",
                ["OpenDestFolder"] = "Open Destination Folder",
                ["ClearHistory"] = "Clear All History",
                ["Copied"] = "COPIED",
                ["Moved"] = "MOVED",
                ["Ignored"] = "IGNORED",
                ["FailedStatus"] = "FAILED",
                ["Zipped"] = "ZIPPED",
                ["Cancelled"] = "CANCELLED",

                // Sidebar
                ["SidebarAutomation"] = "Automation",
                ["SidebarAudit"] = "Audit",

                // Automation View
                ["AutomationHub"] = "Automation Hub",
                ["ActiveBadge"] = "{0} Active",
                ["QueueReady"] = "Queue Ready",
                ["AutomationDesc"] = "Orchestrate, monitor and manage your data flows in one place",
                ["StopAllTip"] = "Terminate All Activities",
                ["Active"] = "Active",
                ["Pending"] = "Pending",
                ["Watchers"] = "Watchers",
                ["SearchJobsWatermark"] = "Search by name or path...",
                ["SelectedCount"] = "{0} selected",
                ["RealTimeProcessing"] = "REAL-TIME PROCESSING",
                ["ConfiguredTasks"] = "YOUR CONFIGURED TASKS",
                ["RunTip"] = "Run",
                ["StopTip"] = "Stop",
                ["EditTip"] = "Edit",
                ["DeleteTip"] = "Delete",
                ["OriginLabel"] = "ORIGIN:",
                ["DestLabel"] = "DESTINATION:",
                ["Frequency"] = "FREQUENCY",
                ["SpeedLabel"] = "SPEED",
                ["LiveTerminal"] = "LIVE TERMINAL",
                ["Processing"] = "Processing...",
                ["Idle"] = "Idle",
                ["Queued"] = "Queued...",
                ["NewDirectCopy"] = "New Direct Copy",
                ["NewWatchFolder"] = "New Watch Folder",
                ["JobNameWatermark"] = "Ex: Documents Backup",
                ["Browse"] = "Browse",
                ["ExtensionsWatermark"] = ".jpg, .png, .pdf",
                ["IgnoreWatermark"] = "node_modules, temp",
                ["RegexWatermark"] = "Ex: ^[0-9]{4}.*",
                ["ProcessSubfolders"] = "Process Subfolders",
                ["SmartSyncLabel"] = "Smart Sync (Skip identical)",
                ["EncryptionKeyWatermarkEditor"] = "Key to protect files at destination",
                ["VerifyHashLabel"] = "Verify Integrity (Hash)",
                ["WebhookUrlWatermarkEditor"] = "Specific URL for this task (Optional)",
                ["ClearPreview"] = "Clear Preview",
                ["SaveAutomation"] = "Save Automation",
                ["Sun"] = "S", ["Mon"] = "M", ["Tue"] = "T", ["Wed"] = "W", ["Thu"] = "T", ["Fri"] = "F", ["Sat"] = "S",
                ["OpenApp"] = "Open FolderFlow",
                ["Exit"] = "Exit",
                ["SuccessBadge"] = "OK",
                ["IgnoredBadge"] = "IGN",
                ["ErrorBadge"] = "ERR",
                ["SmtpEmailHeader"] = "SMTP / Email",

                // Engine
                ["JobStarted"] = "Task '{0}' started.",
                ["RetryMode"] = " [RETRY MODE]",
                ["PreScriptFailed"] = "Pre-script failed. Task will continue, check logs.",
                ["SourceNotFound"] = "Source folder not found: {0}",
                ["JobError"] = "Job Error",
                ["JobFailedSourceNotFound"] = "Task '{0}' failed: Source not found.",
                ["StartingFile"] = "Starting: {0}",
                ["SuccessFile"] = "Success: {0}",
                ["ErrorFile"] = "ERROR: {0} - {1}",
                ["FilterIgnored"] = "Exclusion filter or date/size criteria.",
                ["JobFinished"] = "Job Finished: {0}. {1} processed.",
                ["JobCompleted"] = "Job Completed",
                ["FilesFailed"] = "{0} files failed.",
                ["UserCancelled"] = "Task '{0}' cancelled by user.",
                ["UserCancelledOp"] = "Operation cancelled by user.",
                ["CriticalError"] = "Critical Error",
                ["CriticalErrorJob"] = "Critical error in task '{0}': {1}",
                ["PostScriptFailed"] = "Post-script failed.",
                ["IntegrityFailed"] = "Integrity check failed (Hash mismatch).",
                ["ZipCreationFailed"] = "Failed to create ZIP file: {0}",
                ["RetentionOldDeleted"] = "Retention: Old file deleted '{0}'.",
                ["RetentionExpiredDeleted"] = "Retention: Expired file deleted '{0}'.",
                ["RetentionFailed"] = "Failed to apply retention policy: {0}",
                ["SmartSyncDetail"] = "SmartSync",
                ["StartingJobLog"] = "Starting Job: {0}",

                // Preview
                ["PreviewErrorSourceNotFound"] = "[ERROR] Source directory not found: {0}",
                ["PreviewIgnored"] = "[IGNORED] {0}",
                ["PreviewIgnoredSmartSync"] = "[IGNORED - SMART SYNC] {0}",
                ["PreviewIgnoredConflict"] = "[IGNORED - CONFLICT] {0}",
                ["PreviewOverwrite"] = "[OVERWRITE] {0}",
                ["PreviewRename"] = "[RENAME] {0}",
                ["PreviewCopy"] = "[COPY] {0}",
                ["PreviewMove"] = "[MOVE] {0}",

                // Formatting
                ["RamUsageFormat"] = "{0} GB / {1} GB",
                ["TimeSavedFormat"] = "{0}h {1}m",

                // Scheduler
                ["SchedulerLoopError"] = "Error in scheduler loop: {0}",
                ["DailyMaintenanceSuccess"] = "Daily maintenance completed successfully.",
                ["DailyMaintenanceError"] = "Error in daily maintenance: {0}",
                ["DailySummaryName"] = "Daily Summary",
                ["InXDays"] = "in {0} days",
                ["InXHours"] = "in {0}h",
                ["InXMinutes"] = "in {0}min",
            },
            _ => new Dictionary<string, string> { 
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
                
                // Enums
                ["Copy"] = "Copiar", ["Move"] = "Mover",
                ["Skip"] = "Pular", ["Overwrite"] = "Sobrescrever", ["Rename"] = "Renomear",
                ["None"] = "Nenhum", ["Interval"] = "Intervalo", ["Daily"] = "Diário", ["Weekly"] = "Semanal",
                ["RealTime"] = "Tempo Real", ["Polling"] = "Varredura (Polling)",
                ["OnSuccess"] = "No Sucesso", ["OnError"] = "No Erro", ["OnBoth"] = "Em Ambos",

                // Dashboard
                ["ControlPanel"] = "Painel de Controle",
                ["SystemOverview"] = "Visão geral do sistema e métricas de desempenho",
                ["Type"] = "Tipo:",
                ["Task"] = "Tarefa:",
                ["SystemHealth"] = "SAÚDE DO SISTEMA",
                ["TotalVolume"] = "VOLUME TOTAL",
                ["ProcessedData"] = "Dados processados",
                ["TimeSaved"] = "TEMPO ECONOMIZADO",
                ["AutomatedWork"] = "Trabalho automatizado",
                ["Files"] = "ARQUIVOS",
                ["DirectCopy"] = "Cópia Direta",
                ["WatchFolder"] = "Watch Folder",
                ["PauseAll"] = "Pausar Tudo",
                ["RecentActivities"] = "ATIVIDADES RECENTES",
                ["UpcomingRadar"] = "RADAR DE AGENDAMENTOS",
                ["SystemMonitor"] = "MONITOR DO SISTEMA",
                ["CopySpeed"] = "Velocidade de Cópia",
                ["Processor"] = "Processador (CPU)",
                ["Ram"] = "Memória RAM",
                ["CloudSync"] = "Sincronização Nuvem",
                ["All"] = "Todos",
                ["Excellent"] = "Excelente",
                ["Good"] = "Bom",
                ["Attention"] = "Atenção",
                ["Critical"] = "Crítico",
                ["NoData"] = "Sem dados",
                ["Downloading"] = "Baixando...",
                ["Synced"] = "Sincronizado",
                ["GlobalPaused"] = "Processamento global pausado.",
                ["GlobalResumed"] = "Processamento global retomado.",

                // Settings
                ["AdvancedSettings"] = "Configurações Avançadas",
                ["SettingsDesc"] = "Controle total sobre performance, integração e visual",
                ["EnginePerformance"] = "MOTOR E PERFORMANCE",
                ["Parallelism"] = "Paralelismo (Threads)",
                ["ParallelismDesc"] = "Número de arquivos processados simultaneamente",
                ["ProcessPriority"] = "Prioridade do Processo",
                ["PriorityDesc"] = "Impacto do software no desempenho do Windows",
                ["GlobalBandwidth"] = "Limite de Banda Global",
                ["BandwidthDesc"] = "Limite a velocidade de cópia para não saturar a rede (0 = Ilimitado)",
                ["SecurityPrivacy"] = "SEGURANÇA E PRIVACIDADE",
                ["AccessPin"] = "PIN de Acesso",
                ["PinWatermark"] = "Defina um PIN numérico",
                ["PinDesc"] = "Proteja a interface com uma senha",
                ["LockOnMinimize"] = "Bloquear ao minimizar",
                ["EncryptionKey"] = "Chave de Criptografia Mestra",
                ["EncryptionKeyWatermark"] = "Sua chave global",
                ["EncryptionDesc"] = "Sugerida como padrão para novos jobs seguros",
                ["ExternalNotifications"] = "NOTIFICAÇÕES EXTERNAS (WEBHOOKS & SMTP)",
                ["WebhookUrl"] = "Webhook URL",
                ["WebhookUrlWatermark"] = "https://discord.com/api/webhooks/...",
                ["Type"] = "Tipo",
                ["SmtpServer"] = "Servidor SMTP",
                ["SmtpHostWatermark"] = "smtp.gmail.com",
                ["Port"] = "Porta",
                ["DestEmail"] = "E-mail Destino",
                ["DestEmailWatermark"] = "alertas@email.com",
                ["EnableSmtp"] = "Ativar E-mail",
                ["NotifyOnSuccess"] = "Notificar Sucessos",
                ["NotifyOnError"] = "Notificar Erros",
                ["AutomationSystem"] = "AUTOMAÇÃO E SISTEMA",
                ["StartWithWindows"] = "Iniciar com o Windows",
                ["StartWithWindowsDesc"] = "Abrir automaticamente ao ligar o computador",
                ["CloseToTray"] = "Fechar para a Bandeja (System Tray)",
                ["CloseToTrayDesc"] = "Manter o FolderFlow rodando ao fechar a janela principal",
                ["MaintenanceData"] = "MANUTENÇÃO E DADOS",
                ["LogRetention"] = "Retenção de Logs",
                ["LogRetentionDesc"] = "Dias para manter os registros no histórico",
                ["DatabaseSize"] = "Tamanho: {0}",
                ["ClearNow"] = "Limpar Agora",
                ["OptimizeDatabase"] = "Otimizar Banco",
                ["VisualLanguage"] = "VISUAL E IDIOMA",
                ["InterfaceTheme"] = "Tema da Interface",
                ["SystemLanguage"] = "Idioma do Sistema",
                ["GlassOpacity"] = "Opacidade do Efeito Vidro (Mica/Acrylic)",
                ["DiscardChanges"] = "Descartar Alterações",
                ["SaveSettings"] = "Salvar Configurações",

                // Job Editor
                ["ConfigureAutomation"] = "Configurar Automação",
                ["AutomationRules"] = "Defina as regras, filtros e destinos para o fluxo de dados",
                ["General"] = "Geral",
                ["Filters"] = "Filtros",
                ["Schedule"] = "Agenda",
                ["Security"] = "Segurança",
                ["Advanced"] = "Avançado",
                ["JobName"] = "Nome da Tarefa",
                ["SourcePath"] = "Caminho de Origem",
                ["DestPath"] = "Caminho de Destino",
                ["WorkMode"] = "Modo de Trabalho",
                ["FileConflicts"] = "Conflitos de Arquivo",
                ["IncludeExtensions"] = "Extensões (Inclusão)",
                ["IgnorePatterns"] = "Padrões a Ignorar",
                ["RegexExpression"] = "Expressão Regular (REGEX)",
                ["WatcherConfigs"] = "CONFIGURAÇÕES DO WATCHER",
                ["DetectionMethod"] = "Método de Detecção",
                ["StabilizationTime"] = "Tempo de Estabilização",
                ["ScheduleType"] = "Tipo de Agendamento",
                ["RepeatOnDays"] = "Repetir nos dias:",
                ["ExecutionTime"] = "Horário de Execução",
                ["DataProtection"] = "PROTEÇÃO DE DADOS (AES-256)",
                ["EncryptionKeyLabel"] = "Chave de Criptografia",
                ["JobWebhooks"] = "WEBHOOKS DE TAREFA",
                ["SimulationSummary"] = "RESUMO DA SIMULAÇÃO",
                ["FilesIdentified"] = "Arquivos Identificados",
                ["DataVolume"] = "Volume de Dados",
                ["SimulationSuccess"] = "A simulação foi concluída com sucesso. Os filtros definidos capturaram os dados acima.",
                ["Simulate"] = "Simular (Preview)",
                ["NewDirectCopyTitle"] = "Nova Cópia Direta",
                ["NewWatchFolderTitle"] = "Nova Watch Folder",
                ["Save"] = "Salvar",
                ["Cancel"] = "Cancelar",

                // History / Audit
                ["AuditHistory"] = "Auditoria e Histórico",
                ["ForenseTracking"] = "Rastreamento forense de todas as operações de arquivos",
                ["ExportCsv"] = "Exportar CSV",
                ["OpDetails"] = "DETALHES DA OPERAÇÃO",
                ["File"] = "Arquivo",
                ["Destination"] = "Destino",
                ["Size"] = "Tamanho",
                ["Duration"] = "Duração",
                ["EngineLogs"] = "Logs do Engine",
                ["SelectEntry"] = "Selecione uma entrada para ver os detalhes",
                ["SearchHistory"] = "Pesquisar arquivo ou destino...",
                ["SearchWatermark"] = "Buscar arquivo ou detalhe...",
                ["Time"] = "Hora",
                ["Task"] = "Tarefa",
                ["Origin"] = "Origem",
                ["OpenSourceFolder"] = "Abrir Pasta de Origem",
                ["OpenDestFolder"] = "Abrir Pasta de Destino",
                ["ClearHistory"] = "Limpar Todo o Histórico",
                ["Copied"] = "COPIADO",
                ["Moved"] = "MOVIDO",
                ["Ignored"] = "IGNORADO",
                ["FailedStatus"] = "FALHA",
                ["Zipped"] = "ZIPADO",
                ["Cancelled"] = "CANCELADO",

                // Sidebar
                ["SidebarAutomation"] = "Automação",
                ["SidebarAudit"] = "Auditoria",

                // Automation View
                ["AutomationHub"] = "Central de Automação",
                ["ActiveBadge"] = "{0} Ativos",
                ["QueueReady"] = "Fila Pronta",
                ["AutomationDesc"] = "Orquestre, monitore e gerencie seus fluxos de dados em um único lugar",
                ["StopAllTip"] = "Encerrar Todas as Atividades",
                ["Active"] = "Ativos",
                ["Pending"] = "Pendentes",
                ["Watchers"] = "Watchers",
                ["SearchJobsWatermark"] = "Buscar por nome ou caminho...",
                ["SelectedCount"] = "{0} selecionados",
                ["RealTimeProcessing"] = "PROCESSAMENTO EM TEMPO REAL",
                ["ConfiguredTasks"] = "SUAS TAREFAS CONFIGURADAS",
                ["RunTip"] = "Iniciar",
                ["StopTip"] = "Parar",
                ["EditTip"] = "Editar",
                ["DeleteTip"] = "Excluir",
                ["OriginLabel"] = "ORIGEM:",
                ["DestLabel"] = "DESTINO:",
                ["Frequency"] = "FREQUÊNCIA",
                ["SpeedLabel"] = "VELOCIDADE",
                ["LiveTerminal"] = "TERMINAL LIVE",
                ["Processing"] = "Processando...",
                ["Idle"] = "Ocioso",
                ["Queued"] = "Na Fila...",
                ["NewDirectCopy"] = "Nova Cópia Direta",
                ["NewWatchFolder"] = "Nova Watch Folder",
                ["JobNameWatermark"] = "Ex: Backup de Documentos",
                ["Browse"] = "Procurar",
                ["ExtensionsWatermark"] = ".jpg, .png, .pdf",
                ["IgnoreWatermark"] = "node_modules, temp",
                ["RegexWatermark"] = "Ex: ^[0-9]{4}.*",
                ["ProcessSubfolders"] = "Processar Subpastas",
                ["SmartSyncLabel"] = "Smart Sync (Pular idênticos)",
                ["EncryptionKeyWatermarkEditor"] = "Chave para proteger os arquivos no destino",
                ["VerifyHashLabel"] = "Validar Integridade Pós-Cópia (Hash)",
                ["WebhookUrlWatermarkEditor"] = "URL específica para esta tarefa (Opcional)",
                ["ClearPreview"] = "Limpar Preview",
                ["SaveAutomation"] = "Salvar Automação",
                ["Sun"] = "D", ["Mon"] = "S", ["Tue"] = "T", ["Wed"] = "Q", ["Thu"] = "Q", ["Fri"] = "S", ["Sat"] = "S",
                ["OpenApp"] = "Abrir FolderFlow",
                ["Exit"] = "Sair",
                ["SuccessBadge"] = "OK",
                ["IgnoredBadge"] = "IGN",
                ["ErrorBadge"] = "ERR",
                ["SmtpEmailHeader"] = "SMTP / E-mail",

                // Engine
                ["JobStarted"] = "Tarefa '{0}' iniciada.",
                ["RetryMode"] = " [MODO RETRY]",
                ["PreScriptFailed"] = "Pre-script falhou. A tarefa continuará, mas verifique os logs.",
                ["SourceNotFound"] = "Pasta de origem não encontrada: {0}",
                ["JobError"] = "Erro no Job",
                ["JobFailedSourceNotFound"] = "Tarefa '{0}' falhou: Origem não encontrada.",
                ["StartingFile"] = "Iniciando: {0}",
                ["SuccessFile"] = "Sucesso: {0}",
                ["ErrorFile"] = "ERRO: {0} - {1}",
                ["FilterIgnored"] = "Filtro de exclusão ou critérios de data/tamanho.",
                ["JobFinished"] = "Job Finalizado: {0}. {1} processados.",
                ["JobCompleted"] = "Job Concluído",
                ["FilesFailed"] = "{0} arquivos falharam.",
                ["UserCancelled"] = "Tarefa '{0}' cancelada pelo usuário.",
                ["UserCancelledOp"] = "Operação cancelada pelo usuário.",
                ["CriticalError"] = "Erro Crítico",
                ["CriticalErrorJob"] = "Erro crítico na tarefa '{0}': {1}",
                ["PostScriptFailed"] = "Post-script falhou.",
                ["IntegrityFailed"] = "Falha na verificação de integridade (Hash mismatch).",
                ["ZipCreationFailed"] = "Falha ao criar arquivo ZIP: {0}",
                ["RetentionOldDeleted"] = "Retenção: Arquivo antigo excluído '{0}'.",
                ["RetentionExpiredDeleted"] = "Retenção: Arquivo expirado excluído '{0}'.",
                ["RetentionFailed"] = "Falha ao aplicar política de retenção: {0}",
                ["SmartSyncDetail"] = "SmartSync",
                ["StartingJobLog"] = "Iniciando Job: {0}",

                // Preview
                ["PreviewErrorSourceNotFound"] = "[ERRO] Diretório de origem não encontrado: {0}",
                ["PreviewIgnored"] = "[IGNORADO] {0}",
                ["PreviewIgnoredSmartSync"] = "[IGNORADO - SMART SYNC] {0}",
                ["PreviewIgnoredConflict"] = "[IGNORADO - CONFLITO] {0}",
                ["PreviewOverwrite"] = "[SOBRESCREVER] {0}",
                ["PreviewRename"] = "[RENOMEAR] {0}",
                ["PreviewCopy"] = "[COPIAR] {0}",
                ["PreviewMove"] = "[MOVER] {0}",

                // Formatting
                ["RamUsageFormat"] = "{0} GB / {1} GB",
                ["TimeSavedFormat"] = "{0}h {1}m",

                // Scheduler
                ["SchedulerLoopError"] = "Erro no loop do agendador: {0}",
                ["DailyMaintenanceSuccess"] = "Manutenção diária concluída com sucesso.",
                ["DailyMaintenanceError"] = "Erro na manutenção diária: {0}",
                ["DailySummaryName"] = "Resumo Diário",
                ["InXDays"] = "em {0} dias",
                ["InXHours"] = "em {0}h",
                ["InXMinutes"] = "em {0}min",
            }
        };
        return dict;
    }

    private void CreateDefaultFile(string path, string culture)
    {
        var dict = GetDefaults(culture);
        SaveTranslations(path, dict);
    }
}
