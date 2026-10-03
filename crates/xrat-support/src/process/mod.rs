use std::ffi::{OsStr, OsString};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Output};
use std::sync::Arc;

use async_trait::async_trait;
mod system;
pub use system::SystemProcessSpawner;

pub enum Stdio {
    Inherit,
    Null,
    Piped,
    File(std::fs::File),
}
impl Stdio {
    pub fn null() -> Self {
        Self::Null
    }
    pub fn piped() -> Self {
        Self::Piped
    }
    pub fn inherit() -> Self {
        Self::Inherit
    }
}
impl From<std::fs::File> for Stdio {
    fn from(file: std::fs::File) -> Self {
        Self::File(file)
    }
}

pub struct CommandSpec {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub env: Vec<(OsString, OsString)>,
    pub current_dir: Option<PathBuf>,
    pub stdin: Option<Stdio>,
    pub stdout: Option<Stdio>,
    pub stderr: Option<Stdio>,
    pub kill_on_drop: bool,
}

pub trait ChildHandle: Send {
    fn id(&self) -> u32;
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>>;
    fn wait(&mut self) -> io::Result<ExitStatus>;
    fn kill(&mut self) -> io::Result<()>;
}
pub struct Child {
    handle: Box<dyn ChildHandle>,
    pub stdout: Option<Box<dyn Read + Send>>,
    pub stderr: Option<Box<dyn Read + Send>>,
}
impl Child {
    pub fn from_handle(
        handle: Box<dyn ChildHandle>,
        stdout: Option<Box<dyn Read + Send>>,
        stderr: Option<Box<dyn Read + Send>>,
    ) -> Self {
        Self {
            handle,
            stdout,
            stderr,
        }
    }
    pub fn id(&self) -> u32 {
        self.handle.id()
    }
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.handle.try_wait()
    }
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.handle.wait()
    }
    pub fn kill(&mut self) -> io::Result<()> {
        self.handle.kill()
    }
}

#[async_trait]
pub trait ProcessSpawner: Send + Sync {
    fn spawn(&self, spec: &CommandSpec) -> io::Result<Child>;
    fn run(&self, spec: &CommandSpec, capture: bool) -> io::Result<Output>;
    async fn output_async(&self, spec: &CommandSpec) -> io::Result<Output>;
}

pub struct Command {
    pub spec: CommandSpec,
    spawner: Arc<dyn ProcessSpawner>,
}
impl Command {
    pub fn new(program: impl AsRef<OsStr>) -> Self {
        Self::with_spawner(program, Arc::new(SystemProcessSpawner))
    }
    pub fn with_spawner(program: impl AsRef<OsStr>, spawner: Arc<dyn ProcessSpawner>) -> Self {
        Self {
            spec: CommandSpec {
                program: program.as_ref().into(),
                args: Vec::new(),
                env: Vec::new(),
                current_dir: None,
                stdin: None,
                stdout: None,
                stderr: None,
                kill_on_drop: false,
            },
            spawner,
        }
    }
    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.spec.args.push(arg.as_ref().into());
        self
    }
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for arg in args {
            self.arg(arg);
        }
        self
    }
    pub fn env(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> &mut Self {
        self.spec
            .env
            .push((key.as_ref().into(), value.as_ref().into()));
        self
    }
    pub fn current_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.spec.current_dir = Some(path.as_ref().into());
        self
    }
    pub fn stdin(&mut self, value: Stdio) -> &mut Self {
        self.spec.stdin = Some(value);
        self
    }
    pub fn stdout(&mut self, value: Stdio) -> &mut Self {
        self.spec.stdout = Some(value);
        self
    }
    pub fn stderr(&mut self, value: Stdio) -> &mut Self {
        self.spec.stderr = Some(value);
        self
    }
    pub fn kill_on_drop(&mut self, value: bool) -> &mut Self {
        self.spec.kill_on_drop = value;
        self
    }
    pub fn spawn(&mut self) -> io::Result<Child> {
        self.spawner.spawn(&self.spec)
    }
    pub fn output(&mut self) -> io::Result<Output> {
        self.spawner.run(&self.spec, true)
    }
    pub fn status(&mut self) -> io::Result<ExitStatus> {
        self.spawner
            .run(&self.spec, false)
            .map(|output| output.status)
    }
    pub async fn output_async(&mut self) -> io::Result<Output> {
        self.spawner.output_async(&self.spec).await
    }
}

impl ChildHandle for Child {
    fn id(&self) -> u32 {
        self.handle.id()
    }
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.handle.try_wait()
    }
    fn wait(&mut self) -> io::Result<ExitStatus> {
        self.handle.wait()
    }
    fn kill(&mut self) -> io::Result<()> {
        self.handle.kill()
    }
}

#[cfg(all(test, unix))]
mod tests;
