# Estrutura do projeto — AutoFlow v2

**Status:** `Planejado`

Monorepo **pnpm + Cargo workspace**. UI em `apps/desktop`; core em `crates/*`.

---

## 1. Árvore de diretórios

```text
autoflow/
├── apps/
│   └── desktop/                    # Tauri + Svelte 5
│       ├── src/
│       │   ├── lib/
│       │   │   ├── components/
│       │   │   │   └── ui/         # Wrappers bits-ui
│       │   │   ├── core/
│       │   │   │   ├── ipc/        # invoke + listen
│       │   │   │   ├── stores/     # nanostores
│       │   │   │   └── i18n/       # paraglide
│       │   │   ├── features/
│       │   │   │   ├── dashboard/
│       │   │   │   ├── flows/
│       │   │   │   ├── blueprints/
│       │   │   │   ├── history/
│       │   │   │   ├── settings/
│       │   │   │   └── browser/    # Fase 5
│       │   │   └── contracts/      # bindings.ts + zod schemas
│       │   ├── routes/
│       │   │   ├── +layout.svelte
│       │   │   ├── +page.svelte              # Dashboard
│       │   │   ├── flows/+page.svelte
│       │   │   ├── blueprints/+page.svelte
│       │   │   ├── history/+page.svelte
│       │   │   └── settings/+page.svelte
│       │   ├── app.css             # tokens CSS
│       │   └── app.html
│       ├── src-tauri/
│       │   ├── src/
│       │   │   ├── main.rs
│       │   │   ├── lib.rs
│       │   │   └── commands/       # thin handlers → core
│       │   ├── capabilities/
│       │   ├── tauri.conf.json
│       │   └── Cargo.toml
│       ├── package.json
│       ├── svelte.config.js
│       ├── vite.config.ts
│       ├── tailwind.config.ts
│       └── tsconfig.json             # strict: true
│
├── crates/
│   ├── autoflow-domain/
│   ├── autoflow-application/
│   ├── autoflow-infrastructure/
│   │   └── migrations/
│   └── autoflow-core/              # AppState + workers
│
├── packages/
│   └── ts-config/                  # shared tsconfig (opcional)
│
├── docs/
│   ├── v2/                         # esta documentação
│   ├── adr/
│   └── specs/schemas/
│
├── tools/
│   └── migrate-v1-json/            # Fase 5
│
├── Cargo.toml                      # workspace
├── pnpm-workspace.yaml
├── rust-toolchain.toml
└── README.md
```

---

## 2. Scripts principais

### `apps/desktop/package.json`

```json
{
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "check": "svelte-check --tsconfig ./tsconfig.json && tsc --noEmit",
    "test": "vitest run",
    "test:e2e": "playwright test",
    "lint": "eslint . && prettier --check .",
    "format": "prettier --write ."
  }
}
```

### Raiz

```json
{
  "scripts": {
    "dev": "pnpm --filter desktop dev",
    "test": "pnpm --filter desktop test && cargo test --workspace",
    "check": "pnpm --filter desktop check && cargo clippy --workspace -- -D warnings"
  }
}
```

---

## 3. TypeScript strict (`tsconfig.json`)

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitOverride": true,
    "exactOptionalPropertyTypes": true,
    "moduleResolution": "bundler",
    "paths": {
      "$lib/*": ["./src/lib/*"]
    }
  }
}
```

Regra skill **global-typescript-strict-architecture**: `unknown` na borda IPC → Zod → tipo de domínio UI.

---

## 4. Tauri capabilities (segurança)

Princípio de least privilege em `capabilities/default.json`:

- `dialog:allow-open`, `dialog:allow-save`
- `notification:default`
- `shell:allow-open` — apenas paths validados pelo core
- **Negar** `fs:default` na UI — IO via commands Rust

---

## 5. CI sugerido

```yaml
# .github/workflows/ci.yml (resumo)
jobs:
  rust:
    - cargo fmt --check
    - cargo clippy -- -D warnings
    - cargo test
  ui:
    - pnpm install
    - pnpm check
    - pnpm test
    - pnpm exec playwright install --with-deps
    - pnpm test:e2e
```

---

## 6. Convenções de código

| Área | Convenção |
|------|-----------|
| Rust | `snake_case`, errors com `thiserror` |
| TS/Svelte | `camelCase` vars, `PascalCase` components |
| Arquivos Svelte | `JobCard.svelte`, co-localized `*.test.ts` |
| Features | Uma pasta por bounded context |
| IPC | 1 command por use case; sem “god commands” |

---

## 7. Onde cada skill se aplica

| Skill | Aplicação |
|-------|-----------|
| global-typescript-strict-architecture | `contracts/`, IPC client, forms Zod |
| global-living-docs | Atualizar `docs/v2/*` a cada feature |
| minimalist-ui | UI-DESIGN-SYSTEM + revisão visual PR |
| global-clean-code | Rust handlers finos; funções < 40 linhas |
| global-rust-idiomatic-safety | domain/infrastructure crates |
