#Requires -Version 5.1
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Push-Location (Join-Path $PSScriptRoot '../..')
try {
    foreach ($tool in @('cargo', 'rustup', 'npx')) {
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
    & npx --yes @tailwindcss/cli@4.1.18 -i crates/xrat-web/assets/app.css -o crates/xrat-app/web-assets/app.css --minify
    if ($LASTEXITCODE -ne 0) { throw 'Failed to build browser styles.' }
    Copy-Item crates/xrat-web/index.html crates/xrat-app/web-assets/
    foreach ($asset in @('boot.js', 'components.js', 'xrat-logo.png', 'favicon.ico', 'manifest.webmanifest', 'pwa.js', 'sw.js', 'offline.html', 'controllers', 'icons')) {
        Copy-Item "crates/xrat-web/assets/$asset" crates/xrat-app/web-assets/ -Recurse -Force
    }
}
finally { Pop-Location }
