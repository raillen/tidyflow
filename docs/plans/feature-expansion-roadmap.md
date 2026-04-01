# Roadmap de Expansão: FolderFlow Premium

Este documento define o plano arquitetural e de execução para elevar o **FolderFlow** de uma excelente ferramenta de cópia para uma solução de backup e automação de nível corporativo e comercial.

## Visão Geral das Fases

O plano está dividido em 4 fases, priorizadas do maior impacto e menor esforço (Quick Wins) até as reestruturações arquiteturais mais profundas.

---

## 🎯 Fase 1: Quick Wins & Experiência do Usuário (UX)
*Foco: Adicionar valor visual imediato e integrações fáceis para atrair usuários e DevOps.*

### 1.1. Tempo Restante Estimado (ETA)
*   **Descrição:** Calcular e exibir o tempo restante de uma operação em andamento.
*   **Implementação:**
    *   **Domain:** Adicionar `EstimatedTimeRemaining` em `JobProgressInfo`.
    *   **Application:** Atualizar `GlobalProgressService` para usar Média Móvel Exponencial (EMA) da velocidade (`TransferSpeed`) contra os bytes restantes (`TotalBytes - ProcessedBytes`).
    *   **App:** Atualizar `JobsView.axaml` e o Dashboard para exibir o ETA no formato `hh:mm:ss`.

### 1.2. Notificações Externas (Webhooks)
*   **Descrição:** Enviar alertas de Sucesso/Falha para Discord, Slack, Teams ou e-mails.
*   **Implementação:**
    *   **Domain:** Adicionar campo `WebhookUrl` e `NotifyOn` (Success, Failure, Both) na entidade `Job`.
    *   **Infrastructure:** Criar `WebhookNotificationService` implementando uma nova interface `IExternalNotificationService`.
    *   **Application:** No `ExecutionEngine`, após a execução do Job, realizar um POST assíncrono para o Webhook com o resumo do `AuditEntry`.

### 1.3. Scripts Pré e Pós Execução (Hooks)
*   **Descrição:** Rodar scripts `.bat`, `.ps1` ou `.sh` atrelados à tarefa.
*   **Implementação:**
    *   **Domain:** Adicionar `PreScriptPath` e `PostScriptPath` na entidade `Job`.
    *   **Application:** Injetar um `IScriptRunner`. No `ExecutionEngine`, aguardar a execução do Pre-Script antes de iniciar a cópia e o Post-Script ao finalizar (ou em caso de erro, com flags específicas).

---

## 🔒 Fase 2: Segurança, Retenção e Otimização
*Foco: Proteger os dados do usuário, economizar espaço e evitar perdas catastróficas.*

### 2.1. Compactação de Arquivos (ZIP/7z)
*   **Descrição:** Salvar os backups em arquivos compactados.
*   **Implementação:**
    *   **Domain:** Adicionar Enum `ArchiveFormat` (None, Zip) na entidade `Job`.
    *   **Infrastructure:** Criar `ZipFileOperator` que implementa `IFileOperator` usando `System.IO.Compression`.
    *   **Application:** O `ExecutionEngine` usará um padrão *Factory* para decidir qual `IFileOperator` usar dependendo do tipo da tarefa.

### 2.2. Criptografia AES-256
*   **Descrição:** Criptografar arquivos durante o trânsito e no repouso.
*   **Implementação:**
    *   **Domain:** Adicionar `EncryptionKey` (armazenada de forma segura/hasheada) no `Job`.
    *   **Infrastructure:** Criar `CryptoStreamService` para envolver a `FileStream` durante a leitura/escrita no `LocalFileOperator`.

### 2.3. Versionamento (Grandfather-Father-Son)
*   **Descrição:** Manter histórico de backups em vez de apenas sobrescrever.
*   **Implementação:**
    *   **Domain:** Adicionar `RetentionPolicy` (Ex: KeepLastXVersions, KeepDays).
    *   **Application:** Criar um `RetentionAppService` que roda após o Job para limpar arquivos antigos no diretório de destino baseado na política escolhida.

---

## 🌍 Fase 3: Conectividade e Enterprise
*Foco: Romper a barreira do armazenamento local e conectar com redes externas.*

### 3.1. Conexões SFTP / FTP / SMB
*   **Descrição:** Usar servidores externos como Origem ou Destino.
*   **Implementação:**
    *   **Domain:** Mudar `SourcePath` e `TargetPath` para suportar URIs (ex: `sftp://user:pass@host/path`). Adicionar gerenciamento de credenciais via *Windows Credential Manager* ou *KeyRing*.
    *   **Infrastructure:** Usar biblioteca `SSH.NET` para criar um `SftpFileOperator` que obedece à mesma interface `IFileOperator`.
    *   **Application:** Expandir o *Factory* de FileOperators para injetar o operador correto baseado no prefixo da string do caminho.

### 3.2. Controle de Banda (Throttling) e Concorrência
*   **Descrição:** Limitar a velocidade máxima de rede/disco e arquivos paralelos.
*   **Implementação:**
    *   **Domain:** Adicionar `MaxBandwidthMBps` e `MaxConcurrentFiles` no `Job`.
    *   **Infrastructure:** Modificar o buffer do loop em `LocalFileOperator.CopyAsync` para utilizar um `SemaphoreSlim` e `Task.Delay` baseados no cálculo de tokens (Token Bucket Algorithm) para limitar a velocidade por segundo.

---

## ⚙️ Fase 4: Engenharia Avançada
*Foco: Desempenho extremo e execução silenciosa.*

### 4.1. Sincronização Delta (Nível de Bloco)
*   **Descrição:** Copiar apenas as partes modificadas de arquivos gigantes (PST, VHD, ISO).
*   **Implementação:**
    *   **Infrastructure:** Implementar o algoritmo *Rsync* (Rolling Hash / Adler-32). O `DeltaSyncService` precisará ler as assinaturas de blocos do arquivo de origem e destino, calcular as diferenças e transferir apenas os deltas.
    *   *Nota:* Requer alto conhecimento computacional, mas economiza 99% de banda em arquivos grandes.

### 4.2. FolderFlow Windows Service / Daemon
*   **Descrição:** Desacoplar o motor (Engine) da Interface Visual (UI).
*   **Implementação:**
    *   **Architecture:** Separar a `FolderFlow.Application` e `FolderFlow.Infrastructure` em um projeto `FolderFlow.Service` (Background Worker usando `Microsoft.Extensions.Hosting`).
    *   **Communication:** A `FolderFlow.App` (UI Avalonia) deixará de rodar as tarefas localmente e passará a se comunicar com o `FolderFlow.Service` via **gRPC** ou **Named Pipes** para enviar Jobs, receber Progressos e ler Logs.

---

## Estratégia de Arquitetura Limpa (Clean Architecture Impact)

Para suportar essas funcionalidades mantendo a organização:

1.  **Abstração é a Chave:** O `ExecutionEngine` **não** deve saber se está copiando para C:\ ou para um SFTP, ou se está zipando. Tudo deve depender de injeção de `IFileOperator` (Local, Sftp, Zip).
2.  **Segurança de Credenciais:** As senhas para SFTP, Webhooks e Criptografia AES nunca devem ser salvas em texto limpo nos JSONs de Jobs. Deve-se implementar uma interface `ISecretVaultStore` atrelada à API de proteção de dados do OS (DPAPI no Windows, Keychain no Linux/Mac).
3.  **Filas Paralelas:** Para a concorrência (Fase 3), o `ChannelJobQueue` atual processa um Job por vez. Ele precisará ser atualizado para um roteador de Jobs com `ActionBlock` ou controle de `SemaphoreSlim`.

---
*Plano gerado para evoluir o FolderFlow rumo à categoria Enterprise/Pro.*