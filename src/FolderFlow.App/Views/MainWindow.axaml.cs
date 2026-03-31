using System;
using System.ComponentModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Reactive;
using FolderFlow.App.ViewModels;

namespace FolderFlow.App.Views;

public partial class MainWindow : Window
{
    private bool _isExiting = false;

    public MainWindow()
    {
        InitializeComponent();
        
        // Ativa a movimentação da janela pela área personalizada de 45px no topo
        DragArea.PointerPressed += OnDragAreaPointerPressed;

        // Monitora mudanças de estado (Minimizar)
        this.GetObservable(Window.WindowStateProperty).Subscribe(new AnonymousObserver<WindowState>(state =>
        {
            System.IO.File.AppendAllText("trace.txt", $"   -> WindowState changed to: {state}\n");
            if (state == WindowState.Minimized)
            {
                var viewModel = DataContext as MainWindowViewModel;
                bool closeToTray = viewModel?.Settings?.Settings?.CloseToTray ?? true;
                
                System.IO.File.AppendAllText("trace.txt", $"   -> closeToTray: {closeToTray}\n");
                if (closeToTray)
                {
                    System.IO.File.AppendAllText("trace.txt", $"   -> Calling Hide()...\n");
                    Hide();
                }
            }
        }));
    }

    protected override void OnClosing(WindowClosingEventArgs e)
    {
        var viewModel = DataContext as MainWindowViewModel;
        bool closeToTray = viewModel?.Settings?.Settings?.CloseToTray ?? true;

        // Se não for um fechamento definitivo (pelo menu Sair do tray) e CloseToTray estiver ativo
        if (!_isExiting && closeToTray)
        {
            e.Cancel = true;
            Hide();
        }
        
        base.OnClosing(e);
    }

    public void ForceClose()
    {
        _isExiting = true;
        Close();
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