# TidyFlow — Estado Atual do Projeto

**Data:** 30 de outubro de 2026
**Branch:** `main` (HEAD `c3e8ace` + alterações no working tree)

## Verificações Verdes

- `cargo fmt --all -- --check` ✅
- `cargo clippy --workspace --all-targets -- -D warnings` ✅
- `cargo test --workspace` ✅ (104 testes)
- `pnpm --filter @tidyflow/desktop check` ✅ (0 erros / 0 warnings)
- `pnpm --filter @tidyflow/desktop test` ✅ (28 testes Vitest)

## Concluído nesta rodada

### i18n completo (UI + contratos)
- Dicionários `pt-BR`/`en-US` espelhados (882 chaves).
- `localeStore`, `t(key, params)`, `currentLocale()` para `Intl`.
- Listas de opções/constantes de UI convertidas em funções reativas ao locale:
  `navItems()`, `weekdays()`, `templateSyntaxLegend()`, `templateQuickPatterns()`,
  `templateTokenGroups()`, `templateTransformGroups()`, `templateRegexGroups()`,
  `counterScopeOptions()`, `changelogEntries()`, `folderActionOptions()`.
- Chaves dinâmicas via template literal:
  `t(\`blueprints.template.token.${name}.label\`)`.
- Removidas 36 chaves órfãs; `LANGUAGE_OPTIONS` reduzido a pt-BR/en-US.
- Varredura final: nenhuma string PT/EN hardcoded na UI (exceto marca/dados bancários).

### Backend (Rust)
- `JobRunDeps { data_dir, jobs, audit, rollback }` (elimina `too_many_arguments`).
- `SmtpConnection::Tls` boxed (large enum variant).
- `NamingPattern.template_override` na UI (`FolderStructureEditor`).
- `DomainError::FolderOutOfPattern` usado em `handle_folder_auto_correct`.
- Single-instance lock via `try_lock()` em `tidyflow.lock` (por data dir).
- PIN bloqueia ao ir para a bandeja (`CloseRequested` + tray `hide`).
- `SecretVault` (AES-256-GCM via `ring`) criptografa segredos de
  notificação por job em repouso (webhook URL, bot token, chat ID, headers,
  senha de pack), com chave mestre no OS keyring ou arquivo `0600`.
- `namingPatternSchema` aceita `null` do backend (transform).

### Frontend
- `BlueprintTemplatePanel`, `BlueprintsModulePage`, `+layout.svelte` localizados.
- `BlueprintEditor`/`FolderStructureEditor` com template de renomeação.
- `Modal` close button localizado.

## Pendências Funcionais Restantes

1. **Rollback de blueprints** — estender `RollbackManifest`/`rollback.rs`.
2. **Chave mestre de criptografia** além da senha por job (UI + storage).
3. **Notificações admin remoto** — TLS/UI de servidor (hoje sem TLS).
4. **Updater + E2E Playwright** — `tauri-plugin-updater`/`autostart` não
   disponíveis offline; Playwright não instalado.
5. **Assinatura de release** — automatizar no CI (requer secrets).
6. **Testes golden** por transform/regex preset do tokenizer.
7. **PIN na bandeja** — refinamento: emitir `security_lock` também ao
   minimizar (hoje apenas close-to-tray e tray hide).

## Notas Operacionais

- `HEAD` inicial era `c3e8ace`; todas as correções desta rodada estão no
  working tree (sem commit) até o push atual.
- Disco raiz quase cheio durante o desenvolvimento: use `CARGO_INCREMENTAL=0`.
- `target/debug/incremental` pode ser removido para liberar espaço.
- `tauri build` e assinatura requerem secrets/certificados não presentes.
