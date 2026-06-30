# 🔄 TidyFlow v2

**TidyFlow** é uma solução desktop moderna, leve e de alta performance para automação de gestão de arquivos e pastas. Desenvolvido com **Tauri 2, Rust e Svelte 5**, ele permite criar pipelines inteligentes de arquivos (Jobs), organizar diretórios in-place através de Blueprints e monitorar pastas em tempo real com baixíssimo uso de recursos do sistema.

---

## 🚀 Principais Recursos

Abaixo estão os recursos centrais do TidyFlow v2, unindo a segurança e performance do Rust com a flexibilidade da interface moderna em Svelte 5:

| Recurso | Descrição | Tecnologia Base |
| :--- | :--- | :--- |
| **Watch Folders** | Monitoramento reativo em tempo real de alterações de disco com debounce automático para evitar execuções parciais. | Crate `notify` em Rust + canais assíncronos. |
| **Blueprints & Tokenizer** | Regras flexíveis para renomear e estruturar diretórios in-place usando metadados e contadores dinâmicos (data, stem, UUID). | Engine customizada Rust (`autoflow-domain/src/tokenizer`). |
| **Filtros Avançados** | Regras rígidas de correspondência por extensão, glob patterns, expressões regulares, tamanho de arquivo, datas ou EXIF. | Engine de filtros e busca em Rust. |
| **Agendador Integrado** | Execução programada de tarefas periódicas ou em horários específicos. | Engine core de agendamento em Rust. |
| **Segurança de Caminhos** | Validação automática de diretórios autorizados, detecção de loops de recursão e bloqueio de caminhos restritos. | Módulo `PathPolicy` no core Rust. |
| **Simulação (Dry-Run)** | Exibe o plano exato das ações, colisões de nomes e alertas operacionais antes de qualquer gravação física no disco. | Rust dry-run engine + Tauri IPC. |
| **Fila de Execução** | Controle refinado de concorrência, cancelamento com tokens assíncronos e prioridades de execução de jobs. | Tokio channels e workers assíncronos. |
| **Histórico & Auditoria** | Logs operacionais completos com persistência local e interface interativa para rollback de movimentações de arquivos. | SQLite (via sqlx) + UI de auditoria Svelte. |

---

## 🛠️ Stack Técnica (TidyFlow v2)

*   **Core (Backend):** Rust (tokio, sqlx, notify-debouncer-full, serde, tracing, regex, chrono)
*   **Interface (Frontend):** Svelte 5 (Runes), TypeScript (modo estrito), Vite 6, Tailwind CSS 4
*   **Desktop Shell:** Tauri 2 (IPC tipado com Specta, plugins nativos de dialog, notification e window state)
*   **Persistência:** SQLite (via sqlx) para auditoria e histórico; JSON local para preferências e configurações rápidas.
*   **Monorepo:** `pnpm` workspaces (web/desktop) e `Cargo` workspaces (crates Rust).

---

## 📦 Como Executar o Desenvolvimento

### Pré-requisitos
Certifique-se de ter instalado:
*   [Node.js](https://nodejs.org/) (v18+)
*   [pnpm](https://pnpm.io/) (v9+)
*   [Rust](https://www.rust-lang.org/) (v1.78+)

### Instalação & Execução
```bash
# 1. Clone o repositório
git clone https://github.com/raillen/tidyflow.git
cd tidyflow

# 2. Instale as dependências do monorepo
pnpm install

# 3. Execute o aplicativo em modo de desenvolvimento (Tauri + Svelte)
pnpm dev
```

---

## 🧪 Executando Testes

O projeto contém testes unitários e de integração abrangentes tanto para o frontend (Vitest) quanto para o backend (Cargo).

```bash
# Executar testes do core em Rust
pnpm test:rust        # (atalho para `cargo test --workspace`)

# Executar testes da interface Svelte/TS (Vitest)
pnpm test

# Verificar integridade e tipagem do Svelte
pnpm --filter @tidyflow/desktop check
```

---

## 📁 Estrutura do Projeto

```text
tidyflow/
├── apps/
│   └── desktop/                 # Shell Tauri 2 + Interface SvelteKit
├── crates/                      # Módulos principais do core em Rust
│   ├── autoflow-domain          # Entidades e regras de domínio puro
│   ├── autoflow-application     # Casos de uso e orquestração de serviços
│   ├── autoflow-infrastructure  # SQLite, watchers, IO e persistência
│   ├── autoflow-core            # Fila de tarefas, agendamento e CLI
│   └── autoflow-admin-server    # Servidor de monitoramento e agente remoto
├── docs/                        # Documentação técnica e especificações (v2)
├── package.json                 # Monorepo configs e scripts rápidos
├── Cargo.toml                   # Workspace Cargo
└── pnpm-workspace.yaml          # Monorepo pnpm workspaces
```

---

## ☕ Apoie o Projeto (Donate)

Se o TidyFlow te ajudou a otimizar sua rotina e economizar tempo, você pode apoiar o desenvolvimento do projeto através do **Buy Me a Coffee**:

*   **Buy Me a Coffee:** [buymeacoffee.com/raillen](https://www.buymeacoffee.com/raillen)

---
Desenvolvido por [Raillen Santos](https://github.com/raillen)
