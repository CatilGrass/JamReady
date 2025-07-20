using System.Collections.Generic;
using JamReadyGui.AppData.Explorer.Inserters;
using JamReadyGui.AppData.Explorer.PathMenus;

namespace JamReadyGui.AppData.Explorer;

public static class Registry
{
    public static List<ItemInserter> Inserters = new()
    {
        new TestInserter()
    };
    
    public static List<PathMenu> PathMenus = new()
    {
        new TestMenu()
    };
}