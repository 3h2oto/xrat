use sha2::{Digest, Sha256};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

pub struct Fixture {
    root: tempfile::TempDir,
    pub bin: PathBuf,
    pub source: PathBuf,
    pub hash: String,
}

impl Fixture {
    pub fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("xrat installer م ")
            .tempdir()
            .unwrap();
        let source = root.path().join("source.ps1");
        std::fs::write(&source, "installer fixture").unwrap();
        let archive = root.path().join("fixture.zip");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&archive).unwrap());
        zip.start_file("xrat.exe", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"installer fixture").unwrap();
        zip.finish().unwrap();
        let hash = format!("{:x}", Sha256::digest(std::fs::read(archive).unwrap()));
        let fixture = Self {
            bin: root.path().join("bin"),
            root,
            source,
            hash,
        };
        fixture.write_checksum(&fixture.hash);
        fixture
    }

    fn write_checksum(&self, hash: &str) {
        std::fs::write(
            self.root.path().join("SHASUMS256.txt"),
            format!("{hash}  xrat-v9.8.7-x86_64-pc-windows-msvc.zip\n"),
        )
        .unwrap();
    }

    pub fn duplicate_checksum(&self) {
        let path = self.root.path().join("SHASUMS256.txt");
        let content = std::fs::read_to_string(&path).unwrap();
        std::fs::write(path, content.repeat(2)).unwrap();
    }

    pub fn tamper_checksum(&self) {
        self.write_checksum(&"0".repeat(64));
    }
    pub fn installed(&self) -> String {
        std::fs::read_to_string(self.bin.join("xrat.exe")).unwrap()
    }

    fn invoke(&self, body: &str, architecture: &str, wow64: &str) -> Output {
        let script = self.root.path().join("invoke.ps1");
        let prelude = r#"
$ErrorActionPreference = 'Stop'
$OutputEncoding = [Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding = $OutputEncoding
Import-Module (Join-Path $PSHOME 'Modules/Microsoft.PowerShell.Utility/Microsoft.PowerShell.Utility.psd1') -ErrorAction Stop
$env:PROCESSOR_ARCHITECTURE = $env:XRAT_TEST_ARCHITECTURE
$env:PROCESSOR_ARCHITEW6432 = $env:XRAT_TEST_WOW64
. $env:XRAT_INSTALL_SCRIPT -InstallDir $env:XRAT_TEST_BIN -NoSetup
function Get-LatestReleaseTag { return 'v9.8.7' }
function Add-UserPathEntry { param($Directory) return $false }
function Invoke-WebRequest {
    param($Uri, $OutFile, [switch]$UseBasicParsing)
    if ($Uri.EndsWith('/SHASUMS256.txt')) { Copy-Item -LiteralPath $env:XRAT_TEST_SUMS -Destination $OutFile }
    elseif ($Uri.EndsWith('/' + $env:XRAT_TEST_ARCHIVE_NAME)) { Copy-Item -LiteralPath $env:XRAT_TEST_ARCHIVE -Destination $OutFile }
    else { throw "Unexpected download: $Uri" }
}
try {
    $result = & {
"#;
        std::fs::write(&script, format!("{prelude}\n{body}\n}}\nWrite-Output ('XRAT_TEST_RESULT=' + [string]$result)\n}} catch {{ [IO.File]::WriteAllText($env:XRAT_TEST_ERROR, $_.Exception.Message, [Text.UTF8Encoding]::new($false)); exit 1 }}\n")).unwrap();
        Command::new(
            std::env::var_os("XRAT_TEST_POWERSHELL").unwrap_or_else(|| "powershell.exe".into()),
        )
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(script)
        .stdin(Stdio::null())
        .env(
            "XRAT_INSTALL_SCRIPT",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("install.ps1"),
        )
        .env("XRAT_TEST_ROOT", self.root.path())
        .env("XRAT_TEST_ERROR", self.root.path().join("error.txt"))
        .env("USERPROFILE", self.root.path())
        .env("XRAT_TEST_BIN", &self.bin)
        .env("XRAT_TEST_SOURCE", &self.source)
        .env("XRAT_TEST_SUMS", self.root.path().join("SHASUMS256.txt"))
        .env("XRAT_TEST_ARCHIVE", self.root.path().join("fixture.zip"))
        .env(
            "XRAT_TEST_ARCHIVE_NAME",
            "xrat-v9.8.7-x86_64-pc-windows-msvc.zip",
        )
        .env("OS", "Windows_NT")
        .env("XRAT_TEST_ARCHITECTURE", architecture)
        .env("XRAT_TEST_WOW64", wow64)
        .output()
        .expect("run PowerShell installer fixture")
    }

    pub fn run(&self, body: &str, architecture: &str, wow64: &str) -> String {
        let output = self.invoke(body, architecture, wow64);
        assert!(
            output.status.success(),
            "{} {}",
            String::from_utf8_lossy(&output.stderr),
            std::fs::read_to_string(self.root.path().join("error.txt")).unwrap_or_default()
        );
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .find_map(|line| line.strip_prefix("XRAT_TEST_RESULT="))
            .expect("fixture result")
            .to_string()
    }

    pub fn fails(&self, body: &str, architecture: &str, expected: &str) {
        let output = self.invoke(body, architecture, "");
        assert!(
            !output.status.success(),
            "installer unexpectedly accepted {body}"
        );
        let error = std::fs::read_to_string(self.root.path().join("error.txt"))
            .unwrap_or_else(|_| String::from_utf8_lossy(&output.stderr).into_owned());
        assert!(error.contains(expected), "{error}");
    }
}
