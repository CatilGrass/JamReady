using System;
using JamReadyGui.Data;
using ReactiveUI;

namespace JamReadyGui.ViewModels;

public class WorkspaceViewModel : ReactiveObject
{ 
    private string _workspaceDirectory;
    public string WorkspaceDirectory
    {
        get => _workspaceDirectory;
        set
        {
            if (value == _workspaceDirectory) return;
            ReloadWorkspace();
            this.RaiseAndSetIfChanged(ref _workspaceDirectory, value);
        }
    }

    public WorkspaceViewModel()
    {
        var config = AppConfig.Read();
        _workspaceDirectory = config.Workspace;
        config.Save();

        ReloadWorkspace();
    }

    private void ReloadWorkspace()
    {
        
    }
}