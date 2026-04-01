using System;
using System.Threading;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Domain.Entities;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

namespace FolderFlow.Service;

public class Worker : BackgroundService
{
    private readonly ILogger<Worker> _logger;
    private readonly QueueProcessor _queueProcessor;
    private readonly ISchedulerService _schedulerService;
    private readonly WatchAppService _watchAppService;
    private readonly JobAppService _jobAppService;

    public Worker(
        ILogger<Worker> logger, 
        QueueProcessor queueProcessor,
        ISchedulerService schedulerService,
        WatchAppService watchAppService,
        JobAppService jobAppService)
    {
        _logger = logger;
        _queueProcessor = queueProcessor;
        _schedulerService = schedulerService;
        _watchAppService = watchAppService;
        _jobAppService = jobAppService;
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        _logger.LogInformation("FolderFlow Windows Service started at: {time}", DateTimeOffset.Now);

        try
        {
            // Inicia processador de fila e o agendador
            _queueProcessor.Start(stoppingToken);
            _schedulerService.Start(stoppingToken);

            // Carrega tarefas ativas e inicia watches
            var jobs = await _jobAppService.GetAllJobsAsync();
            foreach (var job in jobs)
            {
                if (job.WatchEnabled)
                {
                    _watchAppService.UpdateJobWatching(job);
                }
            }

            while (!stoppingToken.IsCancellationRequested)
            {
                await Task.Delay(10000, stoppingToken);
            }
        }
        catch (OperationCanceledException)
        {
            // Normal shutdown
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Fatal error in FolderFlow Service");
        }
        finally
        {
            _logger.LogInformation("FolderFlow Windows Service stopped at: {time}", DateTimeOffset.Now);
        }
    }
}
