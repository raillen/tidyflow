using Avalonia.Controls;
using Avalonia.Interactivity;

namespace AutoFlow.App.Views;

public partial class ChangelogWindow : Window
{
    public ChangelogWindow()
    {
        InitializeComponent();
    }

    public void CloseClick(object? sender, RoutedEventArgs e)
    {
        Close();
    }
}