using System;
using JamReadyGui.Data.Base;

namespace JamReadyGui.Data;

public class AppConfig : DataFile<AppConfig>
{
    public string Workspace = Environment.CurrentDirectory;
}