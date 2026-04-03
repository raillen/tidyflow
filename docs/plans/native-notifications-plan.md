# Plano de Implementação: Notificações Nativas do Windows

## 1. Objetivo
Permitir que o FolderFlow envie notificações para o Desktop (Central de Ações do Windows) mesmo quando estiver minimizado no tray, utilizando recursos nativos do sistema operacional.

## 2. Abordagem Técnica
Para evitar dependências de pacotes UWP pesados ou complexos, utilizaremos o comando `New-BurntToastNotification` via PowerShell (se disponível) ou, como fallback universal, o objeto `Windows.UI.Notifications` disparado via script inline do PowerShell.

## 3. Mudanças na Infraestrutura (`FolderFlow.Infrastructure`)
- Criar `WindowsNotificationHelper` na pasta `Helpers`.
- Este helper terá um método `SendToast(title, message, iconPath)`.
- O método executará um processo `powershell.exe` em background com os argumentos necessários para disparar a notificação nativa.

## 4. Mudanças no Serviço de Notificação (`AvaloniaNotificationService`)
- Atualizar a lógica do `Show(title, message, isError)`.
- **Regra de Decisão:**
    - Se a `MainWindow` estiver visível e ativa: Usa o `WindowNotificationManager` (pop-up interno atual).
    - Se a `MainWindow` estiver oculta (minimizado no tray) ou não for a janela em foco: Dispara a notificação via `WindowsNotificationHelper`.

## 5. Passos de Execução
1.  Implementar `WindowsNotificationHelper.cs`.
2.  Atualizar `AvaloniaNotificationService.cs` para injetar a lógica de detecção de estado da janela.
3.  Testar disparando uma notificação com o app minimizado.

## 6. Vantagens
- **Leveza:** Zero DLLs externas adicionais.
- **UX:** O usuário recebe feedback visual imediato de erros ou conclusões de Jobs sem precisar abrir o app.
- **Consistência:** Segue o padrão visual do Windows 10/11.
