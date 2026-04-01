# Planejamento Premium: Orquestrador de Fila (Queue Manager) - FolderFlow

Este documento detalha a evolução da fila de automação para um sistema de orquestração inteligente e interativo.

## 1. Nova Visualização: Queue Manager View
- **Dedicated Page:** Uma nova aba no menu lateral exclusivamente para gerenciar o que está acontecendo "Agora" e "Próximo".
- **Visual States:**
  - **Running:** Com progresso detalhado e métricas de velocidade por tarefa.
  - **Pending:** Lista ordenada de tarefas aguardando recursos.
  - **Paused:** Tarefas suspensas com opção de retomada.
  - **Completed (Session):** Resumo das tarefas finalizadas desde a abertura do app.

## 2. Controle e Orquestração
- **Manual Reordering:** Botões de "Subir/Descer" ou Drag & Drop para mudar a prioridade na fila.
- **Run Now (Force Start):** Comando para ignorar a fila e iniciar o processamento imediato.
- **Per-Job Throttling:** Configurar limites de banda e threads específicos para uma tarefa na fila, sobrepondo o global.
- **Skip Current File:** Botão para pular o arquivo atual que está travando a fila sem cancelar o Job inteiro.

## 3. Inteligência de Processamento
- **Smart Checkpoints:** Salvar o estado de progresso para retomada após reinicialização.
- **Resource Aware:** Pausar ou reduzir threads se o uso de CPU global do sistema ultrapassar 90%.
- **Conflict Resolver:** Se a fila detectar um conflito de arquivo, abrir um pequeno popup ou aba de "Atenção" na fila para o usuário decidir (Overwrite/Skip/Rename) em tempo real.

## 4. UI/UX e Feedback
- **Sparkline per Item:** Gráfico de performance individual para cada tarefa ativa.
- **Detailed Tooltips:** Mostrar a lista de próximos 5 arquivos a serem processados ao passar o mouse.
- **Status Badges:** "Criptografando", "SFTP Upload", "Delta Sync" - ícones claros do que o motor está fazendo no momento.

## 5. Menus Rápidos (Context Menu)
- **"Prioridade Máxima":** Move para o topo e aloca o dobro de threads.
- **"Pausar após o arquivo atual":** Permite uma parada limpa.
- **"Abrir pasta temporária":** Ver arquivos que estão sendo preparados para envio (ex: ZIP ou Encriptação).
