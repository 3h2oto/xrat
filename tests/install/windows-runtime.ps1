#Requires -Version 5.1
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$Archive)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$directory = Join-Path ([IO.Path]::GetTempPath()) ('xrat runtime ' + [guid]::NewGuid().ToString('N'))
$previousRoot = $env:XRAT_PATH
$exe = $null

function Invoke-Xrat {
    param([string[]]$Arguments)
    $output = & $exe @Arguments | Out-String
    if ($LASTEXITCODE -ne 0) { throw "XRAT $($Arguments[0]) failed with exit code $LASTEXITCODE" }
    return $output
}

try {
    New-Item -ItemType Directory -Path $directory | Out-Null
    Expand-Archive -LiteralPath $Archive -DestinationPath (Join-Path $directory 'bin')
    $exe = Join-Path $directory 'bin/xrat.exe'
    $env:XRAT_PATH = Join-Path $directory 'profile with spaces'
    Invoke-Xrat -Arguments @('init') | Out-Null
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    $listener.Start()
    $port = $listener.LocalEndpoint.Port
    $listener.Stop()
    $password = 'XRAT-CI-' + [guid]::NewGuid().ToString('N')
    $hash = ($password | & $exe hash-password | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $hash.StartsWith('$argon2id$')) { throw 'Could not prepare isolated WebUI credentials.' }
    $configuration = @"
[server]
enabled = true
host = "127.0.0.1"
port = $port
[server.web]
enabled = true
admin_username = "ci-admin"
admin_password_hash = '$hash'
public_url = "http://localhost:$port"
"@
    [IO.File]::WriteAllText((Join-Path $env:XRAT_PATH 'config.toml'), $configuration, [Text.UTF8Encoding]::new($false))
    Invoke-Xrat -Arguments @('daemon', 'start') | Out-Null
    Invoke-Xrat -Arguments @('daemon', 'status') | Out-Null
    $base = "http://localhost:$port"
    $ready = $false
    for ($attempt = 0; $attempt -lt 50; $attempt++) {
        try {
            $html = Invoke-WebRequest "$base/" -UseBasicParsing -TimeoutSec 2
            if ($html.StatusCode -eq 200) { $ready = $true; break }
        }
        catch { Start-Sleep -Milliseconds 100 }
    }
    if (-not $ready -or $html.Content -notmatch 'id="main"') { throw 'Packaged WebUI did not serve its HTML.' }
    foreach ($asset in @('boot.js', 'xrat_web.js', 'xrat_web_bg.wasm', 'app.css')) {
        $response = Invoke-WebRequest "$base/$asset" -UseBasicParsing -TimeoutSec 20
        if ($response.StatusCode -ne 200 -or $response.RawContentLength -lt 8) { throw "Missing packaged WebUI asset: $asset" }
    }
    $session = [Microsoft.PowerShell.Commands.WebRequestSession]::new()
    $login = Invoke-RestMethod "$base/api/v1/login" -Method Post -WebSession $session -Headers @{ Origin = $base } -ContentType 'application/json' -Body (@{username = 'ci-admin'; password = $password} | ConvertTo-Json)
    $me = Invoke-RestMethod "$base/api/v1/me" -WebSession $session
    if ($me.user.username -ne 'ci-admin') { throw 'Packaged WebUI login did not establish the expected identity.' }
    Invoke-Xrat -Arguments @('daemon', 'restart') | Out-Null
    Invoke-Xrat -Arguments @('daemon', 'status') | Out-Null
    Invoke-Xrat -Arguments @('daemon', 'stop') | Out-Null
    Invoke-Xrat -Arguments @('daemon', 'stop') | Out-Null
    Write-Host 'Packaged Windows daemon, HTTP assets, and login acceptance passed.'
}
finally {
    if ($exe -and (Test-Path -LiteralPath $exe)) {
        & $exe daemon stop 2>$null | Out-Null
    }
    $env:XRAT_PATH = $previousRoot
    Remove-Item -LiteralPath $directory -Recurse -Force -ErrorAction SilentlyContinue
}
