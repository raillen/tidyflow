# ADR-002: Stack Tauri + Svelte + Rust (AutoFlow v2)

- **Status:** `Accepted`
- **Data:** 2026-06-19
- **Substitui parcialmente:** [ADR-001](./ADR-001-stack-tecnica.md) (stack Avalonia/.NET permanece como implementação legada)
- **Documentação v2:** [`docs/v2/README.md`](../v2/README.md)

---

## 1. Contexto

O AutoFlow v1 (.NET + Avalonia) entregou funcionalidades ricas, mas acumulou:

- UI acoplada a implementações concretas (ex.: `SqliteAuditService` nos ViewModels)
- Agent e interface no mesmo processo
- Persistência mista (JSON + SQLite) sem migrations claras
- Features documentadas como prontas, porém incompletas (File Browser, SMTP, stats)
- Autorização de paths frágil (`StartsWith` sem normalização)

A v2 deve preservar o domínio do produto com arquitetura mais assertiva, binário leve e base cross-platform.

---

## 2. Decisão

| Camada | Tecnologia |
|--------|------------|
| **Shell desktop** | Tauri 2 |
| **UI** | Svelte 5 + TypeScript strict + Vite |
| **Core / Agent** | Rust (tokio, sqlx, notify) |
| **IPC** | Tauri Commands + Events tipados (`tauri-specta`) |
| **Persistência** | SQLite (fonte única de verdade) |
| **Testes** | Vitest + Testing Library (UI), cargo test (Rust) |

### Regras operacionais

1. **UI nunca acessa filesystem diretamente** — só via commands Tauri.
2. **Rust core roda com ou sem janela** — tray + autostart; fila e watch continuam ativos.
3. **Contratos tipados** — TS gerado a partir de Specta; Zod na borda da UI para forms.
4. **SQLite + migrations** — jobs, blueprints, audit, settings, rollback.
5. **PathPolicy** — normalização com separador antes de autorizar qualquer path.
6. **Feature Done** — só quando command + teste + tela + doc existirem.

---

## 3. Consequências

### Positivas

- Binário pequeno e baixo uso de RAM vs Electron
- UI reativa (Svelte 5 runes) ideal para progresso e logs em tempo real
- Rust forte para I/O concorrente, watch e segurança de paths
- Cross-platform (Windows, Linux, macOS) com mesmo código
- Type-safety ponta a ponta (Rust → TS)

### Negativas

- Reescrita completa — não há migração automática trivial do Avalonia
- Curva Rust para equipe só C#
- Ecossistema desktop Tauri menor que .NET nativo no Windows
- Integrações Windows profundas (OneDrive hydration) exigem crates/plugins específicos

---

## 4. Alternativas consideradas

| Alternativa | Motivo da rejeição |
|-------------|-------------------|
| Manter Avalonia + refatorar | Menor risco, mas não resolve peso/ecossistema web UI; dívida permanece |
| Tauri + React | Válido; Svelte escolhido por bundle menor e menos boilerplate para UI observacional |
| Tauri + Solid | Equivalente; Svelte tem mais templates e componentes acessíveis (bits-ui) |
| Electron + Svelte | Contradiz objetivo de app leve |
| WinUI 3 + .NET Agent | Excelente no Windows, fraco cross-platform |

---

## 5. Impacto no Sprint 0

Ver [`docs/v2/ROADMAP.md`](../v2/ROADMAP.md) — Fase 0.

Entregáveis mínimos:

- Monorepo `apps/desktop` + `crates/autoflow-core`
- SQLite schema v1 + migrations
- Commands: `health`, `list_jobs`, `list_audit`
- Evento: `execution_progress`
- Tela shell com sidebar + rota Dashboard placeholder

---

## 6. Governança

- ADR-001 permanece como registro histórico da v1.
- Decisões de UI visual: [`docs/v2/UI-DESIGN-SYSTEM.md`](../v2/UI-DESIGN-SYSTEM.md)
- Contratos IPC: [`docs/v2/API-IPC.md`](../v2/API-IPC.md)
- Revisar este ADR se migrarmos IPC para gRPC sidecar separado do Tauri.
