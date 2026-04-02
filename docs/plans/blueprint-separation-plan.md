# Plano de Refatoração: Separação de Transferência e Organização (Blueprint)

## 1. Visão Geral
Este plano descreve a separação das funcionalidades de transporte de arquivos (Cópia/Movimentação) das funcionalidades de organização in-place (Scaffolding/Renomeação), criando um módulo dedicado chamado **Blueprint**.

## 2. Mudanças de Naming e UX
- **Módulo Automação** -> Renomeado para **Transferências** (Flows).
- **Novo Módulo** -> **Blueprint** (Estrutura e Organização).
- **Navbar**: Inclusão do ícone de Blueprint e atualização dos rótulos.

## 3. Implementação Técnica

### Fase A: Limpeza e Ajuste do Domínio
- Remover campos de organização da entidade `Job`.
- Criar a entidade `Blueprint` no domínio:
    - `Id`, `Name`, `Path`, `BlueprintFolders`, `RenameTemplate`, `IsActive`.

### Fase B: Repositório e Aplicação
- Criar `IBlueprintStore` para persistência JSON.
- Criar `BlueprintAppService` para gerenciar a lógica de negócio.
- Atualizar o `WatchAppService` para monitorar caminhos de Blueprints separadamente de Jobs.

### Fase C: Interface (Apresentação)
- **BlueprintView**: Lista de diretórios com blueprints ativos (reaproveitando visual da Automação).
- **BlueprintEditor**: Modal focado apenas em caminho único, subpastas e renomeação.
- **JobEditor**: Remoção da aba de Organização para manter o foco em transporte.

### Fase D: Internacionalização
- Atualizar `JsonLocalizationService` com as novas chaves:
    - `Flows`, `Blueprint`, `NewBlueprint`, `PathToOrganize`, etc.

## 4. Cronograma de Execução
1. Atualização do Domínio e Localização.
2. Limpeza do JobEditor atual.
3. Criação da infraestrutura do módulo Blueprint.
4. Implementação da UI do Blueprint.
5. Integração final com o motor de Watcher.
