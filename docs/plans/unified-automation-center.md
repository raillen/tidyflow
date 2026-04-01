# Planejamento Premium: Central de Automação Unificada (Unified Hub)

Este documento descreve a unificação dos módulos de Jobs e Queue em uma única experiência fluida e poderosa.

## 1. O Conceito: "Control Center"
A substituição da `JobsView` e `QueueView` por uma única `AutomationCenterView`. O foco sai da configuração estática e passa para o ciclo de vida dinâmico da tarefa.

## 2. Componentes da Interface
### A. Painel de Execução (The Stage)
- Localizado no topo da página.
- Exibe cards informativos das tarefas que estão consumindo recursos (CPU/Rede) no momento.
- Métricas em tempo real: Velocidade, ETA e Arquivo atual.

### B. Lista Unificada de Tarefas
- Uma grade/lista inteligente onde cada item possui estados visuais distintos:
  - **Estado Ativo:** ProgressBar integrada, controles de interrupção.
  - **Estado Pendente:** Indicador de posição na fila (#1, #2...) e botões de reordenação de prioridade.
  - **Estado Ocioso:** Resumo do próximo agendamento e botão de início rápido.
  - **Estado de Erro:** Destaque em vermelho com botão de "Ver Erro" que abre o painel lateral forense.

## 3. Ações e Menus Rápidos
- **Global Actions:** "Pausar Tudo", "Retomar Tudo", "Limpar Concluídos".
- **Context Actions (Right Click):** 
  - "Priorizar" (Move para o topo da fila).
  - "Editar Configurações" (Abre o editor).
  - "Ver Logs" (Navega para o histórico com filtro).

## 4. Benefícios Premium
- **Fim da Navegação Fragmentada:** O usuário monitora e configura no mesmo lugar.
- **Feedback Imediato:** Ao clicar em "Rodar", o item visualmente se transforma em um "Ativo" ou "Pendente" na mesma hora.
- **Escalabilidade:** Uso de Agrupamento (Groups) para separar "Watch Folders" de "Cópias Diretas" automaticamente.

## 5. Passos Técnicos para Unificação
1. Mesclar `JobsViewModel` e `QueueViewModel` em `AutomationViewModel`.
2. Criar `AutomationView.axaml` com o layout de seções dinâmicas.
3. Implementar `DataTemplateSelectors` para mudar o visual da linha baseado no estado do Job.
4. Remover a entrada "Orquestrador" do menu lateral, renomeando "Tarefas" para "Automação".
