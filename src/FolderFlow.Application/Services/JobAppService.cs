using System;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Services;

public class JobAppService
{
    private readonly IJobStore _jobStore;
    private readonly IJobQueue _jobQueue;

    public JobAppService(IJobStore jobStore, IJobQueue jobQueue)
    {
        _jobStore = jobStore;
        _jobQueue = jobQueue;
    }

    public Task<IEnumerable<Job>> GetAllJobsAsync() => _jobStore.GetAllAsync();

    public async Task SaveJobAsync(Job job)
    {
        ValidateJob(job);
        await _jobStore.SaveAsync(job);
    }

    public Task DeleteJobAsync(Guid id) => _jobStore.DeleteAsync(id);

    public async Task RunJobAsync(Guid id)
    {
        var job = await _jobStore.GetByIdAsync(id);
        if (job != null)
        {
            await _jobQueue.EnqueueAsync(job);
        }
    }

    public async Task ExportJobsAsync(IEnumerable<Job> jobs, string filePath)
    {
        var json = System.Text.Json.JsonSerializer.Serialize(jobs, new System.Text.Json.JsonSerializerOptions { WriteIndented = true });
        await File.WriteAllTextAsync(filePath, json);
    }

    public async Task<int> ImportJobsAsync(string filePath)
    {
        if (!File.Exists(filePath)) return 0;

        var json = await File.ReadAllTextAsync(filePath);
        var imported = System.Text.Json.JsonSerializer.Deserialize<List<Job>>(json);
        
        if (imported == null) return 0;

        int count = 0;
        foreach (var job in imported)
        {
            job.Id = Guid.NewGuid();
            await _jobStore.SaveAsync(job);
            count++;
        }
        return count;
    }

    private void ValidateJob(Job job)
    {
        if (string.IsNullOrWhiteSpace(job.Name))
            throw new ArgumentException("O nome do Job  obrigatrio.");

        if (string.IsNullOrWhiteSpace(job.SourcePath))
            throw new ArgumentException("O caminho de origem  obrigatrio.");

        if (string.IsNullOrWhiteSpace(job.TargetPath))
            throw new ArgumentException("O caminho de destino  obrigatrio.");

        if (job.SourcePath.Equals(job.TargetPath, StringComparison.OrdinalIgnoreCase))
            throw new ArgumentException("Origem e destino no podem ser iguais.");
    }
}
