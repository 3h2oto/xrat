use crate::app::ports::ReleaseProvider;

pub struct GithubReleaseProvider;

#[async_trait::async_trait]
impl ReleaseProvider for GithubReleaseProvider {
    async fn latest_tag(&self, timeout_secs: u64) -> crate::app::Result<String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .user_agent(concat!("xrat/", env!("CARGO_PKG_VERSION")))
            .build()?;
        let response = client
            .get(format!(
                "https://api.github.com/repos/{}/releases/latest",
                super::REPO
            ))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(crate::app::AppError::InvalidArgument(format!(
                "failed to query latest release: HTTP {}",
                response.status()
            )));
        }
        let payload: serde_json::Value = serde_json::from_str(&response.text().await?)?;
        payload
            .get("tag_name")
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                crate::app::AppError::InvalidArgument(
                    "latest release response had no tag_name".into(),
                )
            })
    }
}
