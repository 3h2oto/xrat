#Requires -Version 5.1
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Executable,
    [Parameter(Mandatory = $true)][string]$ConfigPath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Join-Path ([IO.Path]::GetTempPath()) ('xrat upgrade ' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root | Out-Null
try {
    $version = (& $Executable --version | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Could not inspect packaged executable version.' }
    foreach ($tamper in @($false, $true)) {
        $install = Join-Path $root ([guid]::NewGuid().ToString('N'))
        $transaction = Join-Path $install ('.xrat-upgrade-' + [guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $transaction | Out-Null
        $target = Join-Path $install 'xrat.exe'
        $next = Join-Path $transaction 'next.exe'
        $helper = Join-Path $transaction 'updater.exe'
        foreach ($destination in @($target, $next, $helper)) { Copy-Item -LiteralPath $Executable -Destination $destination }
        $originalHash = (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash
        $manifest = Join-Path $transaction 'transaction.json'
        $record = @{
            target = $target; config = $ConfigPath; parent_pid = $PID; parent_creation = 0
            digest = (Get-FileHash -LiteralPath $next -Algorithm SHA256).Hash.ToLowerInvariant()
            version = $version.Substring(5); state = 'staged'; message = 'native acceptance fixture'
            restart_daemon = $false
        }
        [IO.File]::WriteAllText($manifest, ($record | ConvertTo-Json), [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText((Join-Path $install 'xrat-upgrade.json'), (ConvertTo-Json $manifest), [Text.UTF8Encoding]::new($false))
        if ($tamper) { [IO.File]::WriteAllText($next, 'tampered executable') }

        $start = [Diagnostics.ProcessStartInfo]::new()
        $start.FileName = $helper
        $start.Arguments = 'upgrade --apply-transaction "' + $manifest + '"'
        $start.UseShellExecute = $false
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        $process = [Diagnostics.Process]::Start($start)
        try {
            $stdout = $process.StandardOutput.ReadToEndAsync()
            $stderr = $process.StandardError.ReadToEndAsync()
            if (-not $process.WaitForExit(30000)) { $process.Kill(); throw 'Upgrade helper did not finish within 30 seconds.' }
            $exitCode = $process.ExitCode
            $stdout.Wait()
            $stderr.Wait()
        }
        finally { $process.Dispose() }

        $result = Get-Content -LiteralPath $manifest -Raw | ConvertFrom-Json
        if ($tamper) {
            if ($exitCode -eq 0 -or $result.state -ne 'failed') { throw 'Tampered upgrade was not rejected.' }
            if ((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash -ne $originalHash) { throw 'Failed upgrade modified the installed binary.' }
        }
        else {
            if ($exitCode -ne 0 -or $result.state -ne 'completed') { throw "Native upgrade failed: $($result.message)" }
            $installedVersion = (& $target --version | Out-String).Trim()
            if ($LASTEXITCODE -ne 0 -or $installedVersion -ne $version) { throw 'Replacement executable did not report the expected version.' }
            $status = & $target upgrade --status | Out-String | ConvertFrom-Json
            if ($LASTEXITCODE -ne 0 -or $status.state -ne 'completed') { throw 'Public upgrade status did not report completion.' }
            if (Test-Path -LiteralPath (Join-Path $transaction 'backup.exe')) { throw 'Successful upgrade retained a binary backup.' }
            if (Test-Path -LiteralPath $helper) { throw 'Completed updater helper was not cleaned up.' }
        }
    }
    Write-Host 'Native staged replacement, migrations, checksum failure, and upgrade status checks passed.'
}
finally { Remove-Item -LiteralPath $root -Recurse -Force -ErrorAction SilentlyContinue }
