using System;
using System.Windows.Input;
using Microsoft.Xaml.Behaviors.Core;

namespace JamReadyGui.AppWindows.AppExplorer.ExplorerData.Commands;

public class RelayCommand<T> : ICommand
{
    private readonly Action<T> _execute;
    private readonly ICommand _commandImplementation = new ActionCommand(() => { });

    public RelayCommand(Action<T> execute) => _execute = execute;
    
    public bool CanExecute(object? parameter) => true;
    
    public void Execute(object? parameter)
    {
        if (parameter is T typedParam)
            _execute(typedParam);
    }

    public event EventHandler? CanExecuteChanged
    {
        add => _commandImplementation.CanExecuteChanged += value;
        remove => _commandImplementation.CanExecuteChanged -= value;
    }
}