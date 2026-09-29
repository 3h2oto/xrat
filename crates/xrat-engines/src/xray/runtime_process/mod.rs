mod process;
mod signals;

pub use process::{
    ManagedXrayPaths, ManagedXrayProcess, XrayRuntimeError, process_is_running, spawn_detached,
};
pub use signals::{
    TerminationOutcome, XraySignalError, terminate_process, terminate_process_gracefully,
};

#[cfg(test)]
mod tests;
