# Stack e bibliotecas — AutoFlow v2

**Status:** `Planejado`

Lista curada para **Tauri 2 + Svelte 5**, com foco em interação, animação, acessibilidade e funcionalidades do produto.

---

## 1. Visão por camada

| Camada | Stack base |
|--------|------------|
| Desktop shell | Tauri 2, Rust 1.78+ |
| UI | Svelte 5, TypeScript 5.x strict, Vite 6 |
| Estilo | Tailwind CSS 4, tailwind-variants |
| Estado | nanostores, runed |
| Validação | Zod 3 |
| i18n | Paraglide-Svelte (inlang) |
| DB | SQLite via sqlx |
| Testes | Vitest, Testing Library, Playwright, cargo test |

---

## 2. Tauri — shell e integração OS

| Pacote | Uso no AutoFlow |
|--------|-----------------|
| `@tauri-apps/api` | invoke, listen, window, path |
| `@tauri-apps/plugin-dialog` | Selecionar pasta/arquivo (origem, destino, scripts) |
| `@tauri-apps/plugin-notification` | Toast nativo Windows/Linux/macOS |
| `@tauri-apps/plugin-autostart` | Iniciar com o sistema |
| `@tauri-apps/plugin-window-state` | Restaurar tamanho/posição da janela |
| `@tauri-apps/plugin-process` | Reiniciar app após update |
| `@tauri-apps/plugin-log` | Logs unificados com tracing Rust |
| `@tauri-apps/plugin-shell` | Abrir pasta no Explorer/Finder (com validação prévia) |
| `@tauri-apps/plugin-updater` | Auto-update (fase pós-MVP) |

**Rust (workspace):**

| Crate | Uso |
|-------|-----|
| `tauri` / `tauri-build` | Shell |
| `tauri-specta` + `specta` | Commands/events tipados → TS |
| `tokio` | Runtime async |
| `sqlx` | SQLite |
| `notify-debouncer-full` | Watch folder com debounce |
| `serde` / `serde_json` | Serialização |
| `tracing` / `tracing-subscriber` | Logs estruturados |
| `thiserror` | Erros de domínio |
| `sha2` | Verificação hash |
| `aes-gcm` ou `chacha20poly1305` | Criptografia opcional |
| `zip` | Backup ZIP |
| `regex` | Filtros e tokenizer |
| `dunce` | Normalização de paths Windows |
| `chrono` | Datas/agendamento |
| `uuid` | IDs |

---

## 3. Svelte — UI core

| Pacote | Versão alvo | Propósito |
|--------|-------------|-----------|
| `svelte` | 5.x | Runes (`$state`, `$derived`, `$effect`) |
| `@sveltejs/vite-plugin-svelte` | latest | Build com Vite |
| `typescript` | 5.x | `strict: true`, `noUncheckedIndexedAccess` |
| `svelte-check` | latest | Typecheck .svelte |

**Utilitários Svelte 5:**

| Pacote | Propósito |
|--------|-----------|
| `runed` | `PersistedState`, `Debounced`, `ElementSize` — forms e layout |
| `nanostores` | Stores globais mínimas (execuções, tema) |
| `@nanostores/persistent` | Persistir preferências UI localmente |

---

## 4. Componentes, interação e acessibilidade

Base: **bits-ui** (primitivos headless acessíveis, padrão WAI-ARIA).

| Pacote | Propósito |
|--------|-----------|
| `bits-ui` | Dialog, Dropdown, Select, Tabs, Tooltip, Checkbox, Switch, Progress |
| `vaul-svelte` | Drawer lateral (detalhes de log, editor rápido) |
| `cmdk-sv` | Command palette (`Ctrl+K` — buscar job, ação rápida) |
| `svelte-sonner` | Toasts acessíveis (anuncia status de execução) |
| `@floating-ui/dom` | Posicionamento de menus contextuais |
| `focus-trap` | Fallback se necessário fora do bits-ui |

**Formulários (Job editor, Blueprint editor, Settings):**

| Pacote | Propósito |
|--------|-----------|
| `felte` + `@felte/validator-zod` | Forms declarativos + validação |
| `zod` | Schemas espelhando contratos IPC |

**Tabelas (Histórico / Auditoria):**

| Pacote | Propósito |
|--------|-----------|
| `@tanstack/svelte-table` | Sort, filter, paginação virtualizada |
| `@tanstack/svelte-virtual` | Listas longas de audit log |

**Acessibilidade — práticas obrigatórias:**

- Foco visível em todos os controles (`:focus-visible`)
- `aria-live="polite"` na região de progresso de execução
- Labels em todos os inputs (Job editor)
- Contraste mínimo WCAG AA (tokens em UI-DESIGN-SYSTEM)
- Navegação por teclado na sidebar e command palette
- `eslint-plugin-svelte` regra `a11y-*` como error
- `@axe-core/playwright` em E2E críticos

---

## 5. Animação e motion

Filosofia: **motion invisível** — presente, nunca distrativo (skill minimalist-ui).

| Pacote / API | Uso |
|--------------|-----|
| `svelte/transition` | Fade/slide em modais, expand de cards |
| `svelte/motion` | Spring sutil em barras de progresso |
| `@formkit/auto-animate` | Reorder suave em listas de jobs |
| CSS `transform` + `opacity` only | Hover de cards, nunca `width/height` animados |

**Padrões AutoFlow:**

| Contexto | Animação |
|----------|----------|
| Card job expand | `slide` 200ms ease-out |
| Progress bar | width via CSS transition 150ms |
| Log streaming | auto-scroll; sem flash por linha |
| Page enter | fade 300ms, uma vez |
| Toast | slide from edge, 250ms |
| Stagger list | delay 40ms por item, max 5 itens |

**Respeitar:** `prefers-reduced-motion: reduce` — desliga stagger e springs.

---

## 6. Estilo visual

| Pacote | Propósito |
|--------|-----------|
| `tailwindcss` | Utility-first; tokens via `@theme` |
| `tailwind-variants` | Variantes de componente type-safe |
| `clsx` + `tailwind-merge` | Composição de classes |
| `@fontsource-variable/geist-sans` | UI sans (evitar Inter/Roboto genéricos) |
| `@fontsource-variable/geist-mono` | Paths, logs, audit |
| `phosphor-svelte` | Ícones Bold/Regular — alinhado ao design system |

**Tema claro/escuro:**

| Pacote | Propósito |
|--------|-----------|
| `mode-watcher` | Dark/light/system sync com `<html class="dark">` |

---

## 7. Dados, IPC e tipos

| Pacote / Ferramenta | Propósito |
|---------------------|-----------|
| `tauri-specta` | Gerar `src/lib/contracts/bindings.ts` |
| `zod` | Validar payloads de forms antes de invoke |
| Tipos manuais em `contracts/` | DTOs de domínio UI-only |

**Cliente IPC (wrapper):**

```typescript
// src/lib/core/ipc/client.ts
import { invoke } from '@tauri-apps/api/core';
import type { Job } from '$lib/contracts/job';
import { jobSchema } from '$lib/contracts/job.schema';

export async function listJobs(): Promise<Job[]> {
  const raw: unknown = await invoke('jobs_list');
  return z.array(jobSchema).parse(raw);
}
```

---

## 8. Internacionalização

| Pacote | Propósito |
|--------|-----------|
| `@inlang/paraglide-sveltekit` ou `@inlang/paraglide-js` | Mensagens type-safe |
| Idiomas v1 | `pt-BR`, `en-US` |
| Idiomas v2 | `es-ES`, `ja-JP`, `ru-RU` |

Regra: **status de execução** usa chaves i18n na UI; Rust emite códigos estáveis (`COPIED`, `FAILED`), não strings PT.

---

## 9. Dashboard e gráficos

| Pacote | Propósito |
|--------|-----------|
| `uplot` + `uplot-svelte` | Gráfico performance 60s — leve (~45KB) |
| Alternativa | `layerchart` se precisar mais de um chart |

Evitar Chart.js completo no desktop leve.

---

## 10. Testes e qualidade

| Ferramenta | Escopo |
|------------|--------|
| `vitest` | Unit TS + stores |
| `@testing-library/svelte` | Componentes |
| `playwright` + `@axe-core/playwright` | E2E + a11y |
| `cargo test` | Domain + application Rust |
| `cargo clippy` | Lint Rust |
| `eslint` + `prettier` | Lint/format TS/Svelte |
| `husky` + `lint-staged` | Pre-commit |

---

## 11. DevEx monorepo

| Ferramenta | Propósito |
|------------|-----------|
| `pnpm` workspaces | `apps/desktop`, `packages/*` |
| `turbo` | Cache de build/test (opcional) |
| `rust-toolchain.toml` | Pin Rust stable |

---

## 12. Matriz feature → biblioteca

| Feature AutoFlow | Bibliotecas principais |
|------------------|------------------------|
| CRUD Jobs | felte + zod + bits-ui Dialog |
| Progresso tempo real | Tauri events + nanostores + bits-ui Progress |
| Watch / schedule | Rust notify + tokio (sem lib UI) |
| Histórico + filtros | TanStack Table + Virtual |
| Rollback | Command + sonner confirm Dialog |
| Blueprint tokenizer | Rust regex (core) + felte preview |
| Browse seguro (v2) | Command + grid custom Svelte |
| Command palette | cmdk-sv |
| Tray / autostart | Tauri plugins |
| Tema + glass | mode-watcher + CSS variables |
| Simulação dry-run | Command + Dialog com lista virtualizada |

---

## 13. Dependências proibidas / evitar

| Evitar | Motivo |
|--------|--------|
| SvelteKit SSR | Desktop offline; Vite SPA basta |
| `@tauri-apps/plugin-fs` direto na UI | Quebra boundary; IO só no Rust |
| Material UI / Ant Design | Bundle pesado, visual genérico |
| Lucide como única icon set | Design system pede Phosphor |
| Polling 1s para estado | Usar events |
| `any` em TypeScript | Quebra contrato strict |

---

## 14. Atualização deste documento

Ao adicionar dependência:

1. Justificar na PR (1 linha)
2. Verificar licença compatível com CC BY-NC-ND do projeto
3. Atualizar `package.json` + esta tabela
