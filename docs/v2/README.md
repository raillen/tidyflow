# TidyFlow v2 — Documentação técnica

**Status:** `Em desenvolvimento` — o código atual já é **Tauri 2 + Svelte 5 + Rust**.
Os documentos e planos que citam Avalonia/.NET são legado da v1.

Esta pasta descreve a arquitetura e os contratos do TidyFlow v2.

---

## Para quem é cada documento

| Documento | Leitor | Conteúdo |
|-----------|--------|----------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Mantenedor | Camadas, fluxos, módulos |
| [STACK.md](./STACK.md) | Dev frontend/backend | Bibliotecas, versões, propósito |
| [DOMAIN.md](./DOMAIN.md) | Dev core | Entidades, regras, invariantes |
| [API-IPC.md](./API-IPC.md) | Full-stack | Commands, events, tipos |
| [UI-DESIGN-SYSTEM.md](./UI-DESIGN-SYSTEM.md) | Dev UI / design | Tokens, componentes, a11y, motion |
| [FRONTEND.md](./FRONTEND.md) | Dev frontend | Estrutura de `src/`, painéis, erros, i18n, a11y |
| [TESTING-E2E.md](./TESTING-E2E.md) | Dev / QA | Playwright headless, mock de IPC, cobertura |
| [PROJECT-STRUCTURE.md](./PROJECT-STRUCTURE.md) | Todos | Pastas, scripts, CI |
| [ROADMAP.md](./ROADMAP.md) | PM / dev | Fases 0–5, Definition of Done |

Decisão de stack: [ADR-002](../adr/ADR-002-stack-tauri-svelte-rust.md)

Contratos legados (v1 JSON): [`docs/specs/contratos-principais.md`](../specs/contratos-principais.md) — usar só como referência de domínio.

---

## Princípios (resumo)

1. **Core sem UI** — Rust executa; Svelte observa.
2. **Um banco** — SQLite; export JSON é backup, não runtime.
3. **Paths seguros** — toda operação passa por `PathPolicy`.
4. **Eventos, não polling** — progresso via Tauri events.
5. **Feature completa ou inexistente** — sem placeholders na UI.

---

## Comandos rápidos

```bash
# Na raiz do monorepo
pnpm install
pnpm dev              # Tauri + SvelteKit
pnpm test:rust        # cargo test --workspace
pnpm test             # Vitest (apps/desktop)

# Só UI
pnpm --filter @tidyflow/desktop check
```

**Nota:** o `pnpm-workspace.yaml` inclui `allowBuilds.esbuild: true` (pnpm 10+).

---

## Estado atual (0.2.1-alpha)

- [x] Monorepo pnpm + Cargo workspace
- [x] Crates Rust: domain, application, infrastructure, core
- [x] Tauri app com comandos de jobs, blueprints, auditoria e admin
- [x] UI Svelte 5 com rotas de automação, blueprints, histórico e settings
- [x] SQLite com migrations e stores
- [x] Blueprints de arquivos e pastas, variáveis, preview e auto-correct
- [x] Testes Rust de domínio/aplicação/core + Vitest dos contratos
- [ ] Testes golden completos por transform
- [ ] Cobertura Playwright de fluxos completos
- [ ] Alinhamento final de toda a documentação legada

---

## Mapa mental

```
Usuário → Svelte UI → Tauri IPC → autoflow-core (Rust)
                                      ├── SQLite
                                      ├── File watcher
                                      ├── Scheduler
                                      └── Execution engine
```

---

## Histórico

| Versão doc | Data | Nota |
|------------|------|------|
| 1.0 | 2026-06-19 | Spec inicial Tauri + Svelte |
