# Changelog - FolderFlow Pro

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.

## [v1.3.3] - 2026-04-02
### Adicionado
- **Padronização Master-Detail:** Unificação visual de todos os módulos principais (Automação, Blueprints, Histórico) usando abas laterais.
- **Acessibilidade Responsiva:** Ajuste automático de padding superior quando a janela está maximizada para evitar sobreposição com botões do sistema Windows.
- Margem de 35px padronizada em todas as sub-sidebars para alinhamento vertical perfeito.

## [v1.3.2] - 2026-04-02
### Adicionado
- **Notificações Nativas (Toasts):** Integração com a Central de Ações do Windows para alertas quando o app está minimizado.
- **Restauração do Tray:** Lógica aprimorada para trazer a janela ao foco ao clicar no ícone da bandeja.
### Corrigido
- **Deadlock no Startup:** Carregamento de configurações agora utiliza método síncrono `Load()` para evitar travamentos na inicialização da UI.
- **Estabilidade do XAML:** Substituição de conversores de ícone complexos por painéis de visibilidade estática.

## [v1.3.1] - 2026-04-02
### Adicionado
- **Bandwidth Throttling:** Implementação completa do motor de limitação de banda para SFTP, ZIP e Delta Sync.
- **Sessão Sobre:** Nova tela com informações do desenvolvedor (Raillen Santos) e links sociais.
- **Changelog Interno:** Visualizador de notas de versão integrado ao software.

## [v1.3.0] - 2026-04-02
### Adicionado
- **Smart Renaming Engine:** Sistema de renomeação automática baseado em tokens.
- **Módulo de Blueprints:** Separação da lógica de organização da lógica de transferência.

---
*FolderFlow Pro - Desenvolvido por Raillen Santos*
