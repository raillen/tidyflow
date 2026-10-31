# TidyFlow — Estado Atual do Projeto

**Data:** 30 de outubro de 2026
**Branch:** `main` (HEAD `c3e8ace` + working tree)

## Verificações Verdes

- `cargo fmt --all -- --check` ✅
- `cargo clippy --workspace --all-targets -- -D warnings` ✅
- `cargo test --workspace` ✅ (104 testes)
- `pnpm --filter @tidyflow/desktop check` ✅ (0 erros / 0 warnings)
- `pnpm --filter @tidyflow/desktop test` ✅ (28 testes Vitest)
- `pnpm --filter @tidyflow/desktop test:e2e` ✅ (77 testes Playwright)
- `pnpm --filter @tidyflow/desktop build` ✅

## Concluído nesta rodada

### Rollback de blueprints
- `RollbackManifest.blueprint_id` + `RollbackItem.is_directory`.
- `rollback_blueprint()` no application layer, `latest_for_blueprint` no store.
- Apply de blueprints (arquivo e pasta) grava manifesto com `execution_id`.
- Comando Tauri `blueprints_rollback` + `rollbackBlueprint()` no client IPC.

### Criptografia condicionada à configuração
- `SqliteJobStore::with_vault_enabled` respeita `security.encryption_enabled`.
- Antes, o vault selava sempre; agora o toggle da UI tem efeito real.

### TLS no admin server
- HTTPS via `AUTOFLOW_ADMIN_TLS_CERT` / `AUTOFLOW_ADMIN_TLS_KEY`.
- `TlsListener` implementando `axum::serve::Listener`; fallback HTTP sem os vars.

### PIN ao minimizar
- `WindowEvent::Resized` detecta minimização e emite `security_lock`.

### Updater + E2E
- `tauri-plugin-updater` adicionado e registrado.
- Playwright instalado; suíte headless com mock de IPC (77 testes).

### Varredura de bugs (corrigidos)

| Bug | Impacto | Correção |
|-----|---------|----------|
| `instanceof Error` não captura erros de IPC | Mensagem real do backend descartada em 40 pontos | `lib/core/errors.ts` + `errorMessage()` |
| Bot token do Telegram em texto puro | Credencial exposta na tela | `type="password"` + `autocomplete="off"` |
| Handler de comando admin sem `try/catch` | Falha de IPC virava unhandled rejection | `runCommand()` centralizado |
| `message` apagada por `refresh()` no admin | Feedback do comando nunca aparecia | `refresh({ keepMessage })` |
| `$effect` sobrescrevia texto em digitação | Vírgula final apagada ao digitar extensões | Ressincroniza só quando divergem |
| Último evento de watch bloqueado | Checkbox dessincronizado do modelo | Checkbox desabilitado |
| `setWatchEnabled` de pasta sem `autoCorrect` | Blueprint de pasta não auto-corrigia | Delega a `defaultWatch()` |
| Botão remover só com ícone | Inacessível para leitor de tela | `aria-label` |
| `resizing` lido não-reativamente | Resize travava após primeiro movimento | Cópia local do edge |
| `input` sem `type` (45 ocorrências) | Semântica/autocomplete incorretos | `type="text"` explícito |
| Strings hardcoded PT em 8 pontos | i18n incompleto | Todas via `t()` |

### Clean-code / componentização

- **`JobEditor`: 1710 → 118 linhas.** Extraídos 6 painéis em
  `components/jobs/panels/`.
- Opções de UI movidas para `contracts/job-options.ts` (funções reativas).
- CSS compartilhado em `lib/styles/editor.css` (necessário: estilo de componente
  Svelte é scoped e não alcança filhos).
- `BlueprintsModulePage.svelte` removido — 450 linhas mortas duplicando a rota.
- `BlueprintEditor` e `FolderStructureEditor` passam a reutilizar
  `defaultWatch`/`watchEvents`/`detectionOptions`.
- 120 chaves i18n órfãs removidas; paridade pt-BR/en-US em 852.

### Documentação

- `docs/v2/FRONTEND.md` — estrutura, painéis, erros, i18n, a11y, checklist.
- `docs/v2/TESTING-E2E.md` — mock de IPC, cobertura, como escrever testes.
- `docs/v2/README.md` e `README.md` — mapa atualizado.
- Script `pnpm test:e2e` na raiz.

## Pendências Restantes

1. **Chave mestre por passphrase** — hoje a chave é aleatória no keyring/arquivo.
2. **Testes golden do tokenizer** por transform/regex preset.
3. **Assinatura de release** — requer secrets/certificados.
4. **`tauri build`** — requer certificados; o build web (`vite build`) passa.

## Notas Operacionais

- `HEAD` inicial era `c3e8ace`; correções no working tree.
- Disco raiz quase cheio: use `CARGO_INCREMENTAL=0`.
- `target/debug/incremental` pode ser removido para liberar espaço.
- E2E não inicia o dev server: rode `pnpm dev` antes, ou
  `npx playwright test` com o Vite já em `:1420`.
