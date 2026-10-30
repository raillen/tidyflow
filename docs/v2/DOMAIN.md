# Domínio — AutoFlow v2

**Status:** `Implementado` · Blueprints: ver `BLUEPRINT-SPEC.md`

Regras de negócio puras. Implementação Rust em `crates/autoflow-domain`. Tipos espelhados em TS via Specta + Zod.

---

## 1. Entidades principais

### Job

Representa um fluxo de transferência (cópia ou movimentação).

| Campo | Tipo | Regras |
|-------|------|--------|
| `id` | UUID | Imutável |
| `name` | string | 1–120 chars, trim |
| `source_path` | PathBuf | Obrigatório, deve existir ao executar |
| `target_path` | PathBuf | Obrigatório; criar se não existir |
| `mode` | `Copy \| Move` | |
| `trigger` | enum | Ver §2 |
| `filters` | FileFilter | |
| `conflict` | `Skip \| Overwrite \| Rename` | |
| `options` | TransferOptions | |
| `schedule` | ScheduleRule? | Obrigatório se trigger = Schedule |
| `watch` | WatchConfig? | Obrigatório se trigger = Watch |
| `scripts` | ScriptsConfig | pre/post opcionais |
| `notify` | NotifyConfig | webhook por job |
| `enabled` | bool | default true |
| `last_run` | DateTime? | |
| `next_run` | DateTime? | Calculado |

### Blueprint

Regra de organização em uma pasta raiz.

| Campo | Tipo | Regras |
|-------|------|--------|
| `id` | UUID | |
| `name` | string | 1–120 caracteres |
| `root_path` | string | Pasta raiz monitorada |
| `kind` | `File \| Folder` | File → arquivos; Folder → pastas |
| `search` | FileFilter | Filtros de arquivos (kind File) |
| `routing` | RoutingConfig | Template de caminho + criar pastas intermediárias |
| `operation` | `Move \| Copy` | Operação do blueprint de arquivos |
| `recursive` | bool | Busca recursiva |
| `conflict` | `Skip \| Overwrite \| Rename` | Estratégia de colisão |
| `rename_template` | TemplatePipeline? | Nome final opcional, independente do conflito |
| `folder_plan` | FolderPlan | Árvore de ações + variáveis (obrigatório no kind Folder) |
| `watch` | WatchConfig? | Automação reativa |
| `counter` | CounterConfig | Contador persistido |
| `enabled` | bool | |
| `last_run` | DateTime? | |

### AuditEntry (imutável)

| Campo | Tipo |
|-------|------|
| `id` | i64 autoincrement |
| `timestamp` | DateTime UTC |
| `job_id` | UUID? |
| `job_name` | string |
| `source_path` | string |
| `target_path` | string |
| `status` | AuditStatus |
| `file_size` | i64 |
| `duration_ms` | f64 |
| `details` | string? |

### RollbackManifest

| Campo | Tipo |
|-------|------|
| `job_id` | UUID |
| `executed_at` | DateTime |
| `items` | Vec\<RollbackItem\> |

---

## 2. Gatilhos (Trigger)

```rust
enum JobTrigger {
    Manual,
    Watch(WatchConfig),
    Schedule(ScheduleRule),
}

struct WatchConfig {
    settle_seconds: u32,      // default 1
    monitoring: RealTime | Polling { interval_secs: u32 },
}

enum ScheduleRule {
    Interval { minutes: u32 },
    Daily { at: TimeOfDay },
    Weekly { days: Vec<Weekday>, at: TimeOfDay },
}
```

**Invariante:** `Watch` e `Schedule` podem coexistir na v2? **Não no MVP** — um job tem um trigger primário; evita ambiguidade.

---

## 3. FileFilter (value object)

```rust
struct FileFilter {
    include_extensions: Vec<String>,  // ".pdf" lowercase
    exclude_patterns: Vec<String>,      // glob
    name_regex: Option<String>,
    min_size_bytes: Option<u64>,
    max_size_bytes: Option<u64>,
    modified_within_days: Option<u32>,
    content_contains: Option<String>,   // txt, md, log
    exif_date_range: Option<DateRange>,
    recursive: bool,
}
```

**ShouldProcess(file, filter)** — função pura testada unitariamente.

---

## 4. TransferOptions

```rust
struct TransferOptions {
    smart_sync: bool,
    delta_sync: bool,
    verify_hash: bool,
    use_trash: bool,
    encryption_key: Option<SecretString>,
    archive_format: None | Zip,
    retention: RetentionPolicy,
    max_bandwidth_mbps: u32,  // 0 = unlimited
}
```

---

## 5. AuditStatus (códigos estáveis)

| Código | Significado UI (i18n) |
|--------|----------------------|
| `COPIED` | Copiado |
| `MOVED` | Movido |
| `IGNORED` | Ignorado |
| `FAILED` | Falha |
| `ZIPped` | Compactado |
| `ORGANIZED` | Organizado (blueprint) |

UI traduz; core nunca envia string localizada.

---

## 6. Tokenizer (Blueprint)

Gramática real:

```
Template  := segmento*
segmento  := "{token}" | "[transform]" | "/preset/" | literal
token     := [A-Za-z_][A-Za-z0-9_]*
transform := nome [ "(" arg ("," arg)* ")" ]
preset    := nome de um preset conhecido
```

**Avaliação:** da esquerda para a direita, em um acumulador.

- `{token}` acrescenta o valor resolvido.
- `[transform]` e `/preset/` reescrevem todo o trecho acumulado.
- Literais são concatenados.

**Tokens:** `original`, `stem`, `ext`, `parent`, `grandparent`, `date`, `year`,
`month`, `day`, `hour`, `minute`, `second`, `counter`, `guid`, `index`.

**Transforms:** `upper`, `lower`, `capitalized`, `title`, `snake`, `camel`,
`pascal`, `kebab`, `trim`, `collapse_spaces`, `strip_symbols`,
`spaces_to_underscore`, `sanitize_windows_filename`, `take`, `skip`, `slice`.

**Regex presets:** `remove_digits`, `strip_special`, `extract_date`,
`remove_parens`, `digits_only`, `letters_only`.

**Variáveis:** `{projectName}` resolve contra `FolderPlan.variables`. O nome da
variável deve ser um identificador (`[A-Za-z_][A-Za-z0-9_]*`).

**Escapes e literais:**

- `{{` e `}}` representam `{` e `}` literais.
- `\[`, `\]`, `\/` e `\\` representam `[`, `]`, `/` e `\` literais.
- `\` antes de outros caracteres é preservado (caminhos Windows).
- `/nome/` só vira preset quando `nome` é um preset conhecido; `/docs/` é literal.
- `[...]` só vira transform quando começa com um transform conhecido; `[2024]` é literal.

**Invariantes:**

- Datas e horas usam o fuso local do sistema.
- Counter é persistido por `(blueprint_id, scope_key)` e reservado de forma atômica.
- Todo caminho resolvido é validado como relativo e sem componentes `..`.
- Falha de template aborta o plano antes de executar; o preview mostra o erro.

---

## 7. PathPolicy

```rust
trait PathPolicy {
    fn authorized_roots(&self) -> Vec<PathBuf>;
    fn is_authorized(&self, path: &Path) -> bool;
    fn assert_authorized(&self, path: &Path) -> Result<(), PathError>;
}
```

Raízes = `job.source`, `job.target`, `blueprint.root_path` (deduplicadas, existentes).

---

## 8. Erros de domínio

```rust
enum DomainError {
    PathNotAuthorized(PathBuf),
    SourceNotFound(PathBuf),
    JobDisabled,
    InvalidSchedule,
    TemplateParseError(String),
    ConflictUnresolved { path: PathBuf },
}
```

Mapear para códigos IPC — ver API-IPC.md.

---

## 9. Casos de uso (Application)

| Use case | Input | Output |
|----------|-------|--------|
| CreateJob | JobDraft | JobId |
| UpdateJob | Job | () |
| DeleteJob | JobId | () |
| RunJob | JobId | ExecutionId |
| CancelExecution | ExecutionId | () |
| SimulateJob | JobId | SimulationReport |
| RollbackJob | JobId | RollbackResult |
| ListAudit | AuditQuery | Page\<AuditEntry\> |
| ApplyBlueprint | BlueprintId, event_path? | BatchResult |
| BrowseDirectory | Path | DirectoryListing |

Cada use case = handler + testes de integração com tempdir.

---

## 10. Migração conceitual v1 → v2

| v1 (C#) | v2 (Rust) |
|---------|-----------|
| `Job.WatchEnabled` + Schedule | `JobTrigger` explícito |
| Status PT no engine | `AuditStatus` enum |
| JSON stores | SQLite rows |
| `ConflictMode` | `ConflictStrategy` (mesmos valores) |
| Blueprint folders list | `scaffolding` |

Script `tools/migrate-v1-json` — fora do MVP, documentado na Fase 5.
