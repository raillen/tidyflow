# Planejamento Premium: Torre de Controle de Automação (Automation Hub)

Este documento detalha as funcionalidades avançadas para transformar o AutoFlow em um sistema de orquestração de arquivos de nível enterprise.

## 1. Diferenciação Visual Extrema
- **Watch Folders (O Monitor):**
  - Identidade visual baseada em tons de Verde Esmeralda/Ciano.
  - Efeito de "pulse" sutil no ícone quando em estado Idle (vigilância).
  - Texto de status "Vigiando..." com micro-animação.
- **Cópia Direta (A Execução):**
  - Identidade visual baseada em Azul Royal/Roxo.
  - Foco em botões de ação imediata (Play/Stop).

## 2. Ações em Massa (Batch Operations)
- **Modo de Seleção:** Checkboxes à esquerda de cada tarefa.
- **Barra de Ação Flutuante:** Surge quando 1+ itens são selecionados.
  - `[ Iniciar Selecionados ]`
  - `[ Pausar Selecionados ]`
  - `[ Parar Selecionados ]`
  - `[ Excluir Selecionados ]`
- **Global Toggle:** Selecionar/Desmarcar todos os jobs visíveis.

## 3. Visão Expansível (Inline Details)
- **Painel Interno:** Acionado pela seta lateral em cada job.
- **Dados Técnicos:**
  - Exibição de caminhos completos com botão "Abrir no Explorer".
  - Detalhes de agendamento legíveis (ex: "Próxima execução em 12min").
  - Histórico de integridade (Último Hash validado).
  - Seção de Erro Forense: Exibe a última falha técnica sem precisar sair da tela.

## 4. Rolling Log (Terminal em Tempo Real)
- **Live Feed:** Lista dos últimos 5-10 arquivos processados pela tarefa.
- **Visual:** Estilo console/terminal compacto dentro do card expandido.
- **Feedback Colapsado:** Uma linha de texto rotativa abaixo do nome da tarefa mostrando o arquivo que está "voando" no momento.

## 5. Gestão de Fluxo e Orquestração
- **Botão de Pânico (Kill All):** No cabeçalho global, encerra todas as operações e limpa a fila.
- **Priorização Dinâmica:** Arrastar tarefas para o topo da fila ativa.
- **Resolução de Conflitos Live:** Notificação visual dentro da lista quando um arquivo aguarda decisão de sobreposição.

## 6. Arquitetura Técnica
- **Event-Driven Progress:** O `ExecutionEngine` passa a reportar cada arquivo individual para o `GlobalProgressService`.
- **Circular Log Buffer:** `JobProgressInfo` manterá um histórico volátil (em memória) para o Rolling Log.
- **Reatividade MVVM:** `JobItemViewModel` gerenciará os estados `IsExpanded` e `IsSelected`.
