using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using FolderFlow.Application.Interfaces;

namespace FolderFlow.Infrastructure.Localization;

public class JsonLocalizationService : ILocalizationService
{
    private Dictionary<string, string> _translations = new();
    private string _currentCulture = "pt-BR";
    private readonly string _basePath;

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
    }

    private void LoadTranslations()
    {
        var filePath = Path.Combine(_basePath, $"{_currentCulture}.json");
        if (!File.Exists(filePath))
        {
            // Criar arquivo padrão se não existir (apenas para demonstração no MVP)
            if (_currentCulture == "pt-BR") CreateDefaultPtBr(filePath);
            else return;
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

    private void CreateDefaultPtBr(string path)
    {
        var defaults = new Dictionary<string, string>
        {
            ["Dashboard"] = "Painel Principal",
            ["Jobs"] = "Tarefas",
            ["History"] = "Histórico",
            ["Settings"] = "Configurações",
            ["NewJob"] = "Nova Tarefa",
            ["EditJob"] = "Editar Tarefa",
            ["Save"] = "Salvar",
            ["Cancel"] = "Cancelar",
            ["Delete"] = "Excluir",
            ["Run"] = "Executar",
            ["Theme"] = "Tema",
            ["Language"] = "Idioma"
        };
        var json = JsonSerializer.Serialize(defaults, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(path, json);
    }
}
