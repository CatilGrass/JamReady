using System;
using System.Collections.Generic;

namespace JamReadyGui.Ui.Window;

public static class SingleWindow
{
    public static Dictionary<Type, Avalonia.Controls.Window?> Instance { get; set; } = new();

    public static Avalonia.Controls.Window GetInstance<TWindow>() where TWindow : Avalonia.Controls.Window, new()
    {
        var windowType = typeof(TWindow);
        if (Instance.TryGetValue(windowType, out var value))
            if (value is { IsVisible: true })
                return value;
        var window = new TWindow();
        Instance[windowType] = window;
        return window;
    }
}