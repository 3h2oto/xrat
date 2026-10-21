mod windows_installer_support;

use windows_installer_support::Fixture;

#[test]
#[ignore = "requires PowerShell; set XRAT_TEST_POWERSHELL to select its executable"]
fn detects_windows_architecture_and_rejects_unsupported_hosts() {
    let fixture = Fixture::new();
    for (architecture, wow64, expected) in [
        ("AMD64", "", "x86_64-pc-windows-msvc"),
        ("ARM64", "", "aarch64-pc-windows-msvc"),
        ("x86", "AMD64", "x86_64-pc-windows-msvc"),
    ] {
        assert_eq!(
            fixture.run("Get-WindowsTarget", architecture, wow64),
            expected
        );
    }
    fixture.fails("Get-WindowsTarget", "x86", "32-bit Windows");
    fixture.fails(
        "Get-WindowsTarget",
        "other",
        "Unsupported Windows architecture",
    );
    fixture.fails("$env:OS = 'Linux'; Invoke-Install", "AMD64", "Windows only");
}

#[test]
#[ignore = "requires PowerShell"]
fn installer_requires_unique_checksum_and_root_executable() {
    let fixture = Fixture::new();
    assert_eq!(
        fixture.run(
            "Get-ExpectedChecksum $env:XRAT_TEST_SUMS $env:XRAT_TEST_ARCHIVE_NAME",
            "AMD64",
            ""
        ),
        fixture.hash
    );
    fixture.fails(
        "Get-ExpectedChecksum $env:XRAT_TEST_SUMS 'missing.zip'",
        "AMD64",
        "found 0",
    );
    fixture.duplicate_checksum();
    fixture.fails(
        "Get-ExpectedChecksum $env:XRAT_TEST_SUMS $env:XRAT_TEST_ARCHIVE_NAME",
        "AMD64",
        "found 2",
    );
    let output = fixture.run("Expand-VerifiedExecutable -Archive $env:XRAT_TEST_ARCHIVE -Directory (Join-Path $env:XRAT_TEST_ROOT 'extracted') -Name 'xrat.exe'", "AMD64", "");
    assert_eq!(
        std::fs::read_to_string(output).unwrap(),
        "installer fixture"
    );
    fixture.fails("Expand-VerifiedExecutable -Archive $env:XRAT_TEST_ARCHIVE -Directory (Join-Path $env:XRAT_TEST_ROOT 'missing') -Name 'absent.exe'", "AMD64", "exactly one");
}

#[test]
#[ignore = "requires PowerShell"]
fn release_install_preserves_existing_binary_on_checksum_failure() {
    let fixture = Fixture::new();
    assert_eq!(fixture.run("Invoke-Install", "AMD64", ""), "0");
    assert_eq!(fixture.installed(), "installer fixture");
    fixture.tamper_checksum();
    std::fs::write(fixture.bin.join("xrat.exe"), "preserve existing binary").unwrap();
    fixture.fails("Invoke-Install", "AMD64", "Checksum mismatch");
    assert_eq!(fixture.installed(), "preserve existing binary");
}

#[test]
#[ignore = "requires PowerShell"]
fn source_install_and_setup_exit_code_are_preserved() {
    let fixture = Fixture::new();
    let source = "$FromSource = $true; function Build-FromSource { param($Target) if ($Target -ne 'x86_64-pc-windows-msvc') { throw 'wrong source target' }; return $env:XRAT_TEST_SOURCE }; function Invoke-WebRequest { throw 'unexpected release download' }; ";
    assert_eq!(
        fixture.run(&format!("{source} Invoke-Install"), "AMD64", ""),
        "0"
    );
    assert_eq!(fixture.installed(), "installer fixture");
    fixture.fails(
        &format!("{source} $Version = 'v9.8.7'; Invoke-Install"),
        "AMD64",
        "cannot be used together",
    );
    assert_eq!(
        fixture.run(
            &format!("{source} $NoSetup = $false; Invoke-Install"),
            "AMD64",
            ""
        ),
        "0"
    );
    std::fs::write(&fixture.source, "Set-Content -LiteralPath (Join-Path $PSScriptRoot 'setup-arguments.txt') -Value ($args -join ' ')\nexit 7\n").unwrap();
    assert_eq!(fixture.run(&format!("{source} $BinaryName = 'setup.ps1'; $Yes = $true; $NoSetup = $false; Invoke-Install"), "AMD64", ""), "7");
    assert_eq!(
        std::fs::read_to_string(fixture.bin.join("setup-arguments.txt"))
            .unwrap()
            .trim(),
        "setup -y"
    );
}
