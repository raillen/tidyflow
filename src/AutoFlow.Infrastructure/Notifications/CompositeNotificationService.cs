using System;
using System.Net;
using System.Net.Mail;
using System.Net.Http;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;
using AutoFlow.Domain.Entities;
using AutoFlow.Domain.Enums;

namespace AutoFlow.Infrastructure.Notifications;

public class CompositeNotificationService : IExternalNotificationService
{
    private readonly HttpClient _httpClient;
    private readonly ISettingsStore _settingsStore;
    private readonly IAppLogger _logger;

    public CompositeNotificationService(ISettingsStore settingsStore, IAppLogger logger)
    {
        _httpClient = new HttpClient();
        _settingsStore = settingsStore;
        _logger = logger;
    }

    public async Task NotifyJobCompletionAsync(Job job, bool success, int processedFiles, string? errorMessage = null)
    {
        var settings = await _settingsStore.LoadAsync();
        
        // Determina se deve notificar baseado na tarefa ou no global
        bool shouldNotify = (success && (job.NotifyOn == NotificationTrigger.Success || job.NotifyOn == NotificationTrigger.Always || settings.NotifyOnSuccess)) ||
                            (!success && (job.NotifyOn == NotificationTrigger.Failure || job.NotifyOn == NotificationTrigger.Always || settings.NotifyOnError));

        if (!shouldNotify) return;

        // 1. Enviar Webhook
        var webhookUrl = !string.IsNullOrEmpty(job.WebhookUrl) ? job.WebhookUrl : settings.WebhookUrl;
        if (!string.IsNullOrEmpty(webhookUrl))
        {
            await SendWebhookAsync(webhookUrl, settings.WebhookType, job.Name, success, processedFiles, errorMessage);
        }

        // 2. Enviar E-mail (SMTP)
        if (settings.EnableSmtp && !string.IsNullOrEmpty(settings.NotificationEmail))
        {
            await SendEmailAsync(settings, job.Name, success, processedFiles, errorMessage);
        }
    }

    private async Task SendWebhookAsync(string url, string type, string jobName, bool success, int count, string? error)
    {
        try
        {
            object payload;
            if (type == "Discord")
            {
                payload = new {
                    content = $"**AutoFlow**: {jobName} -> {(success ? "OK" : "FALHOU")}",
                    embeds = new[] {
                        new {
                            title = jobName,
                            description = success ? $"Sucesso: {count} arquivos." : $"Erro: {error}",
                            color = success ? 65280 : 16711680
                        }
                    }
                };
            }
            else if (type == "Slack")
            {
                payload = new {
                    text = $"*AutoFlow*: {jobName} status: {(success ? ":white_check_mark:" : ":x:")}\n{(success ? $"Arquivos: {count}" : $"Detalhes: {error}")}"
                };
            }
            else
            {
                payload = new { job = jobName, success, count, error, timestamp = DateTime.Now };
            }

            var json = JsonSerializer.Serialize(payload);
            await _httpClient.PostAsync(url, new StringContent(json, Encoding.UTF8, "application/json"));
        }
        catch (Exception ex) { await _logger.LogAsync($"Erro Webhook: {ex.Message}", "WARNING"); }
    }

    private async Task SendEmailAsync(AppSettings s, string jobName, bool success, int count, string? error)
    {
        try
        {
            using var client = new SmtpClient(s.SmtpHost, s.SmtpPort)
            {
                Credentials = new NetworkCredential(s.SmtpUser, s.SmtpPass),
                EnableSsl = true
            };

            var body = success ? $"A tarefa {jobName} foi concluda com sucesso em {DateTime.Now}.\nArquivos processados: {count}" 
                               : $"A tarefa {jobName} FALHOU em {DateTime.Now}.\nErro: {error}";

            var mail = new MailMessage("no-reply@autoflow.com", s.NotificationEmail, $"AutoFlow: {jobName} - {(success ? "Sucesso" : "Falha")}", body);
            await client.SendMailAsync(mail);
        }
        catch (Exception ex) { await _logger.LogAsync($"Erro SMTP: {ex.Message}", "WARNING"); }
    }
}
