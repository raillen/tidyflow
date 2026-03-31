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
        _currentCulture = cultureCode;
        LoadTranslations();
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(null));
    }

    private void LoadTranslations()
    {
        var filePath = Path.Combine(_basePath, $"{_currentCulture}.json");
        if (!File.Exists(filePath))
        {
            CreateDefaultFile(filePath, _currentCulture);
        }

        try
        {
            var json = File.ReadAllText(filePath);
            _translations = JsonSerializer.Deserialize<Dictionary<string, string>>(json) ?? new();
        }
        catch
        {
            _translations = new();
        }
    }

    private void CreateDefaultFile(string path, string culture)
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
                ["NoLogs"] = "No logs found."
            },
            "es-ES" => new Dictionary<string, string> { 
                ["Dashboard"] = "Panel", ["Jobs"] = "Cola", ["History"] = "Historial", ["Settings"] = "Ajustes", 
                ["Greeting"] = "¡Buenas noches!", ["ActiveAuto"] = "Tienes {0} automatizaciones activas.",
                ["Total"] = "Total", ["Running"] = "Ejecutando", ["Completed"] = "Completado", ["Failed"] = "Fallido",
                ["RecentActivity"] = "Actividad Reciente", ["QuickActions"] = "Acciones Rápidas", ["ErrorAudit"] = "Auditoría",
                ["NewJob"] = "Nueva Tarea", ["NewAuto"] = "Nueva Automatización", ["ViewHistory"] = "Ver Historial",
                ["NoErrors"] = "Sin errores.", ["OpenErrorFolder"] = "Abrir Carpeta", ["ClearLogs"] = "Limpiar Logs",
                ["SaveSettings"] = "Guardar", ["Appearance"] = "Apariencia", ["SystemTheme"] = "Tema", ["Language"] = "Idioma",
                ["Behavior"] = "Comportamiento", ["ShowNotifications"] = "Notificaciones", ["StartAtStartup"] = "Auto-inicio",
                ["Support"] = "Soporte", ["SupportDesc"] = "¡Si te gusta, apóyanos!", ["BuyCoffee"] = "Invítame a un café",
                ["Add"] = "Añadir", ["Hotfolder"] = "Hotfolder", ["Run"] = "Ejecutar", ["StopAll"] = "Parar Todo", ["Delete"] = "Eliminar",
                ["QueueManagement"] = "GESTIONAR COLA", ["OpLog"] = "Log de Operaciones", ["ManualOp"] = "Operaciones Manuales", ["Automation"] = "Automatizaciones",
                ["HistoryHeader"] = "HISTORIAL DE AUTOMATIZACIÓN", ["Update"] = "Actualizar", ["SearchHistory"] = "Buscar archivos o destinos...",
                ["NoLogs"] = "No se encontraron logs."
            },
            "ja-JP" => new Dictionary<string, string> { 
                ["Dashboard"] = "ダッシュボード", ["Jobs"] = "キュー", ["History"] = "履歴", ["Settings"] = "設定", 
                ["Greeting"] = "こんばんは！", ["ActiveAuto"] = "{0} 個の自動化が有効です。",
                ["Total"] = "合計", ["Running"] = "実行中", ["Completed"] = "完了", ["Failed"] = "失敗",
                ["RecentActivity"] = "最近の活動", ["QuickActions"] = "クイックアクション", ["ErrorAudit"] = "エラー監査",
                ["NewJob"] = "新規タスク", ["NewAuto"] = "新規自動化", ["ViewHistory"] = "履歴表示",
                ["NoErrors"] = "エラーなし。", ["OpenErrorFolder"] = "フォルダを開く", ["ClearLogs"] = "ログ削除",
                ["SaveSettings"] = "保存", ["Appearance"] = "外観", ["SystemTheme"] = "テーマ", ["Language"] = "言語",
                ["Behavior"] = "動作", ["ShowNotifications"] = "通知", ["StartAtStartup"] = "自動起動",
                ["Support"] = "サポート", ["SupportDesc"] = "開発者を支援してください！", ["BuyCoffee"] = "コーヒーをおごる",
                ["Add"] = "追加", ["Hotfolder"] = "ホットフォルダ", ["Run"] = "開始", ["StopAll"] = "全停止", ["Delete"] = "削除",
                ["QueueManagement"] = "キュー管理", ["OpLog"] = "操作ログ", ["ManualOp"] = "手動操作", ["Automation"] = "自動化",
                ["HistoryHeader"] = "自動化履歴", ["Update"] = "更新", ["SearchHistory"] = "ファイルまたは宛先を検索...",
                ["NoLogs"] = "ログが見つかりません。"
            },
            "ru-RU" => new Dictionary<string, string> { 
                ["Dashboard"] = "Панель", ["Jobs"] = "Очередь", ["History"] = "История", ["Settings"] = "Настройки", 
                ["Greeting"] = "Добрый вечер!", ["ActiveAuto"] = "У вас {0} активных автоматизаций.",
                ["Total"] = "Всего", ["Running"] = "Запущено", ["Completed"] = "Завершено", ["Failed"] = "Ошибка",
                ["RecentActivity"] = "Активность", ["QuickActions"] = "Действия", ["ErrorAudit"] = "Аудит",
                ["NewJob"] = "Новая задача", ["NewAuto"] = "Автоматизация", ["ViewHistory"] = "История",
                ["NoErrors"] = "Ошибок нет.", ["OpenErrorFolder"] = "Открыть папку", ["ClearLogs"] = "Очистить",
                ["SaveSettings"] = "Сохранить", ["Appearance"] = "Внешний вид", ["SystemTheme"] = "Тема", ["Language"] = "Язык",
                ["Behavior"] = "Поведение", ["ShowNotifications"] = "Уведомления", ["StartAtStartup"] = "Автозагрузка",
                ["Support"] = "Поддержка", ["SupportDesc"] = "Поддержите проект!", ["BuyCoffee"] = "Купить кофе",
                ["Add"] = "Добавить", ["Hotfolder"] = "Хотфолдер", ["Run"] = "Пуск", ["StopAll"] = "Стоп", ["Delete"] = "Удалить",
                ["QueueManagement"] = "УПРАВЛЕНИЕ ОЧЕРЕДЬЮ", ["OpLog"] = "Лог операций", ["ManualOp"] = "Вручную", ["Automation"] = "Автоматика",
                ["HistoryHeader"] = "ИСТОРИЯ АВТОМАТИЗАЦИИ", ["Update"] = "Обновить", ["SearchHistory"] = "Поиск файлов или папок...",
                ["NoLogs"] = "Логи не найдены."
            },
            _ => new Dictionary<string, string> { 
                ["Dashboard"] = "Painel", ["Jobs"] = "Fila", ["History"] = "Histórico", ["Settings"] = "Configurações", 
                ["Greeting"] = "Boa noite!", ["ActiveAuto"] = "Você tem {0} automações ativas.",
                ["Total"] = "Total", ["Running"] = "Rodando", ["Completed"] = "Concluído", ["Failed"] = "Falhou",
                ["RecentActivity"] = "Atividade Recente", ["QuickActions"] = "Ações Rápidas", ["ErrorAudit"] = "Auditoria de Erros",
                ["NewJob"] = "Nova Tarefa", ["NewAuto"] = "Nova Automatização", ["ViewHistory"] = "Ver Histórico",
                ["NoErrors"] = "Nenhum erro registrado no sistema.", ["OpenErrorFolder"] = "Abrir Pasta de Erros", ["ClearLogs"] = "Limpar Logs",
                ["SaveSettings"] = "Salvar Configurações", ["Appearance"] = "Aparência e Localização", ["SystemTheme"] = "Tema do Aplicativo", ["Language"] = "Idioma (Cultura)",
                ["Behavior"] = "Comportamiento do Sistema", ["ShowNotifications"] = "Exibir notificações na área de trabalho (Toasts)", ["StartAtStartup"] = "Iniciar FolderFlow automaticamente com o sistema (Background)",
                ["Support"] = "Suporte e Doação", ["SupportDesc"] = "Se o FolderFlow tem sido útil para você, considere apoiar o desenvolvedor!", ["BuyCoffee"] = "Apoie o desenvolvedor",
                ["Add"] = "Adicionar", ["Hotfolder"] = "Hotfolder", ["Run"] = "Rodar", ["StopAll"] = "Parar Tudo", ["Delete"] = "Excluir",
                ["QueueManagement"] = "GERENCIAR FILA DE OPERAÇÕES", ["OpLog"] = "Log de Operações", ["ManualOp"] = "Operações Manuais", ["Automation"] = "Automatizações",
                ["HistoryHeader"] = "HISTÓRICO DE AUTOMAÇÃO", ["Update"] = "Atualizar", ["SearchHistory"] = "Pesquisar arquivo ou destino...",
                ["NoLogs"] = "Nenhum log encontrado."
            }
        };
        
        var json = JsonSerializer.Serialize(dict, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(path, json);
    }
}
