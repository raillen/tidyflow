# Frontend — Estrutura e Convenções

**Status:** `Implementado` — descreve `apps/desktop/src` após a refatoração dos editores.

Este documento é o mapa do frontend: onde cada coisa vive, por que as pastas
existem e quais convenções valem para código novo.

---

## 1. Árvore

```
apps/desktop/src/
├── app.css                 # tokens de design (cores, espaçamento, tipografia)
├── routes/                 # SvelteKit — uma pasta por rota
│   ├── +layout.svelte      # shell, PIN lock, listeners de evento
│   ├── +page.svelte        # dashboard
│   ├── flows/              # lista + editor
│   ├── blueprints/
│   ├── history/
│   ├── settings/
│   └── admin/
└── lib/
    ├── components/
    │   ├── jobs/
    │   │   ├── JobEditor.svelte      # shell de navegação (leve)
    │   │   ├── JobFormModal.svelte   # modal create/edit
    │   │   ├── SimulationModal.svelte
    │   │   └── panels/               # um arquivo por aba
    │   ├── blueprints/
    │   ├── layout/
    │   ├── settings/                 # ToggleField, NumberField, SectionTitle, CollapsibleSection
    │   ├── security/
    │   └── ui/                       # Modal
    ├── contracts/            # schemas Zod + tipos + builders de opções
    ├── core/
    │   ├── errors.ts         # errorMessage()
    │   ├── ipc/              # client.ts (invoke tipado), ui-state.ts
    │   └── stores/           # nanostores (tema, execuções)
    ├── i18n/                 # dicionários + t()
    └── styles/editor.css     # chrome compartilhado dos editores
```

---

## 2. Camadas

| Camada | Pasta | Regra |
|--------|-------|-------|
| Rota | `routes/` | Só orquestra: carrega dados, chama ações, monta componentes. Nada de lógica de domínio. |
| Componente | `lib/components/` | Recebe props, emite callbacks. Não chama IPC diretamente (exceção: modais que carregam o próprio registro). |
| Contrato | `lib/contracts/` | Zod é a fonte de verdade do tipo. `z.infer` gera o TypeScript. |
| IPC | `lib/core/ipc/` | Único lugar que conhece nomes de comando. Todo retorno passa por `.parse()`. |
| Estado | `lib/core/stores/` | Stores globais mínimos: tema e execuções ativas. |

### O contrato é o tipo

```ts
// lib/contracts/job.ts
export const jobSchema = z.object({ /* ... */ });
export type Job = z.infer<typeof jobSchema>;
```

Nunca declare uma interface à mão para algo que vem do backend: o schema já
valida em runtime e inferiu o tipo. Divergência vira erro no `pnpm check`.

---

## 3. Editores com painéis

Os editores grandes (`JobEditor`, `BlueprintEditor`, `FolderStructureEditor`)
foram divididos em **shell + painéis**.

```
JobEditor.svelte          → navegação lateral, estado de aba ativa
panels/JobGeneralPanel.svelte
panels/JobFiltersPanel.svelte
panels/JobAutomationPanel.svelte
panels/JobMonitoringPanel.svelte
panels/JobSecurityPanel.svelte
panels/JobAdvancedPanel.svelte
```

Regras:

1. **Shell não renderiza campos.** Ele só decide qual painel aparece.
2. **Painel recebe `job` com `$bindable()`** e devolve mudanças por atribuição
   imutável: `job = { ...job, campo }`.
3. **Painel não importa `client.ts`.** Se precisa de IPC, recebe callback.
4. **Um painel = uma decisão do usuário.** Se um painel crescer além de ~250
   linhas, é sinal de que ele mistura duas responsabilidades.

### Por que o CSS dos painéis é global

Um bloco `<style>` de componente Svelte é **scoped**: gera classes `s-XXXX` e não
alcança markup de componentes filhos. Como os painéis são filhos do shell, as
classes compartilhadas (`.collapsible`, `.field-row`, `.preset-group`, …) vivem
em `lib/styles/editor.css`, importado no layout raiz.

O que é exclusivo de um componente (a navegação lateral) continua no `<style>`
local do shell.

---

## 4. Tratamento de erro

**Sempre** use `errorMessage(error, fallback)`, nunca `error instanceof Error`.

Motivo: `invoke` do Tauri rejeita com **string** para `Result::Err(String)`. Nesse
caso `instanceof Error` é `false` e a mensagem real do backend seria descartada,
mostrando só o texto genérico.

```ts
// lib/core/errors.ts
export function errorMessage(error: unknown, fallback: string): string;

// uso
catch (e) {
  error = errorMessage(e, t("flows.runError"));
}
```

### Handlers de comando

Ações que chamam IPC em resposta a clique devem tratar falha e mostrar feedback.
No admin existe `runCommand(action)`, que centraliza isso:

```ts
async function runCommand(action: () => Promise<void>) {
  try {
    await action();
  } catch (error) {
    message = errorMessage(error, t("admin.commandError"));
    messageIsError = true;
  }
}
```

Sem esse wrapper, uma falha de IPC vira `unhandled rejection` silencioso.

---

## 5. i18n

- Toda string visível vem de `t("chave")`. A única exceção são dados de marca
  (nome do produto, redes sociais do autor).
- Listas de opções são **funções**, não constantes: `weekdays()`,
  `ruleKindOptions()`, `themeOptions()`. Só assim o rótulo reage à troca de idioma.
- `pt-BR.json` e `en-US.json` devem ter **exatamente** as mesmas chaves. Verifique:

```bash
node -e 'const f=(o,p="")=>Object.entries(o).flatMap(([k,v])=>typeof v==="object"?f(v,p+k+"."):[p+k]);
const a=require("./apps/desktop/src/lib/i18n/pt-BR.json"),b=require("./apps/desktop/src/lib/i18n/en-US.json");
const x=f(a),y=f(b);console.log(x.length,y.length,x.filter(k=>!y.includes(k)),y.filter(k=>!x.includes(k)));'
```

- Chaves dinâmicas usam template literal (`blueprints.template.token.${name}.label`).
  Ao procurar órfãs, exclua esses prefixos.

---

## 6. Acessibilidade

- Use `<button type="button">` dentro de forms. O default é `submit` e provoca
  navegação não intencional.
- Todo `<input>` declara `type` (`text`, `password`, `number`, `datetime-local`).
- Campos de segredo usam `type="password"` **e** `autocomplete` apropriado
  (`new-password` para criar, `off` para tokens de API).
- Estado selecionado: `aria-pressed` em botões de alternância, `aria-current="page"`
  na navegação, `aria-modal` + `role="dialog"` em overlays.
- Mensagens de erro: `role="alert"`. Mensagens de sucesso: `role="status"` +
  `aria-live="polite"`.
- Respeite `prefers-reduced-motion` em transições.

---

## 7. Testes

| Tipo | Ferramenta | Onde |
|------|-----------|------|
| Contrato de schema | Vitest | `lib/contracts/*.test.ts` |
| Comportamento de UI | Playwright | `e2e/*.spec.js` |

Para E2E veja [`TESTING-E2E.md`](./TESTING-E2E.md). Regra de ouro: **importe de
`./fixtures/test.js`**, nunca de `@playwright/test` direto.

---

## 8. Checklist para código novo

- [ ] Schema Zod em `contracts/`, tipo via `z.infer`
- [ ] Nenhum nome de comando fora de `core/ipc/client.ts`
- [ ] `errorMessage()` em todo `catch`
- [ ] `t()` em toda string visível; paridade pt-BR/en-US
- [ ] `<button type="button">` e `<input type="…">`
- [ ] Componente novo abaixo de ~250 linhas; senão, divida
- [ ] Teste E2E para o fluxo principal e para o caso de erro
- [ ] `pnpm check` e `pnpm test:e2e` verdes
