namespace FolderFlow.Application.Interfaces;

public interface ILocalizationService
{
    string this[string key] { get; }
    string GetString(string key);
    void SetLanguage(string cultureCode);
}
