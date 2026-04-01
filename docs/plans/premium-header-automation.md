# Planejamento Premium: Header da Central de Automação

Este documento detalha a reformulação estética e funcional do cabeçalho da Central de Automação para atingir o padrão visual do FolderFlow.

## 1. Identidade Visual
- **Glass Effect:** Aplicação de fundo translúcido com gradiente linear sutil (60% para 90% de opacidade).
- **Subtle Shadow:** Sombra externa leve na parte inferior para criar elevação sobre o conteúdo em scroll.
- **Refined Typography:** Título com `FontWeight="Black"` mas tamanho equilibrado, acompanhado de um subtítulo informativo em `Opacity="0.6"`.

## 2. Indicadores em Tempo Real (Status Badges)
- Implementação de pequenos contadores ao lado do título:
  - **Running Badge:** Círculo verde pulsante + número de tarefas ativas.
  - **Queue Badge:** Ícone de relógio + número de tarefas aguardando.

## 3. Experiência de Botões (Button UX)
- **Primary Action:** Botão "Nova Cópia" com gradiente de marca e ícone centralizado.
- **Secondary Actions:** Botões de "Pausar Tudo" e "Configurações de Fila" com estilo minimalista (Outline) que se preenchem no hover.
- **Micro-interações:** Todos os botões terão transições de 0.2s na escala e cor.

## 4. Busca e Navegação por Segmentos
- **Search Field:** Design sem bordas pesadas, apenas uma linha inferior ou fundo levemente mais escuro.
- **Segmented Control (Chips):** Em vez de ComboBox, usaremos botões de estilo "Chip" para filtrar entre:
  - [ Todos ] [ Ativos ] [ Pendentes ] [ Erros ]

## 5. Reatividade
- O Header deve se comprimir levemente (Compact Header) quando o usuário fizer scroll para baixo, maximizando a área de visualização dos Jobs.
