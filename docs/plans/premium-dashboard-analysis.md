# Análise Premium do Dashboard - AutoFlow

Este documento detalha o plano de evolução do Dashboard do AutoFlow para torná-lo um produto premium e focado em inteligência operacional.

## 1. Métricas de Impacto (Premium Insights)
- **Economia de Tempo:** Calculada com base na média de processamento manual (`arquivos_processados * X segundos`).
- **Volume de Dados:** Exibição do total de TB/GB movimentados em vez de apenas contadores de arquivos.
- **Health Score (Saúde):** Indicador percentual (0-100%) da estabilidade do sistema nas últimas 24h.
- **Previsão de Conclusão (ETA):** Tempo estimado para o término de jobs ativos.

## 2. Visualização de Dados (Gráficos)
- **Gráfico de Throughput:** Monitoramento da vazão (velocidade) em tempo real nos últimos 60 minutos.
- **Heatmap de Erros:** Identificação visual de quais jobs ou horários possuem maior falha.

## 3. Comandos Rápidos e Automação
- **Pause/Resume Master:** Interruptor global para todas as operações.
- **Modo Turbo:** Aumento dinâmico de prioridade do processo e threads.
- **Limpeza Automática:** Atalho para manutenção preventiva do sistema.

## 4. Melhorias de UI/UX
- **Status de Armazenamento:** Barras de progresso para espaço livre em drives de Origem/Destino.
- **Timeline de Agendamentos:** "Radar" dos próximos eventos programados.
- **Micro-interações:** Feedback visual imediato em operações de sucesso/erro.

## 5. Arquitetura Técnica (Migração para SQLite)
- Substituição do motor de logs baseado em CSV por **SQLite** para:
  - Consultas rápidas (Dashboard Instantâneo).
  - Indexação de grandes volumes de dados.
  - Relatórios históricos complexos.
  - Campos adicionais obrigatórios: `FileSize (long)`, `Duration (double)`, `StatusDetails`.
