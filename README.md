# 🔄 TidyFlow v2

**TidyFlow** é uma solução desktop moderna, leve e de alta performance para automação de gestão de arquivos e pastas. Desenvolvido com **Tauri 2, Rust e Svelte 5**, ele permite criar pipelines inteligentes de arquivos (Jobs), organizar diretórios in-place através de Blueprints e monitorar pastas em tempo real com baixíssimo uso de recursos do sistema.

---

## 🚀 Principais Recursos

- **Monitoramento em Tempo Real (Watch Folder):** Detecta instantaneamente modificações, criações e exclusões no sistema de arquivos através do motor reativo do crate `notify` em Rust com debounce integrado.
- **Blueprints de Organização:** Crie regras dinâmicas de estruturação e renomeação de pastas utilizando tokens dinâmicos (Data, Contador, Nome Original, Extensão, UUID, etc.) alimentados por um pipeline extensível de transformações (Tokenizer).
- **Filtros Avançados:** Filtre arquivos por extensão, glob, regex, tamanho, data de criação/modificação ou metadados (como EXIF de imagens).
- **Simulação (Dry-Run):** Pré-visualize de forma clara as ações exatas que o TidyFlow executará antes de aplicar qualquer alteração real em seus arquivos.
- **Fila de Execução Controlada:** Gerenciamento centralizado de tarefas em background com controle de concorrência, prioridade e limites de velocidade.
- **Auditoria & Rollback:** Histórico completo de auditoria persistido em SQLite com a capacidade de desfazer movimentações de arquivos indesejadas.
- **Segurança Operacional:** Validação rigorosa de loops de diretórios recursivos e caminhos restritos através de políticas de permissão de diretórios (`PathPolicy`).

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
Desenvolvido por [Raillen Santos](https://github.com/raillen)
