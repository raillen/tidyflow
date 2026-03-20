namespace FolderFlow.Application.Interfaces;

public interface INotificationService
{
    void Show(string title, string message, bool isError = false);
}
