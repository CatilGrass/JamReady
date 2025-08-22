using Avalonia.Controls;
using JamReadyGui.ViewModels;

namespace JamReadyGui.Views;

public partial class WorkspaceWindow : Window
{
    public WorkspaceWindow()
    {
        InitializeComponent();
        DataContext = new WorkspaceViewModel();
    }
}