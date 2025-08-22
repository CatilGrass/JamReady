using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using JamReadyGui.Ui.Window;
using JamReadyGui.Views;

namespace JamReadyGui;

public partial class App : Application
{
    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
#if DEBUG
            desktop.MainWindow = SingleWindow.GetInstance<DebugWindow>();
#elif RELEASE
            desktop.MainWindow = SingleWindow.GetInstance<WorkspaceWindow>();
#endif
        }

        base.OnFrameworkInitializationCompleted();
    }
}