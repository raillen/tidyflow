# Spec — Watch (monitoramento de pastas)

**Status:** `Implementado`  
**Escopo:** Detecção de mudanças em `source_path` → enfileira execução do job.

---

## 1. Modelo

| Campo | Tipo | Default |
|-------|------|---------|
| `enabled` | bool | false |
| `settle_seconds` | u32 (1–60) | 2 |
| `detection` | `WatchDetectionMode` | `realtime` |
| `events` | `WatchEventKind[]` | `[create]` |

### WatchDetectionMode

| Modo | Uso | Performance |
|------|-----|-------------|
| `realtime` | Disco local SSD/HDD | Menor latência, CPU baixo |
| `polling { interval_secs }` | Rede (UNC), USB instável | CPU proporcional ao intervalo |
| `hybrid { poll_interval_secs }` | Pastas de rede + eventos locais | Melhor confiabilidade; polling leve de backup |

### WatchEventKind

`create` | `modify` | `remove` | `rename`

---

## 2. Regras de produto

1. **Watch ↔ Schedule mutuamente exclusivos** — watch ativo desliga schedule (e vice-versa) em `normalize()`.
2. **Eventos** — usuário escolhe quais disparam o job (checkboxes na UI).
3. **Debounce** — `settle_seconds` agrupa rajadas numa única execução (`notify-debouncer-full`).
4. **Job já em execução** — ignora novo enqueue (coalescer; fila já deduplica por `job_id`).
5. **Recursivo** — segue `job.filters.recursive` para profundidade do watcher.
6. **Multi-folder** — **fora do MVP** (ver §5).

---

## 3. Arquitetura (Opção C)

- Config em `job.watch` (UI: painel **Monitoramento** no editor).
- Engine central `WatchService` em `autoflow-core` (sobrevive com app aberto).
- `sync_all()` no startup; `sync_job()` após CRUD.

---

## 4. Performance (decisões fechadas)

| Pergunta | Decisão |
|----------|---------|
| Quais eventos? | Usuário define; default só `create` |
| Rajada de arquivos? | Debounce `settle_seconds` (default 2s) |
| Job rodando? | Não enfileira duplicata |
| API de detecção? | `notify-debouncer-full`; polling/hybrid para rede |

---

## 5. Multi-folder (futuro)

Vários pares origem→destino no mesmo job é **viável** (v2.1+), mas:

- N watchers + N×M combinações de path aumentam CPU e complexidade de conflito.
- Recomendação: um job = um par; multi-folder via **vários jobs** ou blueprint dedicado.

---

## 6. i18n

Big bang Paraglide **após** features — strings hardcoded PT por enquanto.

---

## 7. Testes (prioridade atual)

Integração em `autoflow-application/tests/`: **pack** + **scripts** (antes de watch E2E).
