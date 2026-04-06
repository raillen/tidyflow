using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;

namespace AutoFlow.App.Views;

public partial class BlueprintEditorWindow : Window
{
    public BlueprintEditorWindow()
    {
        InitializeComponent();
#if DEBUG
        this.AttachDevTools();
#endif
    }

    private void InitializeComponent()
    {
        AvaloniaXamlLoader.Load(this);
    }
}
