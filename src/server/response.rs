use serde::Serialize;

use crate::app::read_models::{ConfigDetail, ConfigSummary, LatestTestSummary};

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ApiLatestTest {
    pub tcp_ok: Option<bool>,
    pub tcp_ms: Option<i64>,
    pub real_delay_ok: Option<bool>,
    pub real_delay_ms: Option<i64>,
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
    pub connect_ms: Option<i64>,
    pub ttfb_ms: Option<i64>,
    pub http_status: Option<i64>,
    pub failure_kind: Option<String>,
    pub failure_reason: Option<String>,
    pub tested_at: String,
}

#[derive(Debug, Serialize)]
pub struct ApiConfigSummary {
    pub id: i64,
    pub r#ref: String,
    pub name: Option<String>,
    pub protocol: String,
    pub address: String,
    pub port: i64,
    pub network: String,
    pub tls: Option<String>,
    pub real_delay_ms: Option<i64>,
    pub tcp_ok: Option<bool>,
    pub last_tested_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiConfigDetail {
    pub id: i64,
    pub r#ref: String,
    pub subscription_id: Option<i64>,
    pub dedup_key: String,
    pub protocol: String,
    pub address: String,
    pub port: i64,
    pub name: Option<String>,
    pub network: String,
    pub tls: Option<String>,
    pub sni: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
    pub is_active: bool,
    pub is_enabled: bool,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
    pub imported_at: String,
    pub created_at: String,
    pub updated_at: String,
    pub latest_test: Option<ApiLatestTest>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub total: usize,
    pub page: u64,
    pub per_page: u64,
    pub items: Vec<T>,
}

pub fn summary_from_summary(summary: &ConfigSummary) -> ApiConfigSummary {
    let latest = summary.latest_test.as_ref();
    ApiConfigSummary {
        id: summary.id,
        r#ref: summary.r#ref.clone(),
        name: summary.name.clone(),
        protocol: summary.protocol.clone(),
        address: summary.address.clone(),
        port: summary.port,
        network: summary.network.clone(),
        tls: summary.tls.clone(),
        real_delay_ms: latest.and_then(|test| test.real_delay_ms),
        tcp_ok: latest.and_then(|test| test.tcp_ok),
        last_tested_at: latest.and_then(|test| test.tested_at.clone()),
    }
}

fn latest_test_from_summary(latest: &LatestTestSummary) -> ApiLatestTest {
    ApiLatestTest {
        tcp_ok: latest.tcp_ok,
        tcp_ms: latest.tcp_ms,
        real_delay_ok: latest.real_delay_ok,
        real_delay_ms: latest.real_delay_ms,
        download_mbps: latest.download_mbps,
        upload_mbps: latest.upload_mbps,
        connect_ms: latest.connect_ms,
        ttfb_ms: latest.ttfb_ms,
        http_status: latest.http_status,
        failure_kind: latest.failure_kind.clone(),
        failure_reason: latest.failure_reason.clone(),
        tested_at: latest.tested_at.clone().unwrap_or_default(),
    }
}

pub fn detail_from_model(detail: &ConfigDetail) -> ApiConfigDetail {
    let summary = &detail.summary;
    ApiConfigDetail {
        id: summary.id,
        r#ref: summary.r#ref.clone(),
        subscription_id: detail.subscription_id,
        dedup_key: detail.dedup_key.clone(),
        protocol: summary.protocol.clone(),
        address: summary.address.clone(),
        port: summary.port,
        name: summary.name.clone(),
        network: summary.network.clone(),
        tls: summary.tls.clone(),
        sni: detail.sni.clone(),
        host: detail.host.clone(),
        path: detail.path.clone(),
        is_active: summary.is_active,
        is_enabled: summary.is_enabled,
        is_deleted: summary.is_deleted,
        deleted_at: detail.deleted_at.clone(),
        imported_at: detail.imported_at.clone(),
        created_at: detail.created_at.clone(),
        updated_at: detail.updated_at.clone(),
        latest_test: summary.latest_test.as_ref().map(latest_test_from_summary),
    }
}
