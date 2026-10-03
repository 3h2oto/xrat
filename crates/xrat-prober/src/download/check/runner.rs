use super::*;

#[allow(clippy::too_many_arguments)]
pub async fn download_speed_check(
    node: &Node,
    test_url: &str,
    engine: ProbeEngineKind,
    binary_path: &Path,
    startup_timeout: Duration,
    request_timeout: Duration,
    gen_options: &XrayGenOptions,
) -> DownloadResult {
    let local_port = match proxied::find_available_port().await {
        Ok(port) => port,
        Err(error) => {
            return DownloadResult::process_failure(format!(
                "Failed to find available port: {error}"
            ));
        }
    };

    let process = match ProbeProcess::spawn(
        node,
        local_port,
        engine,
        binary_path,
        gen_options,
        startup_timeout,
    )
    .await
    {
        Ok(process) => process,
        Err((kind, reason)) => return DownloadResult::failure(kind, reason),
    };

    let result = proxied::make_proxied_download(local_port, test_url, request_timeout).await;
    let _ = process.kill();

    result
}
