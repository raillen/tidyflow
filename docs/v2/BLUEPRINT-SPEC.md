# Spec — Blueprints (organização)

**Status:** `Aprovado — pronto para implementação`  
**Módulo:** separado de Fluxos (`/blueprints`), infra compartilhada (Watch, PathPolicy, Audit).

---

## 1. Visão

Blueprint organiza arquivos ou pastas **dentro de uma raiz autorizada** (`root_path`), via:

- **Tokenizer** — renomear e montar caminhos relativos
- **Routing** — template gera subpastas + move/cópia com regras de busca
- **Scaffolding** — árvore de pastas (principalmente blueprint de pastas)
- **Watch** — automação reativa (mesmos modos de detecção dos fluxos)
- **Manual** — aplicar lote / simular

Fluxos = transferência A→B. Blueprints = ordenação e roteamento **in-place ou intra-raiz**.

---

## 2. Tipos de blueprint (separados no módulo)

Dois tipos distintos na UI e no domínio — **não** combinados num único registro.

### 2.1 `BlueprintKind::File`

Organiza **arquivos** sob `root_path`.

| Bloco | Descrição |
|-------|-----------|
| `root_path` | Pasta raiz monitorada / escopo de busca |
| `search` | `FileFilter` — extensões, glob, regex, datas, recursivo, etc. |
| `routing` | Template de caminho relativo (subpastas + nome final) |
| `operation` | `move` \| `copy` — aplica após resolver destino |
| `recursive` | Busca e aplicação recursiva na raiz (default true) |
| `conflict` | Estratégia de colisão (§5) |
| `rename_template` | Pipeline tokenizer só para **nome do arquivo** (opcional se routing já incluir nome) |
| `watch` | `WatchConfig?` — mesma struct dos fluxos |
| `enabled` | bool |

**Fluxo de apply:** buscar arquivos → `simulate` plano `(origem → destino relativo)` → executar move/copy.

### 2.2 `BlueprintKind::Folder`

Organiza **pastas** sob `root_path`.

| Bloco | Descrição |
|-------|-----------|
| `root_path` | Raiz |
| `search` | Filtro de pastas (profundidade, nome regex, presets exclude) |
| `folder_plan` | Árvore de scaffolding + routing de pastas |
| `routing` | Template para caminho relativo da pasta |
| `operation` | `move` \| `copy` (reorganizar árvore) |
| `recursive` | Processar subárvore |
| `conflict` | §5 |
| `rename_template` | Tokenizer para **nome da pasta** |
| `watch` | `WatchConfig?` |
| `enabled` | bool |

**Fluxo de apply:** identificar pastas candidatas → simular movimentos/rename → aplicar.

---

## 3. Entidade `Blueprint`

```rust
struct Blueprint {
    id: Uuid,
    name: String,
    kind: BlueprintKind,           // file | folder
    root_path: String,
    search: FileFilter,
    routing: RoutingConfig,
    operation: BlueprintOperation, // move | copy
    recursive: bool,
    conflict: ConflictStrategy,    // skip | overwrite | rename
    rename_template: Option<TemplatePipeline>,
    folder_plan: Option<FolderPlan>, // só kind == folder
    watch: Option<WatchConfig>,
    counter: CounterConfig,
    enabled: bool,
    last_run: Option<DateTime<Utc>>,
}
```

### RoutingConfig

```rust
struct RoutingConfig {
    /// Template gera path relativo completo (pastas + nome).
    /// Ex.: "{year}/{month}/{parent}/{original}"
    path_template: TemplatePipeline,
    /// Criar pastas intermediárias ausentes (default true).
    create_intermediate_dirs: bool,
}
```

---

## 4. Tokenizer (motor de templates)

Domínio puro: `crates/autoflow-domain/src/tokenizer/`.

### 4.1 Pipeline

```
TemplatePipeline = Vec<Segment>
Segment = Token | Transform
```

Avaliação: `evaluate(pipeline, &TokenContext) -> Result<String, TokenError>`

### 4.2 Toolbox

**Tokens:** `original`, `stem`, `ext`, `parent`, `grandparent`, `date`, `year`, `month`, `day`, `hour`, `minute`, `second`, `counter`, `guid`, `index`

**Estilo:** `upper`, `lower`, `capitalized`, `title`, `snake`, `camel`, `pascal`, `kebab`

**Substring:** `take(n, from)`, `skip(n, from)`, `slice(from, to)`

**Regex:** custom + presets:
- `remove_digits`, `strip_special`, `extract_date`, `remove_parens`, `digits_only`, `letters_only`

**Limpeza:** `trim`, `collapse_spaces`, `strip_symbols`, `spaces_to_underscore`, `sanitize_windows_filename`

### 4.3 Counter

`CounterConfig`: escopo `global` | `per_day` | `per_folder` | `per_parent`  
Persistência: tabela `blueprint_counters(blueprint_id, scope_key, value)`.

### 4.4 Preview

`blueprints_preview_template` — entrada: pipeline JSON + `sample_path` → `{ resultPath, resultName, valid, warnings[] }`.

Testes golden obrigatórios por transform (fase implementação; suite completa depois).

---

## 5. Colisão de nomes

Reutiliza `ConflictStrategy` do domínio de jobs:

| Valor | Comportamento |
|-------|----------------|
| `skip` | Não altera destino existente; registra aviso |
| `overwrite` | Substitui arquivo/pasta destino |
| `rename` | Sufixo incremental `(1)`, `(2)`, … |

Configurável **por blueprint** na UI (select igual ao fluxo).

Simulação lista colisões antes de apply manual.

---

## 6. Watch

Mesma struct `WatchConfig` de `docs/v2/WATCH-SPEC.md`:

- Detecção: `realtime` | `polling` | `hybrid`
- Eventos selecionáveis pelo usuário
- `settle_seconds` debounce
- Recursividade alinhada a `blueprint.recursive`

**Engine:** estender `WatchService` com `WatchTarget::Blueprint(Uuid)`.

- Evento → debounce → `ApplyBlueprint` (não enfileira job de transferência)
- Blueprint desabilitado ou sem watch → unregister

Schedule em blueprint: **fora do MVP** (watch + manual).

---

## 7. Operações: busca, cópia, move recursivo

### 7.1 Busca

Reutiliza `FilterEngine` / `collect_files` ou variante `collect_dirs` para kind folder.

Respeita `recursive`, `max_depth`, presets exclude (`node_modules`, `.git`, …).

### 7.2 Apply pipeline

```
1. validate root_path + PathPolicy
2. collect candidates (search)
3. build plan: Vec<PlanItem { source, dest_relative, action }>
4. resolve conflicts (dry-run)
5. create intermediate dirs (routing)
6. execute move | copy per item
7. audit ORGANIZED per item
8. increment counters
```

### 7.3 Simulação

`blueprints_simulate(id)` → `{ matched, skipped, plan_sample[], warnings[], collisions[] }`  
Mesmo padrão mental que `jobs_simulate`.

---

## 8. Segurança

- Todo destino resolvido deve estar **sob** `canonicalize(root_path)`
- Rejeitar `..`, paths absolutos no template, caracteres proibidos Windows
- `PathPolicy` inclui raízes de blueprints para browse seguro futuro
- Watch só registra se `root_path` existe e blueprint `enabled`

---

## 9. Persistência SQLite

Migration `003_blueprints.sql`:

```sql
CREATE TABLE blueprints (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,          -- file | folder
  payload TEXT NOT NULL,       -- JSON Blueprint
  enabled INTEGER NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE blueprint_counters (
  blueprint_id TEXT NOT NULL,
  scope_key TEXT NOT NULL,
  value INTEGER NOT NULL,
  PRIMARY KEY (blueprint_id, scope_key)
);
```

Audit: `AuditEntry` ganha `blueprint_id` opcional; status `Organized`.

---

## 10. IPC (Tauri)

| Command | Descrição |
|---------|-----------|
| `blueprints_list` / `_get` / `_create` / `_update` / `_delete` | CRUD |
| `blueprints_simulate` | Dry-run plano |
| `blueprints_apply` | Executa organização |
| `blueprints_preview_template` | Preview tokenizer |
| `blueprints_preview_plan` | Preview árvore de pastas (kind folder) |

Evento opcional: `blueprint_completed { blueprint_id, processed, failed }`.

---

## 11. UI (`/blueprints`)

### Lista

- Tabs ou filtro: **Arquivos** | **Pastas**
- Badge: Watch ativo, kind, última execução
- Ações: Editar, Simular, Aplicar, Excluir

### Editor (modal large, nav lateral)

| Nav | File | Folder |
|-----|------|--------|
| Geral | nome, root, enabled, operation, conflict, recursive | idem |
| Busca | FileFilter (reuso componentes) | filtro de pastas |
| Template | routing + rename + toolbox visual + preview | idem + preview path |
| Pastas | — | árvore scaffolding + routing drag-drop |
| Automação | WatchConfig panel (reuso) | idem |

Componentes compartilhados extraídos de JobEditor: `WatchConfigPanel`, filtros slim.

---

## 12. Interop com Fluxos

| Compartilhado | Não compartilhado |
|---------------|-------------------|
| WatchService, PathPolicy, Audit, FileFilter, ConflictStrategy | ExecutionPipeline copy A→B, TransferOptions, schedule job |
| Modal base, tokens CSS | Entidade Job, editor de fluxo |

Composição futura (v2.1): hook pós-fluxo “aplicar blueprint X no destino” — orquestração, sem merge de módulos.

---

## 13. Fases de implementação

1. **Domain** — `Blueprint`, `TemplatePipeline`, tokenizer + counter  
2. **Application** — simulate, apply, preview  
3. **Infrastructure** — migration 003, store  
4. **Core** — WatchTarget blueprint  
5. **UI** — CRUD + editor file + editor folder  
6. **Testes completos** — após features (decisão do projeto)

---

## 14. Decisões fechadas (2026-06-19)

| # | Decisão |
|---|---------|
| 1 | Tipos **separados** no módulo: blueprint de **arquivos** e de **pastas** |
| 2 | Routing: template gera **subpastas + move**; busca + **copy/move recursivo** configurável |
| 3 | Colisão: **configurável** (`skip` / `overwrite` / `rename`) |
| 4 | Watch: **mesmos 3 modos** de detecção + eventos que fluxos |
| — | Módulo **separado** de Fluxos; infra compartilhada |
| — | i18n Big Bang **depois** de todas as features |
