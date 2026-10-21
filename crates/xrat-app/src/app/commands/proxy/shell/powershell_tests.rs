use super::*;

#[test]
fn powershell_quotes_values_and_preserves_unrelated_environment() {
    let script = enable(
        "http://user'quote@127.0.0.1:8080",
        "socks5h://127.0.0.1:1080",
    );
    assert!(script.contains("'http://user''quote@127.0.0.1:8080'"));
    assert!(!script.contains("NO_PROXY"));
    assert!(!script.contains("http_proxy"));
    assert!(script.contains("-not (Test-Path Env:XRAT_PROXY_HAD_HTTP_PROXY)"));
    let restore = restore();
    assert!(restore.contains("if (Test-Path Env:XRAT_PROXY_HAD_HTTP_PROXY)"));
    assert!(!restore.contains("NO_PROXY"));
}

#[test]
#[cfg(windows)]
fn native_powershell_repeated_enable_restores_original_environment() {
    let script = format!(
        r#"$ErrorActionPreference = 'Stop'
$env:HTTP_PROXY = 'http://original.example:8080'
Remove-Item Env:HTTPS_PROXY -ErrorAction SilentlyContinue
$env:ALL_PROXY = 'socks5://original.example:1080'
$env:NO_PROXY = 'localhost,127.0.0.1'
{}
if ($env:HTTP_PROXY -cne 'http://user''quote@127.0.0.1:8080') {{ throw 'quoted proxy was changed' }}
{}
{}
if ($env:HTTP_PROXY -cne 'http://original.example:8080') {{ throw 'HTTP_PROXY was not restored' }}
if (Test-Path Env:HTTPS_PROXY) {{ throw 'HTTPS_PROXY should remain absent' }}
if ($env:ALL_PROXY -cne 'socks5://original.example:1080') {{ throw 'ALL_PROXY was not restored' }}
if ($env:NO_PROXY -cne 'localhost,127.0.0.1') {{ throw 'NO_PROXY was changed' }}
if (Get-ChildItem Env:XRAT_PROXY_*) {{ throw 'capture markers were not removed' }}
{}
if ($env:HTTP_PROXY -cne 'http://original.example:8080') {{ throw 'repeated restore changed original settings' }}
"#,
        enable(
            "http://user'quote@127.0.0.1:8080",
            "socks5h://127.0.0.1:1080"
        ),
        enable("http://127.0.0.1:9080", "socks5h://127.0.0.1:2080"),
        restore(),
        restore(),
    );
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("proxy.ps1");
    std::fs::write(&path, script).unwrap();
    for executable in ["powershell.exe", "pwsh.exe"] {
        let mut command = std::process::Command::new(executable);
        command.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ]);
        command.arg(&path);
        for name in VARIABLES {
            command.env_remove(format!("XRAT_PROXY_HAD_{name}"));
            command.env_remove(format!("XRAT_PROXY_OLD_{name}"));
        }
        let output = match command.output() {
            Ok(output) => output,
            Err(error)
                if executable == "pwsh.exe" && error.kind() == std::io::ErrorKind::NotFound =>
            {
                continue;
            }
            Err(error) => panic!("could not run {executable}: {error}"),
        };
        assert!(
            output.status.success(),
            "{executable}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
