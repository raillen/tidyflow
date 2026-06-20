# API IPC — AutoFlow v2

**Status:** `Planejado`

Contrato entre **Svelte UI** e **Rust core** via Tauri 2. Tipos gerados com `tauri-specta`.

---

## 1. Convenções

| Item | Regra |
|------|--------|
| Commands | snake_case: `jobs_list`, `jobs_run` |
| Events | snake_case: `execution_progress` |
| IDs | UUID string |
| Datas | ISO 8601 UTC |
| Erros | `{ code: string, message: string, details?: object }` |
| Paginação | `{ items, total, page, page_size }` |

**Validação:** UI valida forms com Zod antes de invoke; Rust valida novamente no handler.

---

## 2. Commands — Jobs

### `jobs_list`

Lista todos os jobs.

```typescript
// Input: none
// Output:
type JobSummary[] = Array<{
  id: string;
  name: string;
  trigger: 'manual' | 'watch' | 'schedule';
  enabled: boolean;
  last_run: string | null;
  next_run: string | null;
}>;
```

### `jobs_get`

```typescript
// Input: { id: string }
// Output: Job (completo, ver DOMAIN.md)
```

### `jobs_create` / `jobs_update` / `jobs_delete`

```typescript
// Input: JobDraft | Job | { id: string }
// Output: { id: string } | void
// Side effect: invalida PathPolicy cache; reconfigura watch/schedule
```

### `jobs_run`

```typescript
// Input: { id: string }
// Output: { execution_id: string }
// Errors: JOB_DISABLED, SOURCE_NOT_FOUND, ALREADY_RUNNING
```

### `jobs_simulate`

```typescript
// Input: { id: string }
// Output: SimulationReport {
//   files_matched: number;
//   files_skipped: number;
//   sample: Array<{ source: string; target: string; action: string }>;
//   warnings: string[];
// }
```

### `jobs_simulate_draft`

```typescript
// Input: { job: Job } — job completo (rascunho do editor)
// Output: SimulationReport (mesmo shape de jobs_simulate)
// Errors: VALIDATION, SOURCE_NOT_FOUND
```

---

## 3. Commands — Execução

### `executions_list_active`

```typescript
// Output: ExecutionState[] {
//   execution_id, job_id, job_name, current_file, percent, bytes_per_sec, recent_log: string[]
// }
```

### `executions_cancel`

```typescript
// Input: { execution_id: string }
```

### `queue_pause` / `queue_resume` / `queue_stop_all`

Controle global da fila.

---

## 4. Commands — Blueprints

### `blueprints_list` / `blueprints_get` / `blueprints_create` / `blueprints_update` / `blueprints_delete`

CRUD padrão.

### `blueprints_apply_batch`

```typescript
// Input: { id: string }
// Output: { processed: number; errors: number }
```

### `blueprints_preview_template`

```typescript
// Input: { template: string; sample_path: string }
// Output: { result_name: string; valid: boolean; error?: string }
```

---

## 5. Commands — Audit

### `audit_query`

```typescript
// Input: AuditQuery {
//   job_id?: string;
//   status?: AuditStatus;
//   search?: string;
//   page: number;
//   page_size: number;  // max 100
// }
// Output: Paginated<AuditEntry>
```

### `audit_export_csv`

```typescript
// Input: AuditQuery
// Output: { path: string }  // arquivo temp; UI pede save via dialog
```

### `audit_clear`

```typescript
// Input: { confirm: true }
// Requer confirmação na UI
```

### `audit_stats`

```typescript
// Input: { job_id?: string; period: 'today' | '7d' | 'all' }
// Output: { copied, ignored, failed, bytes_processed }
```

---

## 6. Commands — Rollback

### `rollback_execute`

```typescript
// Input: { job_id: string }
// Output: { success: boolean; reverted_count: number }
```

---

## 7. Commands — Settings

### `settings_get` / `settings_update`

```typescript
type AppSettings = {
  theme: 'system' | 'light' | 'dark';
  language: 'pt-BR' | 'en-US' | ...;
  bandwidth_limit_mbps: number;
  max_parallel_files: number;
  log_retention_days: number;
  webhook_url?: string;
  webhook_type: 'generic' | 'discord' | 'slack';
  autostart: boolean;
  // ...
};
```

---

## 8. Commands — Browse seguro (Fase 5)

### `browse_list_directory`

```typescript
// Input: { path: string }
// Output: DirectoryListing {
//   path: string;
//   parent: string | null;
//   items: Array<{
//     name: string;
//     path: string;
//     is_directory: boolean;
//     size: number;
//     modified_at: string;
//   }>;
// }
// Errors: PATH_NOT_AUTHORIZED
```

### `browse_authorized_roots`

```typescript
// Output: string[]
```

---

## 9. Events (UI → listen)

### `execution_progress`

Emitido a cada ~200ms ou por arquivo.

```typescript
type ExecutionProgress = {
  execution_id: string;
  job_id: string;
  job_name: string;
  current_file: string;
  percent: number;
  bytes_per_sec: number;
  recent_log: string[];  // max 10 linhas
};
```

### `execution_completed`

```typescript
type ExecutionCompleted = {
  execution_id: string;
  job_id: string;
  success: boolean;
  processed: number;
  failed: number;
  error_message?: string;
};
```

### `queue_state_changed`

```typescript
{ paused: boolean; active_count: number }
```

### `blueprint_applied`

```typescript
{ blueprint_id: string; path: string; action: string }
```

### `path_authorization_changed`

UI recarrega roots autorizadas.

---

## 10. Códigos de erro

| Code | HTTP-like | Descrição |
|------|-----------|-----------|
| `PATH_NOT_AUTHORIZED` | 403 | Path fora das raízes |
| `JOB_NOT_FOUND` | 404 | |
| `JOB_DISABLED` | 409 | |
| `SOURCE_NOT_FOUND` | 422 | Origem inexistente |
| `ALREADY_RUNNING` | 409 | Job já na fila |
| `VALIDATION_ERROR` | 422 | Zod/specta mismatch |
| `INTERNAL_ERROR` | 500 | Logged com tracing |

---

## 11. Cliente TypeScript (padrão)

```typescript
// src/lib/core/ipc/events.ts
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ExecutionProgress } from '$lib/contracts/bindings';

export function onExecutionProgress(cb: (p: ExecutionProgress) => void): Promise<UnlistenFn> {
  return listen<ExecutionProgress>('execution_progress', (e) => cb(e.payload));
}
```

Registrar listeners no `+layout.svelte` root; alimentar nanostore `activeExecutions`.

---

## 12. Geração de tipos

```bash
# Após alterar commands Rust:
cargo run --bin export_bindings
# Gera apps/desktop/src/lib/contracts/bindings.ts
```

Commitar bindings gerados ou gerar no CI — escolher uma política e manter consistente.

---

## 13. Schema JSON (referência)

Ver [`docs/specs/schemas/ipc-v2.schema.json`](../specs/schemas/ipc-v2.schema.json) para estruturas base compartilháveis.
