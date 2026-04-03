# Plano de Implementação: Reestruturação de Configurações, Sessão Sobre e Changelog

## 1. Objetivo
Organizar o módulo de configurações em abas laterais, corrigir a acessibilidade do menu superior e implementar a sessão "Sobre" com informações do desenvolvedor e histórico de atualizações.

## 2. Mudanças Visuais (SettingsView.axaml)
- **Ajuste de Margem:** Adicionar `Margin="0,35,0,0"` ao Grid principal.
- **Implementação de TabControl:**
    - Aba Geral: Tema, Idioma, Opacidade, Comportamento Windows.
    - Aba Performance: Paralelismo, Prioridade, Limite de Banda.
    - Aba Segurança: PIN, Chave de Criptografia.
    - Aba Notificações: Webhooks, SMTP.
    - Aba Manutenção: Logs, SQLite.
    - Aba Apoio: Doações.
    - Aba Sobre: Versão, Links Sociais (GitHub/LinkedIn), Bio do Raillen Santos e Botão de Changelog.

## 3. Mudanças no ViewModel (SettingsViewModel.cs)
- Implementar `OpenLinkCommand` para abrir URLs externas.
- Implementar `ShowChangelogCommand` para exibir o histórico de versões.
- Atualizar a versão do sistema para v1.3.2.

## 4. Localização (JsonLocalizationService.cs)
- Adicionar chaves para os títulos das novas abas: `General`, `Performance`, `Security`, `Notifications`, `Maintenance`, `About`.

## 5. Cronograma de Execução
1. Salvar este plano (OK).
2. Sair do modo de planejamento.
3. Aplicar as mudanças no XAML.
4. Aplicar as mudanças no C#.
5. Atualizar os arquivos de tradução.
6. Validar build e execução.
