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

    public JobAppService(IJobStore jobStore)
    {
        _jobStore = jobStore;
    }

    public Task<IEnumerable<Job>> GetAllJobsAsync() => _jobStore.GetAllAsync();

    public async Task SaveJobAsync(Job job)
    {
        ValidateJob(job);
        await _jobStore.SaveAsync(job);
    }

    public Task DeleteJobAsync(Guid id) => _jobStore.DeleteAsync(id);

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
            // Gera novo ID para evitar conflito com jobs existentes se necessário, 
            // ou apenas salva se for um job novo. No MVP, vamos garantir novos IDs para importações.
            job.Id = Guid.NewGuid();
            await _jobStore.SaveAsync(job);
            count++;
        }
        return count;
    }

    private void ValidateJob(Job job)
    {
        if (string.IsNullOrWhiteSpace(job.Name))
            throw new ArgumentException("O nome do Job é obrigatório.");

        if (string.IsNullOrWhiteSpace(job.SourcePath))
            throw new ArgumentException("O caminho de origem é obrigatório.");

        if (string.IsNullOrWhiteSpace(job.TargetPath))
            throw new ArgumentException("O caminho de destino é obrigatório.");

        if (job.SourcePath.Equals(job.TargetPath, StringComparison.OrdinalIgnoreCase))
            throw new ArgumentException("Origem e destino não podem ser iguais.");

        // Validação básica de existência (opcional para o momento da criação, mas boa prática)
        // No MVP, permitimos salvar mesmo que não exista, mas avisamos ou validamos na execução.
    }
}
