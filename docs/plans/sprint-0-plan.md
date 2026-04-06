# Plano Tecnico: Sprint 0 (Fundação)

- Sprint: `0`
- Duracao sugerida: 2 semanas
- Objetivo macro: Setup da infraestrutura, arquitetura inicial e UI Shell básica em Avalonia.

---

## 1. Resultado esperado

Ao final do Sprint 0, o projeto deve:

- Ter a solução .NET 8 com a arquitetura de 4 camadas estruturada.
- Ter o app Avalonia abrindo com um Shell básico (Sidebar/MainView).
- Suportar troca de temas (Light/Dark) e múltiplos idiomas (i18n).
- Carregar configurações iniciais de um arquivo JSON.

O Sprint 0 nao entrega ainda:

- Execução real de Jobs de arquivo.
- Watch Folder funcional.
- Wizard de criação de Jobs completo.

---

## 2. Escopo fechado

### Incluido

- Setup da solução .NET 8.
- Configuração do Avalonia UI + MVVM.
- Estrutura de pastas e namespaces.
- Sistema de Settings (Persistência JSON).
- Mock do Dashboard inicial.

### Excluido

- Operações de FileSystem reais.
- Agendamento de tarefas.
- Notificações de sistema ativas.

---

## 3. Estrutura minima de codigo

```text
src/
  AutoFlow.App/             # Presentation
  AutoFlow.Application/     # Services/UseCases
  AutoFlow.Domain/          # Entities/Logic
  AutoFlow.Infrastructure/  # Implementation
tests/
  AutoFlow.Tests.Unit/
docs/
  adr/
  specs/
  plans/
```

---

## 4. Backlog tecnico do Sprint 0

| ID | Modulo | Arquivos alvo | Entrega | Pronto quando |
|---|---|---|---|---|
| S0-01 | Solução | `*.sln`, `*.csproj` | Setup de projetos e referências | Build passa sem erros |
| S0-02 | UI Shell | `MainWindow.axaml` | Estrutura de navegação (Sidebar) | Menu lateral clicável |
| S0-03 | Settings | `SettingsJsonStore.cs` | Leitura/Escrita de JSON | Settings persistem no disco |
| S0-04 | Temas | `ThemeService.cs` | Sistema de troca de cores | App reflete mudança claro/escuro |

---

## 5. Sequencia de execucao recomendada

### Bloco A. Fundacao

- **S0-01:** Criar solução e projetos com as dependências corretas (Avalonia, MVVM).
- **S0-02:** Implementar o Shell principal e navegação básica entre páginas (Dashboard, Jobs, Settings).

### Bloco B. Contratos e modelos

- **S0-03:** Definir entidades `Job` e `AppSettings` na camada Domain.
- **S0-04:** Implementar persistência JSON na camada Infrastructure.

---

## 6. Dependencias internas

- Conclusão do setup do Avalonia para iniciar o desenvolvimento de Views.
- Definição do contrato de Settings para implementar o `ThemeService`.

---

## 7. Criticos de implementacao

### Avalonia UI Cross-platform

- Garantir que fontes e ícones carreguem corretamente tanto em Windows quanto em Linux.

### Persistência JSON

- Lidar com exceções de IO ao tentar ler/escrever configurações se o arquivo estiver bloqueado.

---

## 8. Criterios de saida

O sprint so termina quando:

- O código compila em .NET 8.
- O app abre e permite navegar para a página de Settings.
- A troca de idioma/tema funciona visualmente.
- Arquivo `settings.json` é gerado automaticamente se não existir.

---

## 9. Riscos do Sprint 0

### Risco 1. Incompatibilidade de UI no Linux

Mitigacao:
- Testar periodicamente o build em ambiente Linux (ou via WSLg).

---

## 10. Proxima passagem

Se os criterios de saida forem atingidos, o proximo passo operacional e:

- **Epic: Gestão de Jobs** (Sprint 1)
- CRUD de Jobs e validação de caminhos.
