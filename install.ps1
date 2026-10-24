#Requires -Version 5.1

<#
.SYNOPSIS
    Installs the xrat binary on Windows from published GitHub releases.

.DESCRIPTION
    Mirrors install.sh for Windows: resolves the release tag, picks the artifact
    matching the current CPU architecture, verifies its SHA-256 checksum against
    the release SHASUMS256.txt, installs xrat.exe into the user-local bin
    directory, adds that directory to the user PATH, and hands off to
    'xrat setup'.

.PARAMETER FromSource
    Build from this checkout, including browser assets (requires Rust and MSVC tools).

.PARAMETER Version
    Release tag to install (for example v0.31.2). Defaults to the latest release.

.PARAMETER InstallDir
    Directory that receives xrat.exe (default: %USERPROFILE%\.local\bin).

.PARAMETER Yes
    Accept setup defaults, equivalent to 'xrat setup -y'.

.PARAMETER NoSetup
    Install the binary without running 'xrat setup'.

.PARAMETER Help
    Show usage information and exit.
#>
[CmdletBinding()]
param(
    [switch]$FromSource,
    [string]$Version,
    [string]$InstallDir = (Join-Path $env:USERPROFILE '.local\bin'),
    [switch]$Yes,
    [switch]$NoSetup,
    [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Repo = 'mhyrzt/xrat'
$BinaryName = 'xrat.exe'

function Write-Info {
    param([string]$Message)
    Write-Host "[+] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[!] $Message" -ForegroundColor Yellow
}

function Write-Failure {
    param([string]$Message)
    Write-Host "[x] $Message" -ForegroundColor Red
}

function Write-Step {
    param([string]$Message)
    Write-Host "[->] $Message" -ForegroundColor Blue
}

function Show-Usage {
    $usage = @"
Usage: install.ps1 [OPTIONS]

This installer downloads/verifies/places the xrat binary, then hands off to
'xrat setup' for post-install setup (init, daemon, completions and optional
user-local proxy-core installation).

Options:
  -FromSource         Build from this checkout instead of downloading a release
  -Version TAG        Release tag to install (default: latest published release)
  -InstallDir DIR     Binary install directory (default: %USERPROFILE%\.local\bin)
  -Yes                Accept setup defaults (runs 'xrat setup -y')
  -NoSetup            Install the binary only, do not run 'xrat setup'
  -Help               Show this help
"@
    Write-Host $usage -ForegroundColor Cyan
}

function Get-WindowsTarget {
    # PROCESSOR_ARCHITEW6432 carries the real CPU architecture when a 32-bit
    # PowerShell runs on 64-bit Windows (WOW64); PROCESSOR_ARCHITECTURE then
    # reports x86 and would wrongly select a 32-bit target.
    $architecture = $env:PROCESSOR_ARCHITEW6432
    if ([string]::IsNullOrWhiteSpace($architecture)) {
        $architecture = $env:PROCESSOR_ARCHITECTURE
    }
    if ([string]::IsNullOrWhiteSpace($architecture)) {
        throw 'Could not determine the Windows architecture: PROCESSOR_ARCHITECTURE is not set.'
    }

    switch ($architecture) {
        'AMD64' { return 'x86_64-pc-windows-msvc' }
        'ARM64' { return 'aarch64-pc-windows-msvc' }
        'x86' { throw 'XRAT ships 64-bit Windows binaries only. 32-bit Windows (i686) is not supported.' }
        default { throw "Unsupported Windows architecture: '$architecture'. XRAT supports x64 and ARM64." }
    }
}

function Get-LatestReleaseTag {
    $uri = "https://api.github.com/repos/$Repo/releases/latest"
    $release = Invoke-RestMethod -Uri $uri -Headers @{ 'User-Agent' = 'xrat-install' }
    $tag = $release.PSObject.Properties['tag_name']
    if ($null -eq $tag -or [string]::IsNullOrWhiteSpace($tag.Value)) {
        throw 'Could not determine the latest xrat release version.'
    }
    return [string]$tag.Value
}

function Get-ExpectedChecksum {
    param([string]$ChecksumFile, [string]$FileName)

    $entries = @()
    foreach ($line in Get-Content -LiteralPath $ChecksumFile) {
        if ($line -match '^\s*([0-9a-fA-F]{64})\s+\*?(\S+)\s*$' -and $Matches[2] -eq $FileName) {
            $entries += $Matches[1]
        }
    }
    if ($entries.Count -eq 1) { return $entries[0] }
    throw "Expected one checksum entry for $FileName; found $($entries.Count)"
}

function Add-UserPathEntry {
    param([string]$Directory)

    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $normalized = $Directory.TrimEnd('\')
    if (-not [string]::IsNullOrWhiteSpace($userPath)) {
        foreach ($entry in $userPath.Split(';')) {
            if (-not [string]::IsNullOrWhiteSpace($entry) -and $entry.Trim().TrimEnd('\') -ieq $normalized) {
                return $false
            }
        }
    }

    $newPath = if ([string]::IsNullOrWhiteSpace($userPath)) { $normalized } else { $userPath.TrimEnd(';') + ';' + $normalized }
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    return $true
}

function Expand-VerifiedExecutable {
    param([string]$Archive, [string]$Directory, [string]$Name)
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [System.IO.Compression.ZipFile]::OpenRead($Archive)
    try {
        $entries = @($zip.Entries | Where-Object { $_.FullName -ceq $Name -or $_.FullName -ceq "./$Name" })
        if ($entries.Count -ne 1) { throw "Archive must contain exactly one $Name at its root." }
        $entry = $entries[0]
        if ($entry.Length -le 0 -or $entry.Length -gt 268435456) { throw 'Invalid executable size.' }
        New-Item -ItemType Directory -Path $Directory -Force | Out-Null
        $path = Join-Path $Directory $Name
        [System.IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $path)
        return $path
    }
    finally { $zip.Dispose() }
}

function Install-VerifiedBinary {
    param([string]$Source, [string]$Destination)
    $directory = Split-Path -Parent $Destination
    $staged = Join-Path $directory ('.xrat-install-' + [guid]::NewGuid().ToString('N'))
    $backup = "$staged.backup"
    try {
        Copy-Item -LiteralPath $Source -Destination $staged
        if (Test-Path -LiteralPath $Destination) {
            [System.IO.File]::Replace($staged, $Destination, $backup)
        }
        else { [System.IO.File]::Move($staged, $Destination) }
    }
    finally {
        if (Test-Path -LiteralPath $staged) { Remove-Item -LiteralPath $staged -Force }
        if (Test-Path -LiteralPath $backup) { Remove-Item -LiteralPath $backup -Force }
    }
}

function Build-FromSource {
    param([string]$Target)

    $repoDir = $PSScriptRoot
    if (-not $repoDir -or -not (Test-Path -LiteralPath (Join-Path $repoDir 'Cargo.toml'))) {
        throw '-FromSource requires running install.ps1 directly from the repository checkout.'
    }
    foreach ($command in @('cargo', 'rustup')) {
        if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
            throw "Required tool not found: $command"
        }
    }

    Push-Location $repoDir
    try {
        & rustup target add wasm32-unknown-unknown $Target | Out-Host
        if ($LASTEXITCODE -ne 0) { throw 'Failed to install Rust targets.' }
        & ./packaging/windows/build-web.ps1
        if ($LASTEXITCODE -ne 0) { throw 'Failed to build browser assets.' }
        & cargo build --release --locked --target $Target | Out-Host
        if ($LASTEXITCODE -ne 0) { throw 'Failed to build xrat.' }
        return Join-Path $repoDir "target/$Target/release/xrat.exe"
    }
    finally {
        Pop-Location
    }
}

function Invoke-Install {
    if ($env:OS -ne 'Windows_NT') {
        throw 'This installer supports Windows only.'
    }

    if ($FromSource -and $Version) {
        throw '-FromSource and -Version cannot be used together.'
    }

    Write-Info 'Installing xrat - proxy config manager for Xray and sing-box'
    Write-Host ''

    $target = Get-WindowsTarget
    Write-Step "Target architecture: $target"

    $installedBinary = Join-Path $InstallDir $BinaryName
    $workDir = Join-Path ([System.IO.Path]::GetTempPath()) ("xrat-install-" + [guid]::NewGuid().ToString('N'))

    try {
        New-Item -ItemType Directory -Path $workDir -Force | Out-Null
        if ($FromSource) {
            $binaryPath = Build-FromSource -Target $target
            $tag = 'from source'
        }
        else {
            $tag = $Version
            if ([string]::IsNullOrWhiteSpace($tag)) {
                Write-Step 'Sniffing out the latest release'
                $tag = Get-LatestReleaseTag
            }
            $archiveName = "xrat-${tag}-${target}.zip"
            $baseUrl = "https://github.com/$Repo/releases/download/$tag"
            $archivePath = Join-Path $workDir $archiveName

            Write-Step "Scurrying to GitHub for $archiveName"
            try {
                Invoke-WebRequest -Uri "$baseUrl/$archiveName" -OutFile $archivePath -UseBasicParsing
            }
            catch {
                throw "Failed to download $baseUrl/$archiveName : $($_.Exception.Message)"
            }

            Write-Step 'Nibbling the checksum to make sure the archive is clean'
            $sumsPath = Join-Path $workDir 'SHASUMS256.txt'
            try {
                Invoke-WebRequest -Uri "$baseUrl/SHASUMS256.txt" -OutFile $sumsPath -UseBasicParsing
            }
            catch {
                throw "Failed to download $baseUrl/SHASUMS256.txt : $($_.Exception.Message)"
            }

            $expectedHash = Get-ExpectedChecksum -ChecksumFile $sumsPath -FileName $archiveName
            $actualHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash
            if ($actualHash -ne $expectedHash) {
                throw "Checksum mismatch for $archiveName. expected: $expectedHash actual: $actualHash"
            }

            Write-Step 'Unpacking the tunnel gear'
            $extractDir = Join-Path $workDir 'extracted'
            $binaryPath = Expand-VerifiedExecutable -Archive $archivePath -Directory $extractDir -Name $BinaryName
        }

        if (-not (Test-Path -LiteralPath $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        Install-VerifiedBinary -Source $binaryPath -Destination $installedBinary
        Write-Info "Stashed xrat at $installedBinary"

        if (Add-UserPathEntry -Directory $InstallDir) {
            Write-Info "Added $InstallDir to your user PATH. Restart your terminal to pick it up."
        }
        else {
            Write-Info "$InstallDir is already on your user PATH."
        }
        Write-Info "xrat $tag scampered into place successfully"
    }
    finally {
        if (Test-Path -LiteralPath $workDir) {
            Remove-Item -LiteralPath $workDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }

    if ($NoSetup) {
        Write-Info 'Skipping setup (-NoSetup).'
        return 0
    }

    $setupArgs = @('setup')
    if ($Yes) {
        $setupArgs += '-y'
    }
    elseif ([Console]::IsInputRedirected) {
        Write-Warn 'No interactive terminal is available; skipping guided setup.'
        Write-Warn "Run '$installedBinary setup' from a terminal, or reinstall with -Yes."
        return 0
    }

    Write-Step 'Handing off to xrat setup'
    Write-Host ''
    & $installedBinary @setupArgs | Out-Host
    return $LASTEXITCODE
}

if ($Help) {
    Show-Usage
    exit 0
}

# Dot-sourcing the script only defines the helpers so they can be exercised
# without performing an install.
if ($MyInvocation.InvocationName -ne '.') {
    try {
        $exitCode = Invoke-Install
        exit $exitCode
    }
    catch {
        Write-Failure $_.Exception.Message
        exit 1
    }
}
