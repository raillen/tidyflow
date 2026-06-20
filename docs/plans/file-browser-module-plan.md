# Plano de Implementação: Módulo File Browser

## 1. Visão Geral

**Objetivo**: Criar um micro gerenciador de arquivos integrado ao AutoFlow, restrito aos paths registrados nos fluxos (Blueprints + Jobs).

**Arquitetura**: Clean Architecture com padrão MVVM (CommunityToolkit.Mvvm)

---

## 2. Estrutura de Diretórios Criados

```
src/
├── AutoFlow.Domain/Entities/
│   └── FileSystemItem.cs              ✓ CRIADO
├── AutoFlow.Application/Interfaces/
│   ├── IAuthorizedPathProvider.cs    ✓ CRIADO
│   └── IFileExplorerService.cs       ✓ CRIADO
├── AutoFlow.Application/Services/
│   └── FileExplorerService.cs        ✓ CRIADO
├── AutoFlow.Infrastructure/
│   └── AuthorizedPathProvider.cs    ✓ CRIADO
├── AutoFlow.App/ViewModels/
│   └── FileBrowserViewModel.cs      ✓ CRIADO
└── AutoFlow.App/Views/
    ├── FileBrowserView.axaml        ✓ CRIADO
    └── FileBrowserView.axaml.cs     ✓ CRIADO
```

---

## 3. Entidades e Interfaces

### 3.1 FileSystemItem (Domain/Entities/FileSystemItem.cs)

```csharp
public class FileSystemItem
{
    public string Name { get; set; } = string.Empty;
    public string FullPath { get; set; } = string.Empty;
    public bool IsDirectory { get; set; }
    public long Size { get; set; }
    public DateTime ModifiedAt { get; set; }
    public string? Extension { get; set; }
    public bool IsAuthorized { get; set; } = true;
    public bool IsHidden { get; set; }
    public bool IsReadOnly { get; set; }
}
```

### 3.2 IAuthorizedPathProvider

Responsável por fornecer os paths autorizados baseados nos fluxos registrados:

- **Blueprint.Path** → paths de modelos
- **Job.SourcePath** → paths de origem dos jobs
- **Job.TargetPath** → paths de destino dos jobs

### 3.3 IFileExplorerService

Serviço principal para operações de arquivo:

```csharp
public interface IFileExplorerService
{
    Task<IEnumerable<FileSystemItem>> GetDirectoryContentsAsync(string path);
    Task<IEnumerable<FileSystemItem>> SearchAsync(string path, string query);
    Task<IEnumerable<FileSystemItem>> FilterByExtensionAsync(string path, IEnumerable<string> extensions);
    bool IsPathAuthorized(string path);
    Task<string?> GetParentPathAsync(string path);
    Task<IEnumerable<string>> GetAuthorizedRootsAsync();
}
```

---

## 4. Funcionalidades Implementadas

| # | Funcionalidade | Status |
|---|----------------|--------|
| 1 | Listar arquivos dos paths autorizados | ✓ |
| 2 | Navegação (pastas .., subdiretórios) | ✓ |
| 3 | Seleção de arquivo/pasta | ✓ |
| 4 | Busca por nome (filtro em tempo real) | ✓ |
| 5 | Copiar/Recortar/Colar | ✓ |
| 6 | Split view (dois painéis) | ✓ |
| 7 | Caminho atual e navegação | ✓ |
| 8 | Copy path para clipboard | ✓ |
| 9 | Exportar path como JSON | ✓ |

---

## 5. Paleta de Cores (Tema Light)

Seguindo o sistema de temas existente do AutoFlow:

- **Fundo**: `#F4F5F7` (AppBackground)
- **Container**: `#FFFFFF` (CardBackground)
- **Texto Principal**: `#1D2129` (TextPrimary)
- **Texto Secundário**: `#4E5969` (TextSecondary)
- **Accent**: `#0064FF` (AccentColor)
- **Success/Pasta**: `#00B42A` (SuccessColor)
- **Danger**: `#F53F3F` (DangerColor)

---

## 6. ViewModels

### 6.1 FileBrowserViewModel

Propriedades principais:

- `AvailableRoots` - paths autorizados
- `SelectedRoot` - path selecionado
- `CurrentPath` - diretório atual
- `Items` - itens do diretório
- `SearchText` - texto de busca
- `IsSplitView` - modo split view
- `IsLoading` - indicador de carregamento

Comandos:

- `NavigateToPathCommand` - navegar para path
- `NavigateUpCommand` - navegar para pai
- `NavigateToItemCommand` - navegar para item
- `RefreshCurrentDirectoryCommand` - atualizar
- `FilterItemsCommand` - filtrar itens
- `CopyCommand` - copiar
- `CutCommand` - recortar
- `PasteCommand` - colar
- `ToggleSplitViewCommand` - alternar split
- `CopyPathCommand` - copiar path
- `ExportPathAsJsonCommand` - exportar JSON

### 6.2 FileItemViewModel

- `Name` - nome do arquivo/pasta
- `FullPath` - path completo
- `IsDirectory` - é diretório?
- `Size` - tamanho em bytes
- `SizeFormatted` - tamanho formatado
- `IsSelected` - está selecionado?
- `IsHidden` - está oculto?
- `IsReadOnly` - é somente leitura?

---

## 7. UI (FileBrowserView.axaml)

Estrutura:

1. **Header** - título e botão Split View
2. **Toolbar** - busca, filtros, botões de ação
3. **Main Panel** - grade de arquivos/pastas
4. **Path Bar** - navegação e refresh
5. **Status Bar** - caminho atual e contagem

Componentes:

- Campo de busca com ícone de lupa
- Botões Filter, Saved, Export
- Botões Copy, Cut, Paste, Settings
- Grade WrapPanel com cards de itens
- Ícones de pasta/arquivo
- Barra de caminho com botões

---

## 8. Integração com o Sistema

### Registro de Serviços (App.axaml.cs)

Para ativar o módulo, adicionar em `ConfigureServices`:

```csharp
// Services
services.AddSingleton<IAuthorizedPathProvider, AuthorizedPathProvider>();
services.AddSingleton<IFileExplorerService, FileExplorerService>();

// ViewModels
services.AddSingleton<FileBrowserViewModel>();
```

### Navegação

A View pode ser aberta via:

1. Como nova aba no TabControl principal
2. Como janela independentes (`FileBrowserWindow`)
3. Via menu de navegação

---

## 9. Próximos Passos

1. **Corrigir warnings de build** (async void no CopyPath)
2. **Integrar ao App.axaml.cs** (registrar serviços)
3. **Adicionar à navegação principal** (MainWindow)
4. **Testar com fluxos existentes**
5. **Melhorar ícones** (dinâmicos por tipo de arquivo)
6. **Adicionar preview** (texto, imagem)
7. **Menu contextual** (clique direito)

---

## 10. Dependências

- **AutoFlow.Domain** - entidades
- **AutoFlow.Application** - interfaces e serviços
- **AutoFlow.Infrastructure** - implementações
- **AutoFlow.App** - ViewModels e Views
- **CommunityToolkit.Mvvm** - padrão MVVM
- **Material.Icons.Avalonia** - ícones

---

## 11. Considerações de Segurança

- Apenas paths registrados em Blueprints/Jobs são acessíveis
- Validação de autorização em todas as operações
- Cache de paths autorizados invalidado em mudanças

---

*Data: 2026-04-07*
*Status: Implementação base concluída*