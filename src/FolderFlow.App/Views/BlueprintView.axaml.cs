using Avalonia.Controls;
using Avalonia.Markup.Xaml;

namespace FolderFlow.App.Views;

public partial class BlueprintView : UserControl
{
    public BlueprintView()
    {
        InitializeComponent();
    }

    private void InitializeComponent()
    {
        AvaloniaXamlLoader.Load(this);
    }
}
