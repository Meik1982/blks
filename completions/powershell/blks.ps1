
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'blks' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'blks'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'blks' {
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Read checksums from the specified file and check them')
            [CompletionResult]::new('--check', '--check', [CompletionResultType]::ParameterName, 'Read checksums from the specified file and check them')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Digest output format')
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'Number of worker threads for parallel tree reduction (default: all logical cores)')
            [CompletionResult]::new('--threads', '--threads', [CompletionResultType]::ParameterName, 'Number of worker threads for parallel tree reduction (default: all logical cores)')
            [CompletionResult]::new('--completion', '--completion', [CompletionResultType]::ParameterName, 'Generate shell auto-completion script (bash, zsh, fish, powershell, elvish)')
            [CompletionResult]::new('--hex', '--hex', [CompletionResultType]::ParameterName, 'Convenience flag for hexadecimal output (equivalent to --format hex)')
            [CompletionResult]::new('--raw', '--raw', [CompletionResultType]::ParameterName, 'Convenience flag for raw binary output to stdout')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Quiet mode: in check mode, only print failed files')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Quiet mode: in check mode, only print failed files')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output machine-readable JSON telemetry')
            [CompletionResult]::new('--benchmark', '--benchmark', [CompletionResultType]::ParameterName, 'Benchmark mode: print elapsed time and throughput in GB/s')
            [CompletionResult]::new('--no-mmap', '--no-mmap', [CompletionResultType]::ParameterName, 'Disable memory-mapping (forces streaming reads; prevents SIGBUS on network shares or volatile files)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
