using Avalonia.Controls;
using Avalonia.Interactivity;
using JamReadyGui.Ui.Window;

namespace JamReadyGui.Views;

public partial class DebugWindow : Window
{
    public DebugWindow()
    {
        InitializeComponent();
    }

    private void Show_MainWindow(object? sender, RoutedEventArgs e)
    {
        SingleWindow.GetInstance<WorkspaceWindow>().Show();
    }
}