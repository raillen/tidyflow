using Avalonia.Controls;
using Avalonia.Input;

namespace FolderFlow.App.Views;

public partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();
        
        // Ativa a movimentação da janela pela área personalizada de 45px no topo
        DragArea.PointerPressed += OnDragAreaPointerPressed;
    }

    private void OnDragAreaPointerPressed(object? sender, PointerPressedEventArgs e)
    {
        // Se o usuário clicar com o botão esquerdo, inicia o arraste da janela
        if (e.GetCurrentPoint(this).Properties.IsLeftButtonPressed)
        {
            BeginMoveDrag(e);
        }
    }
}