use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use std::{path::Path, sync::Arc};

pub(super) async fn load(
    certificate: &Path,
    private_key: &Path,
) -> crate::app::Result<axum_server::tls_rustls::RustlsConfig> {
    let certificate = tokio::fs::read(certificate).await?;
    let key = tokio::fs::read(private_key).await?;
    let invalid = |_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid TLS certificate or private key",
        )
    };
    let certificates = CertificateDer::pem_slice_iter(&certificate)
        .collect::<Result<Vec<_>, _>>()
        .map_err(invalid)?;
    let key = PrivateKeyDer::from_pem_slice(&key).map_err(invalid)?;
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let mut config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|_| std::io::Error::other("TLS protocol configuration failed"))?
        .with_no_client_auth()
        .with_single_cert(certificates, key)
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "TLS certificate and key do not match",
            )
        })?;
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(axum_server::tls_rustls::RustlsConfig::from_config(
        Arc::new(config),
    ))
}
