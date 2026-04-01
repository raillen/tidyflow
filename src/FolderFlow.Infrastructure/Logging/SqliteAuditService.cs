using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;
using Microsoft.Data.Sqlite;

namespace FolderFlow.Infrastructure.Logging;

public class SqliteAuditService : IAuditService
{
    private readonly string _connectionString;
    private readonly string _dbPath;

    public SqliteAuditService(string? basePath = null)
    {
        var dataFolder = basePath ?? Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data");
        if (!Directory.Exists(dataFolder)) Directory.CreateDirectory(dataFolder);

        _dbPath = Path.Combine(dataFolder, "audit.db");
        _connectionString = $"Data Source={_dbPath}";

        InitializeDatabase();
    }

    private void InitializeDatabase()
    {
        using var connection = new SqliteConnection(_connectionString);
        connection.Open();

        var command = connection.CreateCommand();
        command.CommandText = @"
            CREATE TABLE IF NOT EXISTS AuditEntries (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                Timestamp TEXT NOT NULL,
                JobName TEXT NOT NULL,
                SourcePath TEXT,
                TargetPath TEXT,
                Status TEXT NOT NULL,
                Details TEXT,
                FileSize INTEGER DEFAULT 0,
                DurationMs REAL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON AuditEntries(Timestamp);
            CREATE INDEX IF NOT EXISTS idx_audit_jobname ON AuditEntries(JobName);
        ";
        command.ExecuteNonQuery();
    }

    public async Task SaveReportAsync(string jobName, IEnumerable<AuditEntry> entries)
    {
        using var connection = new SqliteConnection(_connectionString);
        await connection.OpenAsync();

        using var transaction = connection.BeginTransaction();
        var command = connection.CreateCommand();
        command.CommandText = @"
            INSERT INTO AuditEntries (Timestamp, JobName, SourcePath, TargetPath, Status, Details, FileSize, DurationMs)
            VALUES ($timestamp, $jobname, $source, $target, $status, $details, $size, $duration)
        ";

        var pTimestamp = command.CreateParameter(); pTimestamp.ParameterName = "$timestamp"; command.Parameters.Add(pTimestamp);
        var pJobName = command.CreateParameter(); pJobName.ParameterName = "$jobname"; command.Parameters.Add(pJobName);
        var pSource = command.CreateParameter(); pSource.ParameterName = "$source"; command.Parameters.Add(pSource);
        var pTarget = command.CreateParameter(); pTarget.ParameterName = "$target"; command.Parameters.Add(pTarget);
        var pStatus = command.CreateParameter(); pStatus.ParameterName = "$status"; command.Parameters.Add(pStatus);
        var pDetails = command.CreateParameter(); pDetails.ParameterName = "$details"; command.Parameters.Add(pDetails);
        var pSize = command.CreateParameter(); pSize.ParameterName = "$size"; command.Parameters.Add(pSize);
        var pDuration = command.CreateParameter(); pDuration.ParameterName = "$duration"; command.Parameters.Add(pDuration);

        foreach (var entry in entries)
        {
            pTimestamp.Value = entry.Timestamp.ToString("yyyy-MM-dd HH:mm:ss");
            pJobName.Value = entry.JobName;
            pSource.Value = entry.SourcePath ?? (object)DBNull.Value;
            pTarget.Value = entry.TargetPath ?? (object)DBNull.Value;
            pStatus.Value = entry.Status;
            pDetails.Value = entry.Details ?? (object)DBNull.Value;
            pSize.Value = entry.FileSize;
            pDuration.Value = entry.DurationMs;
            await command.ExecuteNonQueryAsync();
        }

        await transaction.CommitAsync();
    }

    public async Task<long> GetTotalBytesProcessedAsync(string? jobName = null)
    {
        using var connection = new SqliteConnection(_connectionString);
        await connection.OpenAsync();
        var command = connection.CreateCommand();
        command.CommandText = "SELECT SUM(FileSize) FROM AuditEntries WHERE Status IN ('COPIADO', 'MOVIDO', 'ZIPADO')";
        if (!string.IsNullOrEmpty(jobName))
        {
            command.CommandText += " AND JobName = $name";
            command.Parameters.AddWithValue("$name", jobName);
        }
        var result = await command.ExecuteScalarAsync();
        return result != DBNull.Value ? Convert.ToInt64(result) : 0;
    }

    public async Task<(int success, int ignored, int errors)> GetStatsAsync(string? jobName = null)
    {
        using var connection = new SqliteConnection(_connectionString);
        await connection.OpenAsync();
        var command = connection.CreateCommand();
        command.CommandText = @"
            SELECT 
                COUNT(CASE WHEN Status IN ('COPIADO', 'MOVIDO', 'ZIPADO') THEN 1 END),
                COUNT(CASE WHEN Status = 'IGNORADO' THEN 1 END),
                COUNT(CASE WHEN Status LIKE 'FALHA%' THEN 1 END)
            FROM AuditEntries";
        
        if (!string.IsNullOrEmpty(jobName))
        {
            command.CommandText += " WHERE JobName = $name";
            command.Parameters.AddWithValue("$name", jobName);
        }

        using var reader = await command.ExecuteReaderAsync();
        if (await reader.ReadAsync())
        {
            return (reader.GetInt32(0), reader.GetInt32(1), reader.GetInt32(2));
        }
        return (0, 0, 0);
    }

    public async Task<IEnumerable<AuditEntry>> GetLogsAsync(string? jobName = null, string? status = null, string? searchText = null, int limit = 100)
    {
        var logs = new List<AuditEntry>();
        using var connection = new SqliteConnection(_connectionString);
        await connection.OpenAsync();

        var command = connection.CreateCommand();
        var query = "SELECT Timestamp, JobName, SourcePath, TargetPath, Status, Details, FileSize, DurationMs FROM AuditEntries WHERE 1=1";

        if (!string.IsNullOrEmpty(jobName) && jobName != "Todos")
        {
            query += " AND JobName = $job";
            command.Parameters.AddWithValue("$job", jobName);
        }

        if (!string.IsNullOrEmpty(status) && status != "Todos")
        {
            query += " AND Status LIKE $status";
            command.Parameters.AddWithValue("$status", $"%{status}%");
        }

        if (!string.IsNullOrEmpty(searchText))
        {
            query += " AND (SourcePath LIKE $search OR TargetPath LIKE $search OR Details LIKE $search)";
            command.Parameters.AddWithValue("$search", $"%{searchText}%");
        }

        query += " ORDER BY Timestamp DESC LIMIT $limit";
        command.Parameters.AddWithValue("$limit", limit);
        command.CommandText = query;

        using var reader = await command.ExecuteReaderAsync();
        while (await reader.ReadAsync())
        {
            logs.Add(new AuditEntry
            {
                Timestamp = DateTime.Parse(reader.GetString(0)),
                JobName = reader.GetString(1),
                SourcePath = reader.IsDBNull(2) ? "" : reader.GetString(2),
                TargetPath = reader.IsDBNull(3) ? "" : reader.GetString(3),
                Status = reader.GetString(4),
                Details = reader.IsDBNull(5) ? "" : reader.GetString(5),
                FileSize = reader.GetInt64(6),
                DurationMs = reader.GetDouble(7)
            });
        }
        return logs;
    }

    public async Task ClearAllLogsAsync()
    {
        using var connection = new SqliteConnection(_connectionString);
        await connection.OpenAsync();
        var command = connection.CreateCommand();
        command.CommandText = "DELETE FROM AuditEntries";
        await command.ExecuteNonQueryAsync();
        await VacuumAsync();
    }

    public long GetDatabaseSize() => new FileInfo(_dbPath).Length;

    public async Task VacuumAsync()
    {
        using var connection = new SqliteConnection(_connectionString);
        await connection.OpenAsync();
        var command = connection.CreateCommand();
        command.CommandText = "VACUUM";
        await command.ExecuteNonQueryAsync();
    }

    public async Task<int> PurgeOldLogsAsync(int days)
    {
        if (days <= 0) return 0;
        
        using var connection = new SqliteConnection(_connectionString);
        await connection.OpenAsync();
        var command = connection.CreateCommand();
        command.CommandText = "DELETE FROM AuditEntries WHERE Timestamp < $cutoff";
        command.Parameters.AddWithValue("$cutoff", DateTime.Now.AddDays(-days).ToString("yyyy-MM-dd HH:mm:ss"));
        
        int affected = await command.ExecuteNonQueryAsync();
        if (affected > 0) await VacuumAsync();
        return affected;
    }

    public async Task<string> GetDailySummaryAsync()
    {
        using var connection = new SqliteConnection(_connectionString);
        await connection.OpenAsync();
        var command = connection.CreateCommand();
        command.CommandText = @"
            SELECT JobName, 
                   COUNT(CASE WHEN Status IN ('COPIADO', 'MOVIDO', 'ZIPADO') THEN 1 END) as Success,
                   COUNT(CASE WHEN Status LIKE 'FALHA%' THEN 1 END) as Fail,
                   SUM(FileSize) as Bytes
            FROM AuditEntries 
            WHERE Timestamp >= $cutoff
            GROUP BY JobName";
        command.Parameters.AddWithValue("$cutoff", DateTime.Now.AddDays(-1).ToString("yyyy-MM-dd HH:mm:ss"));

        var sb = new StringBuilder();
        sb.AppendLine("Resumo de Atividades - ltimas 24 horas");
        sb.AppendLine("---------------------------------------");
        
        using var reader = await command.ExecuteReaderAsync();
        bool hasData = false;
        while (await reader.ReadAsync())
        {
            hasData = true;
            var name = reader.GetString(0);
            var success = reader.GetInt32(1);
            var fail = reader.GetInt32(2);
            var bytes = reader.IsDBNull(3) ? 0 : reader.GetInt64(3);
            sb.AppendLine($"Job: {name} | Sucessos: {success} | Falhas: {fail} | Volume: {bytes / 1024.0 / 1024.0:F2} MB");
        }

        return hasData ? sb.ToString() : "Nenhuma atividade registrada nas ltimas 24 horas.";
    }
}
