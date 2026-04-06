using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Infrastructure.Persistence.Json;

public class ObservableJobQueue : IJobQueue
{
    private readonly List<Job> _queue = new();
    private readonly SemaphoreSlim _signal = new(0);
    private readonly object _lock = new();

    public bool IsPaused { get; set; }
    public int Count { get { lock (_lock) return _queue.Count; } }
    public IEnumerable<Job> PendingJobs { get { lock (_lock) return _queue.ToList(); } }

    public async ValueTask EnqueueAsync(Job job)
    {
        lock (_lock)
        {
            // Evita duplicatas na fila de espera
            if (_queue.Any(j => j.Id == job.Id)) return;
            _queue.Add(job);
        }
        _signal.Release();
        await Task.CompletedTask;
    }

    public async ValueTask<Job?> DequeueAsync()
    {
        await _signal.WaitAsync();
        lock (_lock)
        {
            if (_queue.Count == 0) return null;
            var job = _queue[0];
            _queue.RemoveAt(0);
            return job;
        }
    }

    public void Remove(Guid jobId)
    {
        lock (_lock)
        {
            var job = _queue.FirstOrDefault(j => j.Id == jobId);
            if (job != null && _queue.Remove(job))
            {
                // Se removeu, precisamos "consumir" um sinal se a fila ficou vazia 
                // ou apenas ajustar o semforo. 
                // Como o sinal  lançado no Enqueue, se removermos sem Dequeue, 
                // o prximo Dequeue pode travar ou pegar errado.
                // Mas no DequeueAsync j tratamos count=0.
            }
        }
    }

    public void MoveUp(Guid jobId)
    {
        lock (_lock)
        {
            int index = _queue.FindIndex(j => j.Id == jobId);
            if (index > 0)
            {
                var item = _queue[index];
                _queue.RemoveAt(index);
                _queue.Insert(index - 1, item);
            }
        }
    }

    public void MoveDown(Guid jobId)
    {
        lock (_lock)
        {
            int index = _queue.FindIndex(j => j.Id == jobId);
            if (index >= 0 && index < _queue.Count - 1)
            {
                var item = _queue[index];
                _queue.RemoveAt(index);
                _queue.Insert(index + 1, item);
            }
        }
    }

    public void PushToTop(Guid jobId)
    {
        lock (_lock)
        {
            int index = _queue.FindIndex(j => j.Id == jobId);
            if (index > 0)
            {
                var item = _queue[index];
                _queue.RemoveAt(index);
                _queue.Insert(0, item);
            }
        }
    }

    public void Clear()
    {
        lock (_lock)
        {
            _queue.Clear();
            // Resetar o sinal  complexo com SemaphoreSlim, 
            // mas o Dequeue tratar lista vazia.
        }
    }
}
