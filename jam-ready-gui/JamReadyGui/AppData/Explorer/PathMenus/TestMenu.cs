using System;
using System.Collections.Generic;

namespace JamReadyGui.AppData.Explorer.PathMenus;

public class TestMenu : PathMenu
{
    public override string GetMenuName()
    {
        return "Test";
    }

    public override List<string>? OnRegisterOperation(string path)
    {
        return new()
        {
            "Test Button 1",
            "Test Button 2",
            "Test Button 3",
            "Test Button 4",
            "Test Button 5"
        };
    }

    public override void OnOperate(string path, int operationIndex)
    {
        Console.WriteLine($"Press Button: {operationIndex} at {path}");
    }
}