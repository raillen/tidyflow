# Testes E2E (Playwright)

**Status:** `Implementado` — 69 testes, execução headless contra o frontend real.

Os testes de ponta a ponta exercitam a UI SvelteKit completa, sem o processo Rust.
Isso permite validar navegação, formulários, modais e estados de erro em segundos,
em qualquer máquina e em CI, sem precisar compilar o binário Tauri.

---

## 1. Por que mockar o IPC

A UI conversa com o backend exclusivamente por `@tauri-apps/api`:

| Canal | Uso |
|-------|-----|
| `invoke(cmd, args)` | Todos os comandos (`jobs_*`, `blueprints_*`, `settings_*`, `admin_*`) |
| `listen(event, handler)` | `execution_progress`, `execution_completed`, `security_lock` |

O pacote `@tauri-apps/api` resolve tudo para `window.__TAURI_INTERNALS__`. O mock
substitui esse objeto por um backend em memória, portanto **nenhum código do app
é alterado** para testar.

```
e2e/
├── fixtures/
│   ├── tauri-mock.js    # backend em memória injetado antes do bundle
│   └── test.js          # fixture base do Playwright que instala o mock
├── navigation.spec.js
├── flows.spec.js
├── job-editor.spec.js
├── blueprints.spec.js
├── history.spec.js
├── settings.spec.js
├── admin.spec.js
├── security.spec.js
└── i18n.spec.js
```

---

## 2. Como o mock é instalado

`fixtures/test.js` estende o `test` do Playwright e chama `page.addInitScript`
com o conteúdo de `tauri-mock.js`. O script roda **antes** de qualquer módulo do
app:

```js
export const test = base.extend({
  page: async ({ page }, use) => {
    await page.addInitScript({ content: mockSource });
    await use(page);
  },
});
```

Consequência importante: qualquer `addInitScript` do próprio teste roda **depois**
do mock. Por isso dados persistidos (`localStorage`) são lidos de forma preguiçosa
— veja `currentSettings()` em `tauri-mock.js`.

---

## 3. O que o mock cobre

### Dados de exemplo

| Entidade | Registros |
|----------|-----------|
| Jobs | `Backup de Documentos` (ativo), `Fluxo desativado` (inativo) |
| Blueprints | `Organizar notas fiscais` (arquivo), `Estrutura de projeto` (pasta, com variável `projectName`) |
| Auditoria | 1 `COPIED`, 1 `FAILED`, 1 `ORGANIZED` |
| Admin | 1 instância online com 2 jobs e capacidades `planned` |

### Comportamentos fiéis ao backend

- `security_verify_pin` só aceita `1234`; qualquer outro valor retorna `false`.
- `jobs_get` de um id inexistente **lança** `{ code, message }` — reproduz a
  rejeição do Tauri para que o caminho de erro seja exercitado.
- `audit_query` aplica `status` e `search` de verdade, permitindo testar filtros.
- `settings_update` persiste em `localStorage` e sobrevive ao reload.
- Eventos (`execution_progress`, `security_lock`) são emitidos via
  `window.__TIDYFLOW_MOCK__.emit(...)`.

### Escape hatch para os testes

```js
window.__TIDYFLOW_MOCK__.emit("security_lock", {});
window.__TIDYFLOW_MOCK__.pushExecution({ executionId, jobId, ... });
window.__TIDYFLOW_MOCK__.ids; // UUIDs fixos dos fixtures
```

---

## 4. Cobertura por arquivo

| Spec | Cobre |
|------|-------|
| `navigation` | Dashboard, sidebar, `aria-current`, deep link `?edit=` |
| `flows` | Lista, criar/editar, simular, rollback, excluir, fila, execução ao vivo |
| `job-editor` | Os 6 painéis extraídos: geral, filtros, automação, monitoramento, segurança, avançado |
| `blueprints` | Tabs por tipo, CRUD, variáveis obrigatórias, simular |
| `history` | Métricas, filtros, export CSV/JSON, limpar, paginação |
| `settings` | 9 abas, tema, accent, validação de PIN, webhooks, changelog, idioma |
| `admin` | Snapshot, detalhes da instância, comandos, erro de IPC |
| `security` | Lock screen, PIN correto/incorreto, Enter, botão desabilitado |
| `i18n` | pt-BR padrão, en-US persistido, troca de idioma, nenhuma chave crua |

---

## 5. Executar

```bash
# Sobe o Vite e roda tudo (headless)
pnpm test:e2e

# Um arquivo
cd apps/desktop && npx playwright test flows.spec.js

# Um teste
npx playwright test flows.spec.js -g "rollback"

# Com UI (debug)
npx playwright test --headed --workers=1

# Relatório HTML gerado em playwright-report/
npx playwright show-report
```

### Pré-requisitos

- Navegador Chromium/Chrome disponível. O `playwright.config.ts` aponta para
  `/home/raillen/.local/bin/google-chrome`; ajuste `executablePath` ou rode
  `npx playwright install chrome` na sua máquina.
- O dev server **não** é iniciado pelo Playwright (não há `webServer` na config).
  Suba com `pnpm dev` ou deixe o `test:e2e` cuidar disso.

---

## 6. Convenções ao escrever novos testes

1. **Sempre importe de `./fixtures/test.js`**, nunca de `@playwright/test`
   diretamente — senão o mock não é instalado.
2. **Selecione por papel/`aria-label`**, não por classe CSS. Classes do Svelte são
   geradas (`s-XXXX`) e mudam a cada build.
3. **Aguarde o app montar** antes de emitir eventos: `await expect(navigation).toBeVisible()`.
   O listener de `security_lock` só existe depois do mount.
4. **Teste o comportamento, não a implementação.** Prefira "o card desaparece" a
   "o método X foi chamado".
5. **Adicione o caso de erro** quando o handler trata falha de IPC. O mock lança
   para isso.

---

## 7. Limitações conhecidas

- **Não exercita o Rust.** Regras de negócio do domínio são cobertas por
  `cargo test`. Estes testes garantem que a UI chama o contrato certo e reage
  corretamente às respostas.
- **Diálogos nativos** (`confirm`, `alert`) são interceptados via `page.on("dialog")`.
  O código de produção usa `confirm()` em ações destrutivas; a cobertura depende
  disso.
- **Diálogo de seleção de pasta** (`@tauri-apps/plugin-dialog`) retorna um caminho
  fixo do mock (`/home/demo/escolhido`).
- **Persistência é por página**, não por teste: cada teste começa com o estado
  inicial dos fixtures, exceto o que for gravado em `localStorage`.
