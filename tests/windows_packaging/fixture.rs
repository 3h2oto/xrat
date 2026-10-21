use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub struct Fixture {
    pub directory: tempfile::TempDir,
    pub executable: PathBuf,
    pub config: PathBuf,
    pub profile: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        let directory = tempfile::Builder::new()
            .prefix("xrat runtime م ")
            .tempdir()
            .unwrap();
        let executable = directory.path().join("xrat.exe");
        let archive = std::env::var_os("XRAT_WINDOWS_ARCHIVE")
            .expect("set XRAT_WINDOWS_ARCHIVE to the native packaged ZIP");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(archive).unwrap()).unwrap();
        let mut entry = archive.by_name("xrat.exe").unwrap();
        std::io::copy(&mut entry, &mut std::fs::File::create(&executable).unwrap()).unwrap();
        let profile = directory.path().join("profile with spaces");
        Self {
            config: profile.join("config.toml"),
            directory,
            executable,
            profile,
        }
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command.env("XRAT_PATH", &self.profile).stdin(Stdio::null());
        command
    }

    pub fn xrat(&self, arguments: &[&str]) -> String {
        let output = self.command().args(arguments).output().unwrap();
        assert!(
            output.status.success(),
            "XRAT {arguments:?} failed: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }

    pub fn password_hash(&self, password: &str) -> String {
        let mut child = self
            .command()
            .arg("hash-password")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(password.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let hash = String::from_utf8(output.stdout).unwrap().trim().to_string();
        assert!(hash.starts_with("$argon2id$"));
        hash
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.command().args(["daemon", "stop"]).output();
        let deadline = Instant::now() + Duration::from_secs(5);
        while std::fs::remove_dir_all(self.directory.path()).is_err() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}
