use super::response::decode;
use gloo_net::http::Request;
use serde::{Serialize, de::DeserializeOwned};

pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    decode(
        Request::get(&format!("/api/v1/{path}"))
            .send()
            .await
            .map_err(|e| e.to_string())?,
    )
    .await
}
pub async fn post<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
    csrf: &str,
) -> Result<T, String> {
    let request = Request::post(&format!("/api/v1/{path}"))
        .header("x-csrf-token", csrf)
        .json(body)
        .map_err(|e| e.to_string())?;
    decode(request.send().await.map_err(|e| e.to_string())?).await
}
pub async fn delete(path: &str, csrf: &str) -> Result<bool, String> {
    decode(
        Request::delete(&format!("/api/v1/{path}"))
            .header("x-csrf-token", csrf)
            .send()
            .await
            .map_err(|e| e.to_string())?,
    )
    .await
}
