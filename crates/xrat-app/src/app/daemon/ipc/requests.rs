use super::*;

pub const PROTOCOL_VERSION: u16 = 1;

pub fn default_socket_path(runtime_dir: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        use sha2::Digest;
        let absolute =
            std::path::absolute(runtime_dir).unwrap_or_else(|_| runtime_dir.to_path_buf());
        let identity = absolute.canonicalize().unwrap_or(absolute);
        let identity = identity
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .replace('/', "\\")
            .to_lowercase();
        let owner = xrat_support::windows::current_user_sid().unwrap_or_default();
        let digest = sha2::Sha256::digest(format!("{owner}:{identity}"));
        return PathBuf::from(format!(r"\\.\pipe\xrat-{:x}", digest));
    }
    #[cfg(not(windows))]
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
