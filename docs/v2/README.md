# AutoFlow v2 — Documentação técnica

**Status:** `Planejado` (spec para reimplementação; código legado continua em Avalonia/.NET)

Esta pasta descreve como reconstruir o AutoFlow com **Tauri 2 + Svelte 5 + Rust**, de forma organizada e testável.

---

## Para quem é cada documento

| Documento | Leitor | Conteúdo |
|-----------|--------|----------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Mantenedor | Camadas, fluxos, módulos |
| [STACK.md](./STACK.md) | Dev frontend/backend | Bibliotecas, versões, propósito |
| [DOMAIN.md](./DOMAIN.md) | Dev core | Entidades, regras, invariantes |
| [API-IPC.md](./API-IPC.md) | Full-stack | Commands, events, tipos |
| [UI-DESIGN-SYSTEM.md](./UI-DESIGN-SYSTEM.md) | Dev UI / design | Tokens, componentes, a11y, motion |
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
pnpm --filter @autoflow/desktop check
```

**Nota:** o `pnpm-workspace.yaml` inclui `allowBuilds.esbuild: true` (pnpm 10+).

---

## Estado atual (Fase 0 — iniciado)

- [x] Branch `v2/tauri-svelte`
- [x] Monorepo pnpm + Cargo workspace
- [x] Crates Rust: domain, application, infrastructure, core
- [x] Tauri app com commands `health`, `settings_get`, `settings_update`
- [x] UI: sidebar, 5 rotas, tema light/dark, settings funcional
- [x] Testes Rust (3) + Vitest Zod (2)
- [ ] SQLite migrations (próximo passo)
- [ ] CI GitHub Actions

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
