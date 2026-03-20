# Especificacao de Contratos: FolderFlow

- Status: `Draft`
- Data: 2026-03-20
- Tipo de contrato: `persistencia`, `interfaces publicas`

---

## 1. Regras gerais

- encoding: `UTF-8`
- versao de schema: `1.0.0`
- timestamps: `ISO 8601`
- IDs: `GUID` (string)
- compatibilidade: Backward compatibility deve ser mantida.

---

## 2. Contrato A: Job Configuration (`job.json`)

### Objetivo

Define a configuração de uma tarefa de automação.

### Campos ou estrutura

| Campo | Tipo | Obrigatorio | Regra |
|---|---|---|---|
| `Id` | `string` | Sim | GUID único para o Job. |
| `Name` | `string` | Sim | Nome descritivo dado pelo usuário. |
| `SourcePath` | `string` | Sim | Caminho da pasta de origem. |
| `TargetPath` | `string` | Sim | Caminho da pasta de destino. |
| `Mode` | `enum` | Sim | `Copy` ou `Move`. |
| `Filters` | `array` | Não | Lista de regras de inclusão/exclusão (ex: `*.pdf`). |
| `ConflictMode` | `enum` | Sim | `Overwrite`, `Skip`, `Rename`. |
| `Schedule` | `object` | Não | Configurações de agendamento (tipo, intervalo). |
| `WatchEnabled` | `boolean` | Sim | Ativa/Desativa o Watch Folder para este Job. |

### Exemplo

```json
{
  "Id": "550e8400-e29b-41d4-a716-446655440000",
  "Name": "Backup de PDFs",
  "SourcePath": "C:/Users/User/Downloads",
  "TargetPath": "D:/Backups/PDFs",
  "Mode": "Copy",
  "Filters": ["*.pdf"],
  "ConflictMode": "Rename",
  "WatchEnabled": true
}
```

---

## 3. Contrato B: App Settings (`settings.json`)

### Objetivo

Configurações globais do aplicativo.

### Campos ou estrutura

| Campo | Tipo | Obrigatorio | Regra |
|---|---|---|---|
| `Theme` | `enum` | Sim | `Light`, `Dark`, `System`. |
| `Language` | `string` | Sim | Código de cultura (ex: `pt-BR`, `en-US`). |
| `ShowNotifications` | `boolean` | Sim | Ativa/Desativa alertas de sistema. |
| `StartAtStartup` | `boolean` | Sim | Inicializa o app com o SO. |

### Exemplo

```json
{
  "Theme": "Dark",
  "Language": "pt-BR",
  "ShowNotifications": true,
  "StartAtStartup": false
}
```

---

## 4. Relacao entre contratos

- Um arquivo `settings.json` por usuário.
- Múltiplos arquivos `job.json` dentro de uma pasta configurada.

---

## 5. Regras de validacao aplicacional

- `SourcePath` e `TargetPath` não podem ser iguais.
- O app deve garantir que o diretório de destino existe ou criá-lo no início da execução.
- Caminhos devem ser validados de acordo com o sistema operacional local.
