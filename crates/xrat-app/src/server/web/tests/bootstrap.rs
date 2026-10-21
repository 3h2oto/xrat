use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};

#[tokio::test]
async fn csp_authorizes_exact_inline_bootstrap() {
    let html = include_str!("../../../../../xrat-web/index.html");
    let script = html
        .split_once("<script type=\"module\">")
        .unwrap()
        .1
        .split_once("</script>")
        .unwrap()
        .0;
    let hash = STANDARD.encode(Sha256::digest(script.as_bytes()));
    let (router, _) = fixture().await;
    let response = request(&router, "/", "GET", serde_json::Value::Null, "", "", "").await;
    let policy = response.headers()["content-security-policy"]
        .to_str()
        .unwrap();
    let sources = policy
        .split(';')
        .find_map(|directive| directive.trim().strip_prefix("script-src "))
        .unwrap()
        .split_whitespace()
        .collect::<Vec<_>>();
    assert_eq!(
        sources,
        ["'self'", "'wasm-unsafe-eval'", &format!("'sha256-{hash}'")]
    );
    assert!(!html.contains("/boot.js"));
}
