# ADR-001: Escolha da Stack Tecnica e Arquitetura

- Status: `Accepted`
- Data: 2026-03-20
- Escopo: Projeto FolderFlow
- Relacionado ao PRD: PRD-FolderFlow v1.0.0

---

## 1. Contexto

O projeto FolderFlow requer uma base tecnológica que suporte desenvolvimento cross-platform (Windows e Linux) com uma interface desktop rica, performance em operações de IO e facilidade de manutenção. É necessário decidir qual framework de UI e qual arquitetura de software serão utilizados para garantir longevidade e escalabilidade.

---

## 2. Decisao

Decidimos utilizar **.NET 8** com **Avalonia UI** e uma **Arquitetura em Camadas (4 camadas)** seguindo princípios de Clean Architecture.

### 2.1. Convencao ou escolha principal

- **Framework de UI:** Avalonia UI (Cross-platform nativo para .NET).
- **Arquitetura:** 4 camadas (Presentation, Application, Domain, Infrastructure).
- **Persistência MVP:** Arquivos JSON locais.

### 2.2. Regras operacionais

- Regras de negócio devem residir exclusivamente nas camadas de Domain e Application.
- A UI não deve realizar operações de IO diretamente; deve usar serviços da Infrastructure através de interfaces da Application.
- Todos os Jobs e configurações devem ser serializados em JSON no MVP.

### 2.3. Implementacao obrigatoria

- Uso de MVVM para separação de preocupações na interface.
- Implementação de um `IWatchService` para abstrair a detecção de mudanças em pastas.
- Uso de `Task` e `CancellationToken` para todas as operações assíncronas de IO.

---

## 3. Consequencias

### Positivas

- Mesma base de código para Windows e Linux com aparência consistente.
- Alta performance e suporte a recursos modernos do C# 12+.
- Facilidade de teste devido ao desacoplamento entre camadas.

### Negativas

- Curva de aprendizado do Avalonia UI para quem vem apenas de WPF/WinForms (embora similar).
- Dependência do ecossistema .NET no Linux.

---

## 4. Alternativas consideradas

### Alternativa A. Electron / React

Rejeitada porque:
- Alto consumo de memória.
- Dificuldade de integração profunda com IO de sistema de arquivos comparado ao .NET nativo.

### Alternativa B. MAUI (Multi-platform App UI)

Rejeitada porque:
- O suporte para Linux ainda não é oficial ou robusto o suficiente para uma ferramenta de automação de sistema de arquivos comparado ao Avalonia.

---

## 5. Impacto no Sprint 0

- Criação da solução .NET 8 com os 4 projetos (App, Application, Domain, Infrastructure).
- Setup do Avalonia UI no projeto `FolderFlow.App`.
- Implementação inicial da estrutura de pastas recomendada.

---

## 6. Regra de governanca

Este ADR deve ser revisado se houver necessidade de migrar para um banco de dados relacional (SQLite) ou se novas plataformas (como macOS) exigirem mudanças estruturais profundas.
