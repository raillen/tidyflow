# Changelog - TidyFlow

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.

## [Unreleased] - 2026-10-30

### Adicionado
- **i18n completo da UI**: dicionários `pt-BR`/`en-US` (882 chaves), `localeStore`,
  `t(key, params)`, `currentLocale()`; listas de opções/constantes convertidas em
  funções reativas ao locale.
- **Segredos de notificação criptografados em repouso** (AES-256-GCM via `ring`),
  chave mestre no OS keyring ou arquivo `0600`.
- **Single-instance lock** por data dir via `try_lock()`.
- **PIN ao ir para a bandeja** (close-to-tray e menu tray `hide`).
- **Campo de template de renomeação** (`template_override`) no editor de
  estruturas de pastas, com regex de validação.
- **`DomainError::FolderOutOfPattern`** em `handle_folder_auto_correct`.
- **`SecretVault`**, `seal_job_secrets`/`open_job_secrets`.

### Corrigido
- Clippy limpo no workspace (large enum variant, too many args, assign op, etc.).
- `namingPatternSchema` aceita `null` do backend.
- `naming_pattern` não perde `templateOverride` ao editar a regex.

### Documentação
- `STATUS.md` com o estado atual e pendências funcionais restantes.

## [v0.2.1-alpha] - 2026-06-20

### Adicionado
- **Blueprints de arquivos e pastas** com routing por template, contadores,
  variáveis, ações de pasta (`create`, `rename`, `move`, `copy`, `delete`),
  preview, simulação e auto-correct por watch.
- **Tokenizer de templates** com tokens, transforms, regex presets, parser de
  texto com escapes e round-trip (`parse_template`/`format_template`).
- **Variáveis de pasta** resolvidas em templates (`{projectName}`), com
  validação de identificador e modal de execução.
- **Auto-correct seguro**: mesmo resolvedor e preflight do apply manual,
  validação de caminhos, confirmação de exclusão e proteção contra symlinks.
- **Watch de blueprints** com recursão realtime, auto-correct no polling,
  serialização por blueprint e supressão de eventos próprios.

### Corrigido
- Workspace Rust voltou a compilar: tipos de domínio de folder plan e
  `BlueprintApplyResult` restaurados.
- Contratos Tauri/IPC alinhados (`apply` retorna `processed/failed/skipped`,
  preview de plano recebe variáveis).
- Ordenação determinística de arquivos para `{index}` e contadores.
- Datas/horas dos templates usam o fuso local; counter usa data local.
- `create_intermediate_dirs` agora é respeitado.
- `sanitize_windows_filename` cobre nomes reservados e caracteres de controle.

### Documentação
- Gramática real do tokenizer documentada em `docs/v2/DOMAIN.md`.
- `docs/v2/README.md`, `BLUEPRINT-SPEC.md` e `API-IPC.md` alinhados ao código.
- Documentos C#/Avalonia marcados como legado.

---

## [v1.4.1] - 2026-04-06
### Adicionado
- **Responsividade do Dashboard:** Implementado UniformGrid e TextTrimming nos cards de métricas para evitar quebras em janelas pequenas.
- **Ações Rápidas em Fluxos:** Novos atalhos para criação de tarefas manuais e monitoradas no módulo de Automação.
- **Melhorias na Auditoria:** Novo layout de detalhes para logs de operação, corrigindo problemas de informações cortadas.
- **Navegação Inteligente:** Destaque visual (fundo e cor) para a aba ativa na barra lateral.

## [v1.4.0] - 2026-04-06
### Adicionado
- **Tipografia Personalizável:** Suporte a múltiplas fontes (Inter, Roboto, Montserrat, Open Sans, Public Sans e JetBrains Mono).
- **Ajustes em Tempo Real:** Alterações de tema, fonte e efeitos visuais agora são aplicadas instantaneamente sem reiniciar.
- **Efeito Mica & Glass:** Suporte nativo ao Windows 11 Mica e AcrylicBlur com slider de opacidade variável para a barra lateral.
- **Estabilidade de Interface:** Definido tamanho mínimo de janela (950x650) para garantir integridade visual.

## [v1.3.4] - 2026-04-02
### Adicionado
- **Painel de Controle de Automação:** Visão geral com métricas reais (Total, Ativos, Volume) e monitor de progresso em tempo real.
- **Painel de Controle de Blueprints:** Estatísticas de templates e atalhos rápidos de criação.
- **Auditoria Avançada:** Abas filtradas para Incidentes (Falhas) e Sistema (Logs técnicos).
- **Ajuste de Margem Superior Final:** Aumentado o buffer global para **60px** na MainWindow para garantir 100% de visibilidade dos botões do Windows.

## [v1.3.3] - 2026-04-02
### Adicionado
- **Padronização Master-Detail:** Unificação visual de todos os módulos principais (Automação, Blueprints, Histórico) usando abas laterais.
- **Acessibilidade Responsiva:** Ajuste automático de padding superior quando a janela está maximizada.

## [v1.3.2] - 2026-04-02
### Adicionado
- **Notificações Nativas (Toasts):** Integração com a Central de Ações do Windows para alertas em background.
- **Restauração do Tray:** Lógica aprimorada para focar a janela ao clicar no ícone da bandeja.

## [v1.3.1] - 2026-04-02
### Adicionado
- **Bandwidth Throttling:** Motor de limitação de banda para SFTP, ZIP e Delta Sync.
- **Sessão Sobre:** Informações do desenvolvedor e links sociais.
- **Changelog Interno:** Visualizador de notas de versão integrado.

---
*AutoFlow Pro - Desenvolvido por Raillen Santos*
