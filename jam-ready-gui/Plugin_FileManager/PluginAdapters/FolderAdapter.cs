using System;
using System.Collections.Generic;
using JamReadyGui.AppData.Explorer;

namespace Plugin_FileManager.PluginAdapters;

public class FolderAdapter : ItemAdapter
{
    public override ImagePath OnInit(object value)
    {
        Name = "HAHA";
        return new ImagePath(new Uri("F:\\C\\Picture\\pzw.png"));
    }

    public override List<string> OnRegisterOperation()
    {
        return new List<string>();
    }

    public override void OnEnter()
    {
        
    }

    public override void OnOperate(int operationIndex)
    {
        
    }
}