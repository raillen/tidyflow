using System;
using System.ComponentModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Reactive;
using AutoFlow.App.ViewModels;

namespace AutoFlow.App.Views;

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
            // Só esconde se o DataContext (ViewModel) já estiver carregado e pronto
            if (state == WindowState.Minimized && DataContext is MainWindowViewModel viewModel)
            {
                bool closeToTray = viewModel.Settings?.Settings?.CloseToTray ?? true;
                if (closeToTray)
                {
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