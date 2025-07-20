using System;
using System.Collections.Generic;
using JamReadyGui.AppData.Explorer.Adapters;

namespace JamReadyGui.AppData.Explorer.Inserters;

public class TestInserter : ItemInserter
{
    public override List<ItemAdapter?> GetAdapters(string path)
    {
        var list = new List<ItemAdapter?>();
        for (int i = 0; i < 40; i++)
        {
            var adapter = AdapterFactory.Create<TestAdapter>($"Hello {i}");
            list.Add(adapter);
        }
        return list;
    }
}
