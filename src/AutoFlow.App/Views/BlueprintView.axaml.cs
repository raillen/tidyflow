using Avalonia.Controls;
using Avalonia.Markup.Xaml;

namespace AutoFlow.App.Views;

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
