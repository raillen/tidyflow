namespace FolderFlow.Domain.Enums;

public enum ScheduleType
{
    None,
    Interval,     // Ex: a cada X minutos
    Daily,        // Ex: todo dia as HH:mm
    Weekly        // Ex: toda segunda as HH:mm
}
