$AllCommands = @(
    'help','h',
    'query','q',
    'struct','tree','list','ls',
    'redirect','red',
    'update','sync',
    'commit','cmt','save','sv',
    'archive',
    'clone','c','build',
    'add','new','create',
    'remove','rm','delete','del',
    'move','mv','rename',
    'rollback', 'rb', 'restore'
    'get','g','lock',
    'throw','t','unlock','release',
    'view','v','download','dl',
    'param','set'
)

$VFCommands = @(
    'add','new','create',
    'remove','rm','delete','del',
    'get','g','lock',
    'throw','t','unlock','release',
    'view','v','download','dl',
    'move','mv','rename',
    'rollback', 'rb', 'restore',
    'query', 'q'
)

$PArgCommands = @('param','set')

function Get-JamVFCompletions {
    param(
        [string] $prefix,
        [string] $fragment
    )

    $psi = [System.Diagnostics.ProcessStartInfo]::new('jam.exe', "query list -i `"$prefix`"")
    $psi.RedirectStandardOutput    = $true
    $psi.UseShellExecute           = $false
    $psi.StandardOutputEncoding    = [System.Text.Encoding]::UTF8

    $proc   = [System.Diagnostics.Process]::Start($psi)
    $output = $proc.StandardOutput.ReadToEnd()
    $proc.WaitForExit()

    $output -split "`r?`n" | Where-Object { $_ } |
            ForEach-Object {
                if ($_ -like "$fragment*") {
                    $insertText = "$prefix$_"
                    if ($insertText -match '\s') {
                        $insertText = "`"$insertText`""
                    }

                    [System.Management.Automation.CompletionResult]::new(
                            $insertText,
                            $_,
                            'ParameterValue',
                            $_
                    )
                }
            }
}

Register-ArgumentCompleter -Native -CommandName 'jam' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $tokens     = $commandAst.CommandElements | ForEach-Object { $_.ToString().Trim("'`"") }
    $tokenCount = $tokens.Count

    if ($tokenCount -le 1) {
        $AllCommands | Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        return
    }

    $sub = $tokens[1]

    if ($tokenCount -eq 2 -and ($AllCommands -notcontains $sub)) {
        $AllCommands | Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
        return
    }

    if ($PArgCommands -contains $sub) {
        if ($tokenCount -ge 2) {
            $paramDir = Join-Path (Get-Location) '.jam\param'
            if (Test-Path $paramDir) {
                Get-ChildItem $paramDir -Filter '*.txt' -Name |
                        ForEach-Object { $_ -replace '\.txt$','' } |
                        Where-Object { $_ -like "$wordToComplete*" } |
                        ForEach-Object {
                            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                        }
            }
        }
        return
    }

    if ($VFCommands -contains $sub) {
        $argIndex = $tokenCount - 2
        if ($argIndex -ge 0) {
            if ($wordToComplete.Contains('/')) {
                $i = $wordToComplete.LastIndexOf('/')
                $prefix   = $wordToComplete.Substring(0, $i + 1)
                $fragment = $wordToComplete.Substring($i + 1)
            } else {
                $prefix   = ''
                $fragment = $wordToComplete
            }

            return Get-JamVFCompletions -prefix $prefix -fragment $fragment
        }
    }
}

