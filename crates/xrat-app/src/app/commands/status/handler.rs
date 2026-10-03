use super::*;

pub async fn run(context: &AppContext, args: &StatusArgs) -> crate::app::Result<()> {
    let socket_path = ipc::default_socket_path(&context.runtime_paths.runtime_dir);
    match ipc::runtime_status_daemon(&socket_path).await {
        Ok(response) => display::print_daemon_status(context, response, args.json).await,
        Err(err) if ipc::daemon_unreachable(&err) => {
            Err(crate::app::AppError::InvalidArgument(format!(
                "daemon is not running. Start it with `xrat daemon start` (socket: {})",
                socket_path.display()
            )))
        }
        Err(err) => Err(err),
    }
}
