use super::*;
use tokio::io::{AsyncRead, AsyncReadExt};

pub struct SystemProcessSpawner;
struct SystemChild(std::process::Child);
impl ChildHandle for SystemChild {
    fn id(&self) -> u32 {
        self.0.id()
    }
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.0.try_wait()
    }
    fn wait(&mut self) -> io::Result<ExitStatus> {
        self.0.wait()
    }
    fn kill(&mut self) -> io::Result<()> {
        self.0.kill()
    }
}
fn stdio(value: &Stdio) -> io::Result<std::process::Stdio> {
    Ok(match value {
        Stdio::Null => std::process::Stdio::null(),
        Stdio::Piped => std::process::Stdio::piped(),
        Stdio::Inherit => std::process::Stdio::inherit(),
        Stdio::File(file) => std::process::Stdio::from(file.try_clone()?),
    })
}
fn command(spec: &CommandSpec) -> io::Result<std::process::Command> {
    let mut command = std::process::Command::new(&spec.program);
    command.args(&spec.args).envs(spec.env.iter().cloned());
    if let Some(path) = &spec.current_dir {
        command.current_dir(path);
    }
    if let Some(value) = &spec.stdin {
        command.stdin(stdio(value)?);
    }
    if let Some(value) = &spec.stdout {
        command.stdout(stdio(value)?);
    }
    if let Some(value) = &spec.stderr {
        command.stderr(stdio(value)?);
    }
    Ok(command)
}

struct AsyncChild {
    child: Option<tokio::process::Child>,
    kill_on_drop: bool,
    runtime: tokio::runtime::Handle,
}
impl Drop for AsyncChild {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            if self.kill_on_drop {
                let _ = child.start_kill();
            }
            self.runtime.spawn(async move {
                let _ = child.wait().await;
            });
        }
    }
}

async fn read_output(reader: Option<impl AsyncRead + Unpin>) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    if let Some(mut reader) = reader {
        reader.read_to_end(&mut bytes).await?;
    }
    Ok(bytes)
}
#[async_trait]
impl ProcessSpawner for SystemProcessSpawner {
    fn spawn(&self, spec: &CommandSpec) -> io::Result<Child> {
        let mut child = command(spec)?.spawn()?;
        let stdout = child
            .stdout
            .take()
            .map(|pipe| Box::new(pipe) as Box<dyn Read + Send>);
        let stderr = child
            .stderr
            .take()
            .map(|pipe| Box::new(pipe) as Box<dyn Read + Send>);
        Ok(Child::from_handle(
            Box::new(SystemChild(child)),
            stdout,
            stderr,
        ))
    }
    fn run(&self, spec: &CommandSpec, capture: bool) -> io::Result<Output> {
        let mut command = command(spec)?;
        if capture {
            command.output()
        } else {
            command.status().map(|status| Output {
                status,
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
        }
    }
    async fn output_async(&self, spec: &CommandSpec) -> io::Result<Output> {
        let mut builder = command(spec)?;
        if spec.stdin.is_none() {
            builder.stdin(std::process::Stdio::null());
        }
        if spec.stdout.is_none() {
            builder.stdout(std::process::Stdio::piped());
        }
        if spec.stderr.is_none() {
            builder.stderr(std::process::Stdio::piped());
        }
        let mut child = tokio::process::Command::from(builder).spawn()?;
        drop(child.stdin.take());
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let mut owned = AsyncChild {
            child: Some(child),
            kill_on_drop: spec.kill_on_drop,
            runtime: tokio::runtime::Handle::current(),
        };
        let (status, stdout, stderr) = tokio::try_join!(
            owned
                .child
                .as_mut()
                .expect("child retained until output completes")
                .wait(),
            read_output(stdout),
            read_output(stderr),
        )?;
        drop(owned.child.take());
        Ok(Output {
            status,
            stdout,
            stderr,
        })
    }
}
