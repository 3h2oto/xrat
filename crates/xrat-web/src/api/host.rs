use super::response::decode;
use gloo_net::http::Request;

pub async fn host_action(
    action: &xrat_web::types::Action,
    csrf: &str,
    unlock: &str,
) -> Result<xrat_web::types::Job, String> {
    let request = Request::post("/api/v1/actions")
        .header("x-csrf-token", csrf)
        .header("x-xrat-host-unlock", unlock)
        .json(action)
        .map_err(|error| error.to_string())?;
    decode(request.send().await.map_err(|error| error.to_string())?).await
}
