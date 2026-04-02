# Plano de Refinamento: Blueprint Ultimate (Contadores e Ajuda Avançada)

Este plano descreve os ajustes finais para tornar o módulo de Blueprint impecável, focando na flexibilidade dos contadores e na educação do usuário através de dicas avançadas.

## 1. Motor de Contagem Profissional
- **Parâmetros de Contagem**: Atualizar o `OrganizationService` para que o token `{Counter}` respeite:
  - **Número Inicial**: Definido pelo campo "Número Inicial" na UI.
  - **Padding (Zeros)**: Suportado via `{Counter:N}`.
- **Unificação `{001}`**: O token `{001}` será mantido apenas como um "Atalho Mágico" na UI que insere `{Counter:3}`, eliminando a duplicidade de lógica no código.
- **Lógica de Prévia**: O preview deve considerar o `CounterStart` para mostrar o primeiro nome que será gerado.

## 2. Educação do Usuário (Help Tooltips)
- **Botão "?" de Regex Avançado**:
  - Explicar grupos de captura `()`.
  - Explicar retro-referências `$1`, `$2`.
  - Explicar quantificadores `*`, `+`, `?`, `{n,m}`.
- **Botão "?" de Estilos e Limpeza**:
  - Tabela comparativa entre `camelCase`, `PascalCase`, `snake_case`, etc.
  - Explicar o que o `:clean` remove exatamente.
  - Explicar como usar o `:take(n)` e `:skip(n)`.

## 3. UI: Refinamentos de Layout
- **Controles de Contador**: Adicionar um campo de "Zeros à esquerda" (Padding) ao lado do "Número Inicial" na seção de renomeio.
- **Botões de Informação**: Posicionar ícones de ajuda discretos, mas acessíveis, ao lado dos cabeçalhos das abas na Toolbox.

## 4. Cronograma de Execução

### Fase 1: Backend e Contrato
1. Atualizar a interface `IOrganizationService` para aceitar `counterStart`.
2. Implementar a nova lógica de soma no `OrganizationService.ProcessTemplate`.

### Fase 2: ViewModel e Lógica de Ajuda
1. Adicionar propriedades de texto longo para a ajuda avançada.
2. Garantir que a alteração de "Número Inicial" ou "Padding" na UI dispare a atualização da prévia.

### Fase 3: UI e Localização
1. Adicionar os botões de "?" com Flyouts informativos.
2. Inserir as chaves de localização detalhadas (PT-BR e EN-US).
