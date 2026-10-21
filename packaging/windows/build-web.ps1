#Requires -Version 5.1
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Push-Location (Join-Path $PSScriptRoot '../..')
try {
    foreach ($tool in @('cargo', 'rustup')) {
        if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) { throw "Required tool not found: $tool" }
    }
    & rustup target add wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) { throw 'Failed to install the WebAssembly target.' }
    & cargo build --locked --release -p xrat-web --target wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) { throw 'Failed to build browser WebAssembly.' }
    & cargo install wasm-bindgen-cli --locked --version 0.2.122 --root target/web-tools
    if ($LASTEXITCODE -ne 0) { throw 'Failed to install wasm-bindgen.' }
    $extension = if ($env:OS -eq 'Windows_NT') { '.exe' } else { '' }
    & "./target/web-tools/bin/wasm-bindgen$extension" target/wasm32-unknown-unknown/release/xrat-web.wasm --target web --out-name xrat_web --out-dir crates/xrat-app/web-assets
    if ($LASTEXITCODE -ne 0) { throw 'Failed to generate browser bindings.' }
    if ($env:OS -eq 'Windows_NT') {
        $asset = 'tailwindcss-windows-x64.exe'
        $checksum = '55bc5a2e294520a74fe3523eaa11915ef50047e7228a545aa181ec413cf52612'
    }
    else {
        if (-not [Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([Runtime.InteropServices.OSPlatform]::Linux) -or [Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne [Runtime.InteropServices.Architecture]::X64) {
            throw 'This build helper supports Windows and the Linux x64 CI asset builder.'
        }
        $asset = 'tailwindcss-linux-x64'
        $checksum = '737becf8d4ad1115ea98df69fa94026d402ca8feb91306a035b5b004167da8dd'
    }
    $tailwind = Join-Path (Get-Location) "target/web-tools/$asset"
    if (-not (Test-Path -LiteralPath $tailwind) -or (Get-FileHash -LiteralPath $tailwind -Algorithm SHA256).Hash -ne $checksum) {
        $download = "$tailwind.download"
        try {
            Invoke-WebRequest "https://github.com/tailwindlabs/tailwindcss/releases/download/v4.1.18/$asset" -OutFile $download -UseBasicParsing
            if ((Get-FileHash -LiteralPath $download -Algorithm SHA256).Hash -ne $checksum) { throw 'Tailwind tool checksum mismatch.' }
            Move-Item -LiteralPath $download -Destination $tailwind -Force
        }
        finally { if (Test-Path -LiteralPath $download) { Remove-Item -LiteralPath $download -Force } }
    }
    if ($env:OS -ne 'Windows_NT') {
        & chmod +x $tailwind
        if ($LASTEXITCODE -ne 0) { throw 'Could not prepare the Tailwind tool.' }
    }
    & $tailwind -i crates/xrat-web/assets/app.css -o crates/xrat-app/web-assets/app.css --minify
    if ($LASTEXITCODE -ne 0) { throw 'Failed to build browser styles.' }
    Copy-Item crates/xrat-web/index.html crates/xrat-app/web-assets/
    foreach ($asset in @('favicon.ico', 'manifest.webmanifest', 'sw.js', 'offline.html', 'icons')) {
        Copy-Item "crates/xrat-web/assets/$asset" crates/xrat-app/web-assets/ -Recurse -Force
    }
    foreach ($asset in @('controllers', 'components.js', 'pwa.js', 'boot.js', 'xrat-logo.png')) {
        $staleAsset = "crates/xrat-app/web-assets/$asset"
        if (Test-Path -LiteralPath $staleAsset) { Remove-Item -LiteralPath $staleAsset -Recurse -Force }
    }
    Copy-Item crates/xrat-web/public/* crates/xrat-app/web-assets/ -Recurse -Force
}
finally { Pop-Location }
