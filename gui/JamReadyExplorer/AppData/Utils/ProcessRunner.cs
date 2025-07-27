using System;
using System.Diagnostics;
using System.IO;

namespace JamReadyGui.AppData.Utils;

/// <summary>
/// 进程运行器
/// </summary>
public static class ProcessRunner
{
    /// <summary>
    /// 运行进程
    /// 注意：若仅调用 jam.exe，请使用 AppCoreInvoker.Execute(commands);
    /// </summary>
    /// <param name="workDirectory"> 工作地址 </param>
    /// <param name="exeFile"> 程序地址 </param>
    /// <param name="args"> 参数 </param>
    /// <returns></returns>
    /// <exception cref="FileNotFoundException"></exception>
    /// <exception cref="DirectoryNotFoundException"></exception>
    public static ProcessRunnerResult Run(DirectoryInfo workDirectory, FileInfo exeFile, string[] args)
    {
        if (!exeFile.Exists)
            throw new FileNotFoundException($"The specified executable file was not found: {exeFile.FullName}");

        if (!workDirectory.Exists)
            throw new DirectoryNotFoundException($"The specified working directory was not found: {workDirectory.FullName}");

        try
        {
            using var process = new Process();
            process.StartInfo = new ProcessStartInfo
            {
                FileName = exeFile.FullName,
                Arguments = string.Join(" ", args),
                WorkingDirectory = workDirectory.FullName,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };

            process.Start();

            string output = process.StandardOutput.ReadToEnd();
            string error = process.StandardError.ReadToEnd();

            process.WaitForExit();

            return new ProcessRunnerResult(output, error, process.ExitCode);
        }
        catch (Exception ex)
        {
            return new ProcessRunnerResult(string.Empty, $"An error occurred while executing the process: {ex.Message}", -1);
        }
    }
}

/// <summary>
/// 结果
/// </summary>
public struct ProcessRunnerResult
{
    public readonly string Output;
    public readonly string Error;
    public readonly int ExitCode;

    public ProcessRunnerResult(string output, string error, int exitCode)
    {
        Output = output;
        Error = error;
        ExitCode = exitCode;
    }
}