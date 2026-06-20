# Design System UI — AutoFlow v2

**Status:** `Planejado`

Diretrizes visuais e de interação para Svelte + Tailwind. Baseado no skill **minimalist-ui** e na identidade AutoFlow (clean, respirado, utilitário premium).

---

## 1. Personalidade visual

- **Micro file-manager + control tower** — não imitar Explorer pesado
- Muito whitespace, baixa densidade
- Superfícies brancas/off-white sobre canvas cinza frio
- Cor como recurso escasso — pastéis só para semântica
- Motion quase invisível

---

## 2. Tipografia

| Papel | Fonte | Uso |
|-------|-------|-----|
| UI / corpo | Geist Sans Variable | Labels, botões, listas |
| Editorial / títulos de página | Geist Sans Bold/Black | H1 dashboard, nomes de módulo |
| Mono | Geist Mono Variable | Paths, logs, audit, timestamps |

**Evitar:** Inter, Roboto, Open Sans como primárias.

**Escala:**

| Token | Size | Weight |
|-------|------|--------|
| `--text-xs` | 11px | 500 |
| `--text-sm` | 13px | 400 |
| `--text-base` | 14px | 400 |
| `--text-lg` | 18px | 600 |
| `--text-xl` | 24px | 700 |
| `--text-2xl` | 32px | 800 |

Line-height corpo: `1.6`. Títulos: `1.15`, tracking `-0.02em`.

---

## 3. Cores (CSS variables)

### Light (default)

```css
:root {
  --canvas: #f3f3f5;
  --surface: #ffffff;
  --surface-muted: #f7f6f3;
  --border: #eaeaea;
  --text-primary: #1d2129;
  --text-secondary: #4e5969;
  --text-muted: #787774;
  --accent: #0064ff;
  --success: #00b42a;
  --warning: #ff7d00;
  --danger: #f53f3f;
  --folder-accent: #fbbf24; /* ícones de pasta apenas */
}
```

### Dark

```css
.dark {
  --canvas: #141414;
  --surface: #1f1f1f;
  --surface-muted: #262626;
  --border: rgba(255,255,255,0.08);
  --text-primary: #f2f3f5;
  --text-secondary: #a9aeb8;
  --accent: #3c7eff;
}
```

Pastéis semânticos (badges): ver skill minimalist-ui §4 — `#EDF3EC` success bg, `#FDEBEC` error bg, etc.

---

## 4. Layout

### Shell

```
┌─────────────────────────────────────────────┐
│ Titlebar drag (45px)                        │
├──────────┬──────────────────────────────────┤
│ Sidebar  │ Content area                     │
│ 240px    │ padding 24–32px                  │
│          │ max-width none (fluid)           │
└──────────┴──────────────────────────────────┘
```

- Sidebar: navegação primária, ícone Phosphor + label
- Conteúdo: `bg-canvas` implícito; cards `bg-surface` radius `12px`, border `1px solid var(--border)`
- **Sem** sombras pesadas; hover card: `box-shadow: 0 2px 8px rgba(0,0,0,0.04)`

### Grid dashboard (bento)

- Cards assimétricos CSS Grid
- Padding interno 24–32px
- Radius máximo 12px — nunca pill em containers grandes

---

## 5. Componentes (mapeamento bits-ui)

| Componente AutoFlow | Base | Notas a11y |
|---------------------|------|------------|
| `Button` | bits-ui Button | focus-visible ring 2px accent |
| `Dialog` | bits-ui Dialog | trap focus, Esc fecha |
| `Sheet` / Drawer | vaul-svelte | detalhe de audit |
| `Tabs` | bits-ui Tabs | setas opcionais; roving tabindex |
| `Select` | bits-ui Select | label associado |
| `Switch` | bits-ui Switch | aria-checked |
| `Progress` | bits-ui Progress | aria-valuenow live |
| `Tooltip` | bits-ui Tooltip | delay 400ms |
| `DropdownMenu` | bits-ui DropdownMenu | menu contextual arquivo |
| `Toast` | svelte-sonner | role status/alert |
| `CommandPalette` | cmdk-sv | Ctrl+K global |

Wrappers em `src/lib/components/ui/*` — **nunca** importar bits-ui direto nas features.

---

## 6. Padrões por tela

### Dashboard

- 3 stat cards + gráfico uPlot 60s
- Lista execuções ativas com `aria-live="polite"`
- Atividades recentes monospace compact

### Fluxos (Watchers | Transfers)

- Job card colapsável (chevron + `slide` transition)
- Ações: Simular, Editar, Executar — icon buttons com `aria-label`
- Badge status pill uppercase tracking wide

### Blueprints

- Split File | Folder tabs
- Editor template com preview inline (`blueprints_preview_template`)

### Histórico

- TanStack Table + virtual scroll
- Filtros sticky top
- Painel detalhe drawer direita

### Configurações

- Seções accordion leve (border-bottom only)
- `<kbd>` para atalhos documentados

### File Browser (Fase 5)

- Topbar busca + filtros
- Grid wrap pastas amarelas + cards arquivo selecionado
- Menu contextual DropdownMenu — Copiar caminho, Export JSON path

Referência visual detalhada: [`file_browser_module.md`](../../file_browser_module.md) na raiz do repo.

---

## 7. Motion (implementação)

```css
@media (prefers-reduced-motion: no-preference) {
  .card-enter {
    animation: fade-up 300ms cubic-bezier(0.16, 1, 0.3, 1);
  }
}
@keyframes fade-up {
  from { opacity: 0; transform: translateY(12px); }
  to   { opacity: 1; transform: translateY(0); }
}
```

| Interação | Duração | Easing |
|-----------|---------|--------|
| Hover button | 150ms | ease |
| Expand card | 200ms | ease-out |
| Dialog open | 250ms | cubic-bezier(0.16,1,0.3,1) |
| Progress width | 150ms | linear |
| List stagger | 40ms × n | max n=5 |

 `@formkit/auto-animate` em listas de jobs reorder.

---

## 8. Acessibilidade — checklist por PR de UI

- [ ] Contraste AA verificado (WebAIM ou axe)
- [ ] Todo input tem `<label>` visível ou aria-label
- [ ] Ordem de tab lógica
- [ ] Focus trap em modais
- [ ] Região live para progresso
- [ ] Ícones decorativos `aria-hidden="true"`
- [ ] Botões ícone com aria-label traduzido (Paraglide)
- [ ] `prefers-reduced-motion` respeitado
- [ ] Idioma em `<html lang="pt-BR">` dinâmico

---

## 9. Ícones

**Phosphor Svelte** — peso `bold` na sidebar, `regular` inline.

| Contexto | Ícone |
|----------|-------|
| Dashboard | `SquaresFour` |
| Fluxos | `ArrowsLeftRight` |
| Blueprints | `TreeStructure` |
| Histórico | `ClockCounterClockwise` |
| Settings | `Gear` |
| Pasta | `Folder` (fill amarelo via CSS) |
| Executar | `Play` |
| Simular | `Binoculars` |

---

## 10. Tema Windows (opcional Fase 4)

- `window-vibrancy` crate Rust para Mica/Acrylic no Tauri
- Opacidade sidebar ligada a `settings.glass_opacity`
- Fallback sólido em Linux/macOS

---

## 11. Anti-patterns (proibidos)

- Gradientes hero, neon, glassmorphism exagerado
- `rounded-full` em cards grandes
- Emojis na UI
- Copy marketing: "Seamless", "Unleash", etc.
- Placeholder stats na dashboard
- Cores saturadas em blocos grandes
