#Requires -Version 5.1
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Target,
    [Parameter(Mandatory = $true)][string]$Archive
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$principal = New-Object Security.Principal.WindowsPrincipal ([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'This isolated acceptance fixture requires an administrator to create its temporary standard account.'
}
$archivePath = (Resolve-Path -LiteralPath $Archive).Path
$artifacts = @(& cargo test --locked -p xrat --test windows_packaging --target $Target --no-run --message-format=json)
if ($LASTEXITCODE -ne 0) { throw 'Could not compile packaged acceptance tests.' }
$test = $null
foreach ($line in $artifacts) {
    $event = $line | ConvertFrom-Json
    if ($event.reason -eq 'compiler-artifact' -and $event.target.name -eq 'windows_packaging' -and $event.executable) {
        $test = $event.executable
    }
}
if (-not $test) { throw 'Cargo did not report the packaged acceptance executable.' }

$receipt = Join-Path (Get-Location) 'target/windows-user-acceptance'
$null = New-Item -ItemType Directory -Path $receipt -Force
$name = 'xratci' + [Guid]::NewGuid().ToString('N').Substring(0, 12)
$password = 'XRAT-ci!' + [Guid]::NewGuid().ToString('N') + 'a9'
$account = "$env:COMPUTERNAME\$name"
$work = Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) $name
$created = $false
$process = $null
$result = 1
$previous = @{}
try {
    $creation = & "$env:SystemRoot/System32/net.exe" user $name $password /add /passwordchg:no /y 2>&1
    if ($LASTEXITCODE -ne 0) {
        $detail = ($creation -join [Environment]::NewLine).Replace($password, '[redacted]')
        throw "Could not create the isolated standard test account: $detail"
    }
    $created = $true
    $sid = (New-Object Security.Principal.NTAccount $account).Translate([Security.Principal.SecurityIdentifier])
    $null = New-Item -ItemType Directory -Path $work
    $acl = New-Object Security.AccessControl.DirectorySecurity
    $acl.SetAccessRuleProtection($true, $false)
    $acl.SetOwner((New-Object Security.Principal.SecurityIdentifier 'S-1-5-32-544'))
    foreach ($trustee in @('S-1-5-18', 'S-1-5-32-544', $sid.Value)) {
        $rule = New-Object Security.AccessControl.FileSystemAccessRule (
            (New-Object Security.Principal.SecurityIdentifier $trustee),
            'FullControl', 'ContainerInherit,ObjectInherit', 'None', 'Allow'
        )
        $null = $acl.AddAccessRule($rule)
    }
    Set-Acl -LiteralPath $work -AclObject $acl
    Copy-Item -LiteralPath $test -Destination (Join-Path $work 'acceptance.exe')
    Copy-Item -LiteralPath $archivePath -Destination (Join-Path $work 'package.zip')
    $temporary = Join-Path $work 'temp'
    $null = New-Item -ItemType Directory -Path $temporary
    foreach ($key in @('TEMP', 'TMP', 'XRAT_WINDOWS_ARCHIVE')) {
        $previous[$key] = [Environment]::GetEnvironmentVariable($key, 'Process')
    }
    $env:TEMP = $temporary
    $env:TMP = $temporary
    $env:XRAT_WINDOWS_ARCHIVE = Join-Path $work 'package.zip'
    $credential = New-Object Management.Automation.PSCredential (
        $account, (ConvertTo-SecureString $password -AsPlainText -Force)
    )
    $process = Start-Process -FilePath (Join-Path $work 'acceptance.exe') `
        -ArgumentList @('--ignored', '--test-threads=1', '--nocapture') `
        -Credential $credential -LoadUserProfile -WorkingDirectory $work `
        -RedirectStandardOutput (Join-Path $work 'stdout.log') `
        -RedirectStandardError (Join-Path $work 'stderr.log') -PassThru
    if (-not $process.WaitForExit(600000)) { throw 'Ordinary-user acceptance exceeded ten minutes.' }
    $process.WaitForExit()
    $result = $process.ExitCode
    @{
        account_sid = $sid.Value
        target = $Target
        exit_code = $result
        archive_sha256 = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $receipt 'receipt.json') -Encoding UTF8
}
finally {
    foreach ($key in $previous.Keys) { [Environment]::SetEnvironmentVariable($key, $previous[$key], 'Process') }
    if ($null -ne $process -and -not $process.HasExited) { Stop-Process -Id $process.Id -Force }
    if ($created) {
        Get-Process -IncludeUserName | Where-Object { $_.UserName -ieq $account } | Stop-Process -Force -ErrorAction SilentlyContinue
    }
    foreach ($stream in @('stdout.log', 'stderr.log')) {
        $path = Join-Path $work $stream
        if (Test-Path -LiteralPath $path) {
            Copy-Item -LiteralPath $path -Destination (Join-Path $receipt $stream) -Force
            Get-Content -LiteralPath $path
        }
    }
    if ($created) {
        $null = & "$env:SystemRoot/System32/net.exe" user $name /delete 2>&1
        if ($LASTEXITCODE -ne 0) { throw 'Could not delete the temporary acceptance account.' }
    }
    if (Test-Path -LiteralPath $work) { Remove-Item -LiteralPath $work -Recurse -Force }
}
if ($result -ne 0) { throw "Ordinary-user packaged acceptance failed ($result)." }
