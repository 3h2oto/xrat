use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ReleaseNotes {
    pub version: String,
    pub body: String,
    pub seen: bool,
}

impl ReleaseNotes {
    pub fn pending(root: &Path, version: &str) -> Option<Self> {
        let notes: Self =
            serde_json::from_slice(&std::fs::read(root.join("release-notes.json")).ok()?).ok()?;
        (!notes.seen && same_version(&notes.version, version)).then_some(notes)
    }

    pub fn save(&self, root: &Path) -> crate::app::Result<()> {
        std::fs::create_dir_all(root)?;
        let mut staged = tempfile::NamedTempFile::new_in(root)?;
        staged.write_all(&serde_json::to_vec(self)?)?;
        staged.as_file().sync_all()?;
        staged
            .persist(root.join("release-notes.json"))
            .map_err(|error| error.error)?;
        Ok(())
    }
}

fn same_version(left: &str, right: &str) -> bool {
    left.trim().trim_start_matches('v') == right.trim().trim_start_matches('v')
}

pub(crate) async fn cache_release(client: &xrat_support::http::Client, version: &str) {
    let result = async {
        let root = crate::app::app_paths::resolve()?.root_dir;
        let cached: Option<ReleaseNotes> = std::fs::read(root.join("release-notes.json"))
            .ok().and_then(|bytes| serde_json::from_slice(&bytes).ok());
        if cached.as_ref().is_some_and(|notes| same_version(&notes.version, version)) {
            return Ok::<_, crate::app::AppError>(());
        }
        let fetched = fetch_notes(client, version).await;
        let notes = ReleaseNotes {
            version: version.to_string(),
            body: fetched.unwrap_or_else(|| format!(
                "Release notes are unavailable.\n\nhttps://github.com/{}/releases/tag/{version}", super::releases::REPO
            )),
            seen: false,
        };
        notes.save(&root)
    }.await;
    if let Err(error) = result {
        tracing::warn!(%error, "could not cache release notes");
    }
}

async fn fetch_notes(client: &xrat_support::http::Client, version: &str) -> Option<String> {
    let tag = percent_encoding::utf8_percent_encode(version, percent_encoding::NON_ALPHANUMERIC);
    let response = client
        .clone()
        .with_timeout(std::time::Duration::from_secs(8))
        .get(format!(
            "https://api.github.com/repos/{}/releases/tags/{tag}",
            super::releases::REPO
        ))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let payload: serde_json::Value = serde_json::from_str(&response.text().await.ok()?).ok()?;
    let body = payload.get("body")?.as_str()?.trim();
    (!body.is_empty()).then(|| body.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeHttp(&'static str);

    #[async_trait::async_trait]
    impl xrat_support::http::HttpClient for FakeHttp {
        async fn execute(
            &self,
            request: xrat_support::http::HttpRequest,
        ) -> Result<xrat_support::http::HttpResponse, xrat_support::http::HttpError> {
            assert!(request.url.ends_with("/releases/tags/v1%2E2%2E3"));
            assert_eq!(
                request.options.timeout,
                Some(std::time::Duration::from_secs(8))
            );
            Ok(xrat_support::http::HttpResponse::from_bytes(
                xrat_support::http::StatusCode::OK,
                self.0.as_bytes().to_vec(),
            ))
        }
    }

    #[tokio::test]
    async fn fetches_exact_release_and_tolerates_missing_or_invalid_notes() {
        for (body, expected) in [
            (r##"{"body":"# Changes"}"##, Some("# Changes")),
            ("{}", None),
            ("broken", None),
            (r#"{"body":null}"#, None),
            (r#"{"body":"  "}"#, None),
        ] {
            let client = xrat_support::http::Client::with_transport(
                std::sync::Arc::new(FakeHttp(body)),
                xrat_support::http::HttpOptions::default(),
            );
            assert_eq!(fetch_notes(&client, "v1.2.3").await.as_deref(), expected);
        }
    }

    #[test]
    fn notes_survive_restart_and_only_dismissed_versions_are_seen() {
        let root = tempfile::tempdir().unwrap();
        let mut notes = ReleaseNotes {
            version: "v1.2.3".into(),
            body: "# Hello".into(),
            seen: false,
        };
        notes.save(root.path()).unwrap();
        assert!(ReleaseNotes::pending(root.path(), "1.2.3").is_some());
        assert!(ReleaseNotes::pending(root.path(), "1.2.4").is_none());
        notes.seen = true;
        notes.save(root.path()).unwrap();
        assert!(ReleaseNotes::pending(root.path(), "1.2.3").is_none());
        std::fs::write(root.path().join("release-notes.json"), "broken").unwrap();
        assert!(ReleaseNotes::pending(root.path(), "1.2.3").is_none());
    }
}
