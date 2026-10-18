use serde::{Deserialize, de::DeserializeOwned};

#[derive(Deserialize)]
struct ErrorBody {
    error: String,
}

pub(super) async fn decode<T: DeserializeOwned>(
    response: gloo_net::http::Response,
) -> Result<T, String> {
    if !response.ok() {
        let status = response.status();
        if status == 400 {
            return Err(response
                .json::<ErrorBody>()
                .await
                .map(|body| body.error)
                .unwrap_or_else(|_| "The value could not be saved.".into()));
        }
        return Err(match status {
            401 => "Session expired. Please sign in.".into(),
            403 => {
                "This operation is not permitted. You may need to change your temporary password."
                    .into()
            }
            429 => "Too many requests. Please wait a minute.".into(),
            _ => format!("Request failed ({status})."),
        });
    }
    response.json().await.map_err(|e| e.to_string())
}
