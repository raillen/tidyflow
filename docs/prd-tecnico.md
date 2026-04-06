# PRD Tecnico: AutoFlow

## 1. Metadados

- Produto: AutoFlow
- Repositorio: csharp_BatchBackupBridge
- Tipo: Desktop (Windows/Linux)
- Status do documento: Finalizado (MVP)
- Versao: 1.0.0
- Plataformas-alvo: Windows, Linux
- Data: 2026-03-20

---

## 2. Resumo Executivo

O AutoFlow é um aplicativo desktop projetado para automatizar a gestão de arquivos através de "Jobs". Ele permite que usuários configurem tarefas de cópia, movimentação e backup baseadas em eventos em tempo real (Watch Folder) ou agendamentos específicos.

### Decisao arquitetural do MVP

- linguagem principal: C# 12+
- stack principal: .NET 8, Avalonia UI, MVVM
- persistencia: JSON (local)
- distribuicao: Binários nativos para Windows/Linux
- limitacoes intencionais do MVP: Sem SQLite, sem Quartz.NET (scheduler interno simples), sem rollback avançado.

### Motivos da decisao

- Avalonia UI garante consistência visual e funcional entre Windows e Linux.
- .NET 8 é a versão estável e performática atual.
- JSON permite um desenvolvimento rápido do MVP sem as complexidades iniciais de banco de dados.

---

## 3. Problema

Gerenciar arquivos entre pastas manualmente é tedioso e propenso a erros. Ferramentas existentes muitas vezes são complexas demais ou carecem de uma interface intuitiva para monitoramento contínuo em tempo real em ambientes Windows e Linux.

### Dores principais

- Dificuldade em automatizar fluxos simples de arquivos sem scripts.
- Falta de feedback visual em tempo real sobre o que está sendo processado.
- Inconsistência de ferramentas de backup entre diferentes sistemas operacionais.

---

## 4. Objetivos do Produto

### Objetivos do MVP

- Prover uma UI clara para criação e gestão de Jobs.
- Implementar um Watch Folder confiável.
- Garantir a execução segura de operações de IO.

### Metas de sucesso

- Um Job criado e executado com sucesso via Watch Folder.
- Um Job agendado disparado corretamente.
- Logs claros de cada operação.

---

## 5. Nao Objetivos do MVP

- Sincronização em nuvem nativa.
- Interface mobile ou web.
- Gestão centralizada de múltiplos computadores.

---

## 6. Usuarios-alvo

### Perfil principal

- Desenvolvedores, produtores de conteúdo e usuários avançados que precisam organizar fluxos de trabalho de arquivos localmente.

### Jobs To Be Done

- "Quero monitorar minha pasta de Downloads e mover arquivos .pdf para uma pasta de documentos automaticamente."
- "Preciso fazer um backup diário da minha pasta de projetos para um HD externo."

---

## 7. Escopo do MVP

### Incluido

- CRUD de Jobs (Origem, Destino, Filtros, Conflitos).
- Watch Service (Native/Polling).
- Engine de Execução (Copy/Move).
- Logs básicos em TXT/CSV.
- Persistência em JSON.

### Fora do escopo inicial

- SQLite para histórico pesado.
- Quartz.NET para agendamentos complexos.
- Hash SHA256 para integridade.

---

## 8. Requisitos Funcionais

### RF-01. Criar Job de Automação

O usuário deve definir origem, destino, filtros e modo de operação (Cópia/Movimentação).

### RF-02. Monitoramento de Pastas (Watch Folder)

O sistema deve detectar mudanças na origem e disparar o Job automaticamente.

### RF-03. Agendamento de Tarefas

O usuário deve poder definir horários específicos ou intervalos para execução do Job.

---

## 9. Requisitos Nao Funcionais

### RNF-01. Performance

IO pesado deve rodar em background sem travar a UI.

### RNF-02. Confiabilidade

Operações falhas devem ser logadas sem interromper o sistema.

---

## 10. Decisoes Tecnicas

- stack: .NET 8, Avalonia UI.
- arquitetura: 4 camadas (Presentation, Application, Domain, Infrastructure).
- processo unico ou distribuido: Único processo (Desktop).
- modelo de dados: Entidades POCO serializadas em JSON.

---

## 11. Arquitetura de Alto Nivel

```text
Presentation (Avalonia/MVVM)
  -> Application (Use Cases/Services)
    -> Domain (Entities/Policies)
  -> Infrastructure (FileSystem/Persistence/Watch)
```

### Responsabilidades por camada

#### Presentation

Views e ViewModels que gerenciam a interação do usuário e o estado da tela.

#### Infrastructure

Acesso real ao disco, implementação do watcher e serialização JSON.

---

## 12. Modulos Principais

### 12.1. Job Manager

Gerencia o ciclo de vida dos Jobs (Criação, Edição, Deleção).

### 12.2. Execution Engine

Responsável pelas operações de cópia e movimentação física dos arquivos.

---

## 13. Modelo de Dominio

### Entidades

- `Job`, `JobSource`, `JobTarget`, `ExecutionRun`

### Enums

- `JobStatus`, `JobMode`, `ConflictMode`

---

## 14. Regras de Negocio Criticas

### RG-01. Prevenção de Loops

Um Job não pode ter a mesma origem e destino se o modo for Movimentação ou Cópia recursiva sem filtros.

---

## 15. Fluxos Criticos

### Fluxo 1. Execução Automática via Watcher

1. Watcher detecta arquivo.
2. Filtros são aplicados.
3. Job é enviado para a fila de execução.
4. Engine processa o arquivo.

---

## 16. Persistencia e Integracoes

### Persistencia

- tecnologia: JSON local.
- granularidade: Um arquivo por Job, arquivo central para configurações.

---

## 17. Estrategia de Testes

### Unitarios

- Validação de filtros e políticas de conflito.

### Integracao

- Simulação de operações de arquivo em pastas temporárias.

---

## 18. Backlog MVP Priorizado

### P0

| ID | Epico | Item | Criterio de aceite |
|---|---|---|---|
| MVP-01 | Fundação | Setup do projeto Avalonia | App abre com janela vazia |
| MVP-02 | Jobs | CRUD de Jobs básico | Salva e lê JSON de Jobs |

---

## 19. Roadmap de Sprints

### Sprint 0. Fundacao

Objetivo: Estruturar o projeto e a UI base.

Entrega: Shell da UI em Avalonia, suporte a temas e i18n inicial.

---

## 20. Criterios de Aceite do MVP

- App funciona em Windows e Linux.
- Jobs são persistidos corretamente.
- Arquivos são movidos/copiados via Watcher.
