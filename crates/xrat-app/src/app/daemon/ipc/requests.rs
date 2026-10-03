use super::*;

pub const PROTOCOL_VERSION: u16 = 1;

pub fn default_socket_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("daemon.sock")
}

pub fn daemon_unreachable(err: &crate::app::AppError) -> bool {
    match err {
        crate::app::AppError::Io(io_err) => matches!(
            io_err.kind(),
            ErrorKind::NotFound | ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset
        ),
        _ => false,
    }
}
