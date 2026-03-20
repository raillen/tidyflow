using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Infrastructure.Persistence.Json;

public class JobJsonStore : IJobStore
{
    private readonly string _jobsFolder;

    public JobJsonStore(string? basePath = null)
    {
        var dataFolder = basePath ?? Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data");
        _jobsFolder = Path.Combine(dataFolder, "Jobs");
        if (!Directory.Exists(_jobsFolder))
        {
            Directory.CreateDirectory(_jobsFolder);
        }
    }

    public async Task<IEnumerable<Job>> GetAllAsync()
    {
        var jobs = new List<Job>();
        var files = Directory.GetFiles(_jobsFolder, "*.json");

        foreach (var file in files)
        {
            try
            {
                var json = await File.ReadAllTextAsync(file);
                var job = JsonSerializer.Deserialize<Job>(json);
                if (job != null)
                {
                    jobs.Add(job);
                }
            }
            catch
            {
                // Ignora arquivos corrompidos no MVP
            }
        }

        return jobs;
    }

    public async Task<Job?> GetByIdAsync(Guid id)
    {
        var filePath = GetFilePath(id);
        if (!File.Exists(filePath)) return null;

        try
        {
            var json = await File.ReadAllTextAsync(filePath);
            return JsonSerializer.Deserialize<Job>(json);
        }
        catch
        {
            return null;
        }
    }

    public async Task SaveAsync(Job job)
    {
        var filePath = GetFilePath(job.Id);
        var json = JsonSerializer.Serialize(job, new JsonSerializerOptions { WriteIndented = true });
        await File.WriteAllTextAsync(filePath, json);
    }

    public async Task DeleteAsync(Guid id)
    {
        var filePath = GetFilePath(id);
        if (File.Exists(filePath))
        {
            File.Delete(filePath);
        }
        await Task.CompletedTask;
    }

    private string GetFilePath(Guid id) => Path.Combine(_jobsFolder, $"{id}.json");
}
