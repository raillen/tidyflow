# Planejamento Premium: Módulo de Configurações - FolderFlow

Este documento detalha a visão para transformar as configurações do FolderFlow em um centro de controle avançado.

## 1. Performance & Engine
- **Concurrency Control:** Slider para definir o número de arquivos processados simultaneamente (1-10).
- **CPU Scheduler:** Opção para definir a prioridade do processo no Windows (Background, Balanced, Performance).
- **Transfer Limit:** Possibilidade de definir um limite global de largura de banda (B/s).

## 2. Automação do Sistema
- **Windows Integration:** 
  - Toggle "Iniciar com o Windows".
  - Toggle "Minimizar para System Tray ao fechar".
  - Opção para "Iniciar Minimizado".

## 3. Notificações Pro
- **Webhooks:** Suporte a múltiplos canais (Discord, Slack, Teams) com filtros por tipo de evento (Sucesso, Apenas Erros, Resumo Diário).
- **SMTP Support:** Envio de e-mails detalhados de auditoria.

## 4. Manutenção & Banco de Dados
- **SQLite Maintenance:**
  - Auto-cleanup de registros antigos (Retention Policy).
  - Botão "Optimize Database" (SQL VACUUM).
  - Tamanho atual do banco visível em tempo real.

## 5. UI & Comportamento
- **Theme Engine:** Sincronização automática com o tema do Windows ou escolha manual (Light/Dark/High Contrast).
- **Language Switcher:** Suporte a Português, Inglês e Espanhol.
- **Dashboard Refresh Rate:** Ajustar o intervalo de atualização do monitor de hardware (1s a 10s).

## 6. Segurança
- **App Lock:** Proteção da interface por senha ou PIN.
- **Master Encryption Key:** Configuração global de chave para facilitar novos jobs criptografados.
