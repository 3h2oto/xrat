$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$testDir = Join-Path ([System.IO.Path]::GetTempPath()) ("xrat-installer-test-" + [guid]::NewGuid().ToString('N'))
$savedEnv = @{}
foreach ($name in @('OS', 'USERPROFILE', 'PROCESSOR_ARCHITECTURE', 'PROCESSOR_ARCHITEW6432')) {
    $savedEnv[$name] = [Environment]::GetEnvironmentVariable($name)
}

function Assert-Equal($Actual, $Expected) {
    if ($Actual -ne $Expected) { throw "Expected '$Expected', got '$Actual'" }
}

function Assert-Throws([scriptblock]$Action, [string]$Message) {
    try { & $Action }
    catch {
        if ($_.Exception.Message -notlike "*$Message*") { throw }
        return
    }
    throw "Expected an error containing '$Message'"
}

try {
    New-Item -ItemType Directory -Path $testDir | Out-Null
    $env:USERPROFILE = $testDir
    $env:OS = 'Windows_NT'
    . "$PSScriptRoot/../../install.ps1" -InstallDir (Join-Path $testDir 'bin') -NoSetup

    $env:PROCESSOR_ARCHITEW6432 = ''
    $env:PROCESSOR_ARCHITECTURE = 'AMD64'
    Assert-Equal (Get-WindowsTarget) 'x86_64-pc-windows-msvc'
    $env:PROCESSOR_ARCHITECTURE = 'ARM64'
    Assert-Equal (Get-WindowsTarget) 'aarch64-pc-windows-msvc'
    $env:PROCESSOR_ARCHITECTURE = 'x86'
    Assert-Throws { Get-WindowsTarget } '32-bit Windows'
    $env:PROCESSOR_ARCHITEW6432 = 'AMD64'
    Assert-Equal (Get-WindowsTarget) 'x86_64-pc-windows-msvc'

    $fixture = Join-Path $testDir 'xrat.exe'
    Set-Content -LiteralPath $fixture -Value 'installer fixture'
    $archive = Join-Path $testDir 'fixture.zip'
    Compress-Archive -LiteralPath $fixture -DestinationPath $archive
    $archiveName = 'xrat-v9.8.7-x86_64-pc-windows-msvc.zip'
    $hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash
    $sums = Join-Path $testDir 'SHASUMS256.txt'
    Set-Content -LiteralPath $sums -Value "$hash  $archiveName"
    Assert-Equal (Get-ExpectedChecksum $sums $archiveName) $hash
    Assert-Throws { Get-ExpectedChecksum $sums 'missing.zip' } 'checksum'
    Set-Content -LiteralPath $sums -Value @("$hash  $archiveName", "$hash  $archiveName")
    Assert-Throws { Get-ExpectedChecksum $sums $archiveName } 'found 2'
    Set-Content -LiteralPath $sums -Value "$hash  $archiveName"

    $extracted = Expand-VerifiedExecutable -Archive $archive -Directory (Join-Path $testDir 'extracted') -Name 'xrat.exe'
    Assert-Equal (Get-Content -LiteralPath $extracted) 'installer fixture'
    Assert-Throws { Expand-VerifiedExecutable -Archive $archive -Directory (Join-Path $testDir 'missing') -Name 'absent.exe' } 'exactly one'

    function Get-LatestReleaseTag { return 'v9.8.7' }
    function Add-UserPathEntry { param($Directory) return $false }
    function Invoke-WebRequest {
        param($Uri, $OutFile, [switch]$UseBasicParsing)
        if ($Uri.EndsWith('/SHASUMS256.txt')) {
            Copy-Item -LiteralPath $sums -Destination $OutFile
        }
        elseif ($Uri.EndsWith("/$archiveName")) {
            Copy-Item -LiteralPath $archive -Destination $OutFile
        }
        else { throw "Unexpected download: $Uri" }
    }

    Assert-Equal (Invoke-Install) 0
    Assert-Equal (Get-Content (Join-Path $InstallDir 'xrat.exe')) 'installer fixture'

    Set-Content -LiteralPath $sums -Value "$('0' * 64)  $archiveName"
    Set-Content -LiteralPath (Join-Path $InstallDir 'xrat.exe') -Value 'preserve existing binary'
    Assert-Throws { Invoke-Install } 'Checksum mismatch'
    Assert-Equal (Get-Content (Join-Path $InstallDir 'xrat.exe')) 'preserve existing binary'

    $FromSource = $true
    function Build-FromSource {
        param($Target)
        Assert-Equal $Target 'x86_64-pc-windows-msvc'
        return $fixture
    }
    function Invoke-WebRequest { throw 'Source installation must not download release files' }
    Assert-Equal (Invoke-Install) 0
    Assert-Equal (Get-Content (Join-Path $InstallDir 'xrat.exe')) 'installer fixture'
    $Version = 'v9.8.7'
    Assert-Throws { Invoke-Install } 'cannot be used together'
    $Version = ''

    if ([Console]::IsInputRedirected) {
        $NoSetup = $false
        Assert-Equal (Invoke-Install) 0
    }

    $BinaryName = 'setup.ps1'
    $fixture = Join-Path $testDir 'setup-source.ps1'
    Set-Content -LiteralPath $fixture -Value @'
Set-Content -LiteralPath (Join-Path $PSScriptRoot 'setup-arguments.txt') -Value ($args -join ' ')
exit 7
'@
    $Yes = $true
    $NoSetup = $false
    Assert-Equal (Invoke-Install) 7
    Assert-Equal (Get-Content (Join-Path $InstallDir 'setup-arguments.txt')) 'setup -y'
    Write-Host 'Windows installer checks passed.'
    exit 0
}
catch {
    Write-Error "Windows installer checks failed: $_"
    exit 1
}
finally {
    foreach ($name in $savedEnv.Keys) {
        [Environment]::SetEnvironmentVariable($name, $savedEnv[$name])
    }
    Remove-Item -LiteralPath $testDir -Recurse -Force -ErrorAction SilentlyContinue
}
