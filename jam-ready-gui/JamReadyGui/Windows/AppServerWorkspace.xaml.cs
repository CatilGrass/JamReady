using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Documents;
using System.Windows.Media;
using JamReadyGui.Data;

namespace JamReadyGui.Windows;

public partial class AppServerWorkspace
{
    private const string ProcessName = "jam";
    private const string ProcessFilename = "jam.exe";
    
    // 当前进程
    private Process? _process;
    
    // 是否正在运行
    private bool _isRunning;
    
    private CancellationTokenSource? _cancellationTokenSource;
    private readonly object _processLock = new();
    private readonly StringBuilder _outputBuffer = new();
    private readonly Brush[] _ansiColors = {
        Brushes.Black,
        Brushes.Red, 
        Brushes.Green,
        Brushes.Yellow, 
        Brushes.Blue,
        Brushes.Magenta,
        Brushes.Cyan,
        Brushes.White 
    };
    
    private static string _currentSelectedMember = "";
    private readonly string _workingDirectory;

    private static readonly Dictionary<CheckBox, String> DutiesCheckBox = new();
    
    // 关闭等待任务
    private TaskCompletionSource<bool> _shutdownTask = new();
    
    public AppServerWorkspace(string workingDirectory)
    {
        _workingDirectory = workingDirectory;
        
        InitializeComponent();
        
        DutiesCheckBox.Add(MemberSelectedDebugger, "Debugger");
        DutiesCheckBox.Add(MemberSelectedLeader, "Leader");
        DutiesCheckBox.Add(MemberSelectedDeveloper, "Developer");
        DutiesCheckBox.Add(MemberSelectedCreator, "Creator");
        DutiesCheckBox.Add(MemberSelectedProducer, "Producer");
        
        // 显示工作区名称
        var workspaceName = AppCoreInvoker.Execute(new[] { "query", "workspace" })?.Output ?? "Unknown";
        Title = $"Server Workspace ({workspaceName})";
        
        // 退出当前工作区
        ExitWorkspaceButton.Click += (_, _) =>
        {
            AppPreference.WorkspacePreference.ExitWorkspace();
            Close();
        };
        
        // 成员列表操作
        MemberList.SelectionChanged += (_, _) =>
        {
            if (MemberList.SelectedItems.Count > 0)
            {
                var selectedMember = MemberList.SelectedItems[0]?.ToString();
                if (selectedMember == null) return;
                RefreshSelectedMember(selectedMember);
            }
        };
        
        // 点击复制登录代码
        CopyMemberLoginCodeButton.Click += (_, _) =>
        {
            var loginCode = AppCoreInvoker.Execute(new[] { "query", "login-code", _currentSelectedMember })?.Output ?? "";
            Clipboard.SetText(loginCode);
        };
        
        // 保存成员
        SaveMemberButton.Click += (_, _) =>
        {
            // 职责
            var memberDuties = "";
            foreach (var p in DutiesCheckBox)
            {
                if (p.Key.IsChecked != null && p.Key.IsChecked.Value) 
                    memberDuties += $"{p.Value}, ";
            }
            AppCoreInvoker.Execute(new[] { "set", "member", "duties", _currentSelectedMember, $"\"{memberDuties}\"" });

            // 成员名称
            var memberNewName = MemberNameInput.Text.Trim();
            var members = AppCoreInvoker.Execute(new[] { "list", "member" })?.Output ?? "";
            var containsNewName = members.Contains(memberNewName);
            var nameChanged = _currentSelectedMember.Trim() != memberNewName;
            if (nameChanged && !containsNewName)
            {
                AppCoreInvoker.Execute(new[] { "set", "member", "name", _currentSelectedMember, memberNewName });
                RefreshSelectedMember(memberNewName);
            }

            RefreshMemberList();
        };
        
        // 添加成员
        AddMemberButton.Click += (_, _) =>
        {
            AppCoreInvoker.Execute(new[] { "add", "member", "member" });
            RefreshMemberList();
            RefreshSelectedMember("member");
        };
        
        // 添加成员
        RemoveMemberButton.Click += (_, _) =>
        {
            AppCoreInvoker.Execute(new[] { "remove", "member", _currentSelectedMember });
            RefreshMemberList();
            RefreshSelectedMember("");
        };
        
        // 复制本地 IP
        CopyAddressButton.Click += (_, _) =>
        {
            var loginCode = AppCoreInvoker.Execute(new[] { "query", "local-address", _currentSelectedMember })?.Output ?? "127.0.0.1:5011";
            Clipboard.SetText(loginCode);
        };
        
        // 运行服务器
        RunOrCloseServerButton.Click += (_, _) =>
        {
            Switch();
        };
        
        // 窗口关闭时尝试关闭 jam.exe
        Closing += async (s, e) => {
            e.Cancel = true; // 取消关闭事件
            await ForceShutdownAsync();
            Application.Current.Shutdown();
        };
            
        // 检查并终止已存在的 jam.exe
        KillExistingJamReadyProcess();
        
        // 刷新
        RefreshMemberList();
        RefreshSelectedMember("");
    }

    // 刷新成员管理器
    private void RefreshMemberList()
    {
        MemberList.Items.Clear();
        var members = AppCoreInvoker.Execute(new[] { "list", "member" })?.Output ?? "";
        foreach (var member in members.Split(","))
        {
            if (member != "")
            {
                MemberList.Items.Add(member.Trim());
            }
        }
    }

    // 刷新选择的成员
    private void RefreshSelectedMember(string selectedMember)
    {
        _currentSelectedMember = selectedMember;
        
        var uuid = AppCoreInvoker.Execute(new[] { "query", "uuid", _currentSelectedMember })?.Output ?? "unknown_member";
        var loginCode = AppCoreInvoker.Execute(new[] { "query", "login-code", _currentSelectedMember })?.Output ?? "XXXX-XXXX";
        var duties = AppCoreInvoker.Execute(new[] { "query", "duty", _currentSelectedMember })?.Output ?? "unknown_member";

        MemberNameInput.Text = _currentSelectedMember;

        foreach (var p in DutiesCheckBox)
        {
            p.Key.IsChecked = duties.Contains(p.Value);
        }
        
        MemberLoginCode.Content = loginCode;
        MemberUuid.Content = uuid;
    }

    // 切换进程状态
    private async void Switch()
    {
        if (! _isRunning)
            await StartProcessAsync();
        else
            await StopProcessAsync();
    }
    
    // 杀死正在运行的 jam.exe
    private void KillExistingJamReadyProcess()
    {
        try
        {
            var existing = Process.GetProcessesByName(ProcessName);
            if (existing.Length == 0) return;
                
            AppendLogLine($"Found {existing.Length} jam.exe processes running, attempting to kill.", Brushes.DarkMagenta);
                
            foreach (var p in existing)
            {
                try
                {
                    if (!p.HasExited)
                    {
                        p.Kill();
                        AppendLogLine($"Terminated process [ID: {p.Id}]", Brushes.DarkMagenta);
                    }
                }
                catch (Exception ex)
                {
                    AppendLogLine($"Failed to terminate process [ID: {p.Id}]: {ex.Message}", Brushes.Red);
                }
            }
        }
        catch (Exception ex)
        {
            AppendLogLine($"Process check failed: {ex.Message}", Brushes.Red);
        }
    }
        
    private Task StartProcessAsync()
    {
        lock (_processLock)
        {
                if (_isRunning) return Task.CompletedTask;
                
                // 重置取消令牌
                _cancellationTokenSource?.Dispose();
                _cancellationTokenSource = new CancellationTokenSource();
        }

        try
        {
            CommandOutput.Document.Blocks.Clear();
            AppendLogLine($"Starting process: {ProcessFilename}", Brushes.DarkGreen);
                
            var startInfo = new ProcessStartInfo
            {
                FileName = AppPreference.JamReadyExeFile,
                Arguments = "run --short-logger",
                WorkingDirectory = _workingDirectory,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                RedirectStandardInput = true,
                CreateNoWindow = true,
                StandardOutputEncoding = Encoding.UTF8,
                StandardErrorEncoding = Encoding.UTF8
            };

            var process = new Process
            {
                StartInfo = startInfo,
                EnableRaisingEvents = true
            };

            process.Exited += (_, _) => {
                lock (_processLock)
                {
                    _isRunning = false;
                    _shutdownTask?.TrySetResult(true);
                }
                Dispatcher.Invoke(() => {
                    AppendLogLine($"Process exited with code: {process.ExitCode}", Brushes.Gray);
                });
            };

            if (!process.Start())
            {
                AppendLogLine("Failed to start: Unable to launch process", Brushes.Red);
                return Task.CompletedTask;
            }

            lock (_processLock)
            {
                _process = process;
                _isRunning = true;
            }

            AppendLogLine($"Process started [ID: {process.Id}]", Brushes.Green);
                
            // 开始异步读取输出
            _ = Task.Run(() => ReadStreamAsync(_process.StandardOutput));
            _ = Task.Run(() => ReadStreamAsync(_process.StandardError));
        }
        catch (Exception ex)
        {
            AppendLogLine($"Failed to start: {ex.Message}", Brushes.Red);
        }

        return Task.CompletedTask;
    }

    // force: true表示强制终止，false表示尝试优雅关闭
    private async Task StopProcessAsync(bool force = false)
    {
        lock (_processLock)
        {
            if (!_isRunning || _process == null) return;
                
            // 重置关闭等待任务
            _shutdownTask.TrySetCanceled();
            _shutdownTask = new TaskCompletionSource<bool>();
        }

        try
        {
            // 取消所有正在进行的读取操作
            _cancellationTokenSource?.Cancel();
                
            if (force)
            {
                AppendLogLine($"Forcibly terminating process [ID: {_process.Id}]", Brushes.DarkOrange);
                try
                {
                    if (!_process.HasExited)
                    {
                        _process.Kill();
                    }
                }
                catch (Exception ex) when (ex is InvalidOperationException || ex is Win32Exception)
                {
                    // 进程已终止或被其他程序关闭
                }
            }

            // 等待进程实际退出
            if (!_process.HasExited)
            {
                AppendLogLine("Waiting for process to exit...", Brushes.White);
                await _shutdownTask.Task;
            }
        }
        catch (TaskCanceledException)
        {
            // 任务已取消，正常情况
        }
        finally
        {
            // 清理资源
            lock (_processLock)
            {
                if (_process != null)
                {
                    try
                    {
                        if (!_process.HasExited)
                        {
                            _process.Kill();
                            AppendLogLine($"Process terminated [ID: {_process.Id}]", Brushes.DarkOrange);
                        }
                    }
                    catch (Exception ex)
                    {
                        Debug.WriteLine($"Failed to terminate process: {ex.Message}");
                    }
                        
                    _process.Dispose();
                    _process = null;
                    _isRunning = false;
                }
                    
                // 释放取消令牌
                _cancellationTokenSource?.Dispose();
                _cancellationTokenSource = null;
            }
        }
    }

    private async Task ForceShutdownAsync()
    {
        if (_isRunning)
        {
            await StopProcessAsync(true);
        }
    }

    private async Task ReadStreamAsync(StreamReader reader)
    {
        char[] buffer = new char[4096];
        var cancellationToken = _cancellationTokenSource?.Token ?? CancellationToken.None;
            
        try
        {
            while (!cancellationToken.IsCancellationRequested)
            {
                try
                {
                    int bytesRead = await reader.ReadAsync(buffer, 0, buffer.Length);
                    if (bytesRead == 0) break;

                    string content = new string(buffer, 0, bytesRead);
                    Dispatcher.Invoke(() => ProcessOutput(content));
                }
                catch (Exception ex) when (ex is ObjectDisposedException || 
                                           ex is InvalidOperationException ||
                                           ex is OperationCanceledException)
                {
                    break;
                }
            }
        }
        catch (Exception ex)
        {
            Dispatcher.Invoke(() => {
                AppendLogLine($"Error: {ex.Message}", Brushes.Red);
            });
        }
    }

    private void ProcessOutput(string content)
    {
        _outputBuffer.Append(content);
        string buffer = _outputBuffer.ToString();

        // 处理ANSI颜色代码
        if (buffer.Contains("\u001b[") || buffer.Contains("\n"))
        {
            ProcessAnsiCodes(buffer);
            _outputBuffer.Clear();
        }
    }

    private void ProcessAnsiCodes(string text)
    {
        var paragraph = new Paragraph();
        int pos = 0;
        Brush currentColor = Brushes.Black; // 默认颜色

        while (pos < text.Length)
        {
            // 查找ANSI转义序列
            int ansiStart = text.IndexOf("\u001b[", pos, StringComparison.Ordinal);
            if (ansiStart == -1)
            {
                // 没有更多ANSI代码，添加剩余文本
                paragraph.Inlines.Add(new Run(text.Substring(pos)));
                break;
            }

            // 添加ANSI代码前的文本
            if (ansiStart > pos)
            {
                paragraph.Inlines.Add(new Run(text.Substring(pos, ansiStart - pos)) 
                    { Foreground = currentColor });
            }

            // 解析ANSI代码
            int ansiEnd = text.IndexOf('m', ansiStart);
            if (ansiEnd == -1) break;

            string ansiCode = text.Substring(ansiStart + 2, ansiEnd - ansiStart - 2);
            pos = ansiEnd + 1;

            // 处理颜色代码 (简化版)
            if (ansiCode.StartsWith("3"))
            {
                int colorCode = ansiCode[1] - '0';
                if (colorCode >= 0 && colorCode <= 7)
                {
                    currentColor = _ansiColors[colorCode];
                }
            }
            // 重置样式
            else if (ansiCode == "0")
            {
                currentColor = Brushes.Black;
            }
        }
        
        CommandOutput.Document.Blocks.Add(paragraph);
        CommandOutput.ScrollToEnd();
    }

    private void AppendLogLine(string text, Brush color)
    {
        var paragraph = new Paragraph(new Run(text) { Foreground = color });
        CommandOutput.Document.Blocks.Add(paragraph);
        CommandOutput.ScrollToEnd();
    }
}