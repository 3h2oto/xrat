use super::*;

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
        let mut command = tokio::process::Command::from(command(spec)?);
        command.kill_on_drop(spec.kill_on_drop).output().await
    }
}
