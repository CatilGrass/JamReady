using System;
using System.Collections.Generic;
using System.Text;

namespace JamReadyGui.AppData.Utils;

public struct ExplorerPath
{
    /// <summary>
    /// 路径前缀
    /// </summary>
    public string Prefix
    {
        get => _prefix.Trim().ToUpper();
        set => _prefix = value.Trim().ToUpper();
    }

    /// <summary>
    /// 路径
    /// </summary>
    public string Path
    {
        get => _path.Trim();
        set => _path = PathFormatter.FormatPath(value);
    }

    /// <summary>
    /// 参数
    /// </summary>
    /// <param name="key"></param>
    public string this[string key]
    {
        get => _arguments.GetValueOrDefault(key, "");
        set => _arguments[key] = value;
    }
    
    private string _prefix = "NONE";
    private string _path = "/";
    private readonly Dictionary<string, string> _arguments = new();

    public ExplorerPath(string prefix = "FS")
    {
        _prefix = prefix;
    }

    /// <summary>
    /// 转换为字符串
    /// </summary>
    /// <returns></returns>
    public override string ToString()
    {
        try
        {
            // 处理空路径情况
            if (IsNone()) return "NONE://";
            
            // 无参数情况
            if (_arguments.Count == 0)
            {
                return $"{Prefix}://{Path}";
            }

            // 构建参数字符串
            var paramBuilder = new StringBuilder("[");
            bool isFirst = true;
            foreach (var kv in _arguments)
            {
                if (!isFirst) paramBuilder.Append(',');
                paramBuilder.Append($"{kv.Key}:\"{kv.Value.Replace("\"", "\\\"")}\"");
                isFirst = false;
            }
            paramBuilder.Append(']');

            // 路径为空的情况
            if (string.IsNullOrWhiteSpace(Path))
            {
                return $"{Prefix}://{paramBuilder}";
            }
            
            return $"{Prefix}://{paramBuilder}://{Path}";
        }
        catch
        {
            return "NONE://";
        }
    }
    
    /// <summary>
    /// 从字符串获得路径
    /// </summary>
    /// <param name="input"></param>
    /// <returns></returns>
    public static ExplorerPath? FromString(string input)
    {
        try
        {
            if (string.IsNullOrWhiteSpace(input)) 
                return new ExplorerPath();

            input = input.Trim();
            
            // 解析前缀
            int prefixEnd = input.IndexOf("://", StringComparison.Ordinal);
            if (prefixEnd < 0)
            {
                var result = new ExplorerPath { Path = input };
                return result;
            }

            var explorerPath = new ExplorerPath 
            { 
                Prefix = input.Substring(0, prefixEnd) 
            };

            string remaining = input.Substring(prefixEnd + 3);

            // 处理参数
            if (remaining.StartsWith("["))
            {
                int paramEnd = remaining.IndexOf(']');
                if (paramEnd < 0)
                {
                    ParseArguments(remaining[1..], explorerPath);
                    return explorerPath;
                }

                ParseArguments(remaining[1..paramEnd], explorerPath);
                remaining = remaining[(paramEnd + 1)..];
            }

            // 处理路径
            if (remaining.StartsWith("://"))
            {
                explorerPath.Path = remaining[3..];
            }
            else if (!string.IsNullOrWhiteSpace(remaining))
            {
                explorerPath.Path = remaining;
            }

            return explorerPath;
        }
        catch
        {
            return null;
        }
    }

    /// <summary>
    /// 是否为空
    /// </summary>
    /// <returns></returns>
    public bool IsNone()
    {
        return _prefix == "NONE";
    }

    /// <summary>
    /// 处理参数部分
    /// </summary>
    /// <param name="input"></param>
    /// <param name="explorerPath"></param>
    private static void ParseArguments(string input, ExplorerPath explorerPath)
    {
        if (string.IsNullOrWhiteSpace(input)) return;
        
        int position = 0;
        while (position < input.Length)
        {
            // 跳过空格
            while (position < input.Length && char.IsWhiteSpace(input[position])) 
                position++;
            if (position >= input.Length) break;
            
            // 提取键
            int keyStart = position;
            while (position < input.Length && input[position] != ':' && 
                   !char.IsWhiteSpace(input[position]) && input[position] != ',')
            {
                position++;
            }
            string key = input[keyStart..position].Trim();
            
            // 查找冒号
            while (position < input.Length && input[position] != ':') 
                position++;
            if (position >= input.Length) break;
            position++; // 跳过冒号
            
            // 跳过冒号后的空格
            while (position < input.Length && char.IsWhiteSpace(input[position])) 
                position++;
            if (position >= input.Length) break;
            
            // 处理引号包裹的值
            char quote = '\0';
            if (input[position] == '"' || input[position] == '\'')
            {
                quote = input[position];
                position++;
            }
            
            // 提取值
            int valueStart = position;
            while (position < input.Length && 
                   (quote == '\0' 
                       ? (input[position] != ',' && !char.IsWhiteSpace(input[position]))
                       : input[position] != quote))
            {
                position++;
            }
            string value = input[valueStart..position];
            
            // 处理结束引号
            if (quote != '\0' && position < input.Length)
            {
                position++;
            }
            
            // 添加到结果
            if (!string.IsNullOrWhiteSpace(key))
            {
                explorerPath[key] = value;
            }
            
            // 跳过逗号和空格
            while (position < input.Length && input[position] != ',') 
                position++;
            if (position < input.Length && input[position] == ',')
            {
                position++;
            }
        }
    }
}