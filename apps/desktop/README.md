# TidyFlow Desktop (v2)

Shell **Tauri 2 + SvelteKit (SPA)** do TidyFlow v2.

## Desenvolvimento

Na raiz do monorepo:

```bash
pnpm install
pnpm dev
```

Ou dentro desta pasta:

```bash
pnpm install
pnpm tauri dev
```

## Testes

```bash
pnpm test
pnpm check
cargo test --workspace
```

## Estrutura

- `src/` — UI SvelteKit (`ssr = false`, adapter-static)
- `src-tauri/` — commands Tauri → `autoflow-core`
- `../../crates/` — domínio e serviços Rust

Documentação: [`docs/v2/README.md`](../../docs/v2/README.md)
