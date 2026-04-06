using System;
using System.Net.Http;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;
using AutoFlow.Domain.Entities;
using AutoFlow.Domain.Enums;

namespace AutoFlow.Infrastructure.Notifications;

public class WebhookNotificationService : IExternalNotificationService
{
    private readonly HttpClient _httpClient;
    private readonly IAppLogger _logger;

    public WebhookNotificationService(IAppLogger logger)
    {
        _httpClient = new HttpClient();
        _logger = logger;
    }

    public async Task NotifyJobCompletionAsync(Job job, bool success, int processedFiles, string? errorMessage = null)
    {
        if (string.IsNullOrWhiteSpace(job.WebhookUrl)) return;

        bool shouldNotify = job.NotifyOn == NotificationTrigger.Always ||
                            (job.NotifyOn == NotificationTrigger.Success && success) ||
                            (job.NotifyOn == NotificationTrigger.Failure && !success);

        if (!shouldNotify) return;

        try
        {
            var status = success ? "SUCESSO" : "FALHA";
            var color = success ? 65280 : 16711680; // Verde ou Vermelho (formato Discord decimal)

            // Formato compatvel com Discord Webhooks (e adaptvel para outros genricos)
            var payload = new
            {
                content = $"**AutoFlow Alerta**: A tarefa '{job.Name}' foi concluda com {status}.",
                embeds = new[]
                {
                    new
                    {
                        title = $"Resumo: {job.Name}",
                        description = success ? $"Processou {processedFiles} arquivos com sucesso." : $"Ocorreu um erro: {errorMessage}",
                        color = color,
                        fields = new[]
                        {
                            new { name = "Origem", value = job.SourcePath, inline = false },
                            new { name = "Destino", value = job.TargetPath, inline = false }
                        }
                    }
                }
            };

            var json = JsonSerializer.Serialize(payload);
            var content = new StringContent(json, Encoding.UTF8, "application/json");

            var response = await _httpClient.PostAsync(job.WebhookUrl, content);
            if (!response.IsSuccessStatusCode)
            {
                await _logger.LogAsync($"Falha ao enviar webhook para {job.WebhookUrl}: {response.StatusCode}", "WARNING");
            }
        }
        catch (Exception ex)
        {
            await _logger.LogAsync($"Erro interno ao tentar enviar webhook: {ex.Message}", "ERROR");
        }
    }
}
