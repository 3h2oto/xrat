//! Stable public facade over xrat's reusable proxy-management logic.
//!
//! This crate intentionally exposes a curated subset of the workspace: domain
//! types, subscription/protocol parsing, engine config generation, and probing.
//! Experimental application services require the `services` feature.
//!
//! ```
//! use xrat_sdk::{config::parse_link, engines::xray::generate_runtime_config};
//!
//! let node = parse_link(
//!     "vless://11111111-1111-1111-1111-111111111111@example.com:443#edge"
//! )?.expect("supported link");
//! let config = generate_runtime_config(&node, 1080, Some(8080))?;
//! let json = serde_json::to_string_pretty(&config)?;
//! assert!(json.contains("outbounds"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod model {
    pub use xrat_model::{Node, NodeDedupKey, Protocol};
}

pub mod config {
    pub use xrat_config::{
        ConfigParseError, ImportMode, ImportResult, SubscriptionMetadata, parse_import, parse_link,
        parse_text,
    };
}

pub mod engines;

pub mod prober {
    pub use xrat_prober::{
        AcceptedHttpStatuses, DownloadResult, FailureKind, IcmpResult, ProbeEngineKind,
        RealDelayResult, TcpResult, TestResult, UploadResult, download_speed_check, icmp_ping,
        real_delay_check, tcp_check, upload_speed_check,
    };
}

#[cfg(feature = "services")]
pub mod services {
    pub use xrat_app::app::read_models::{
        ConfigDetail, ConfigSummary, EndpointLocation, LatestTestSummary,
    };
    pub use xrat_app::app::services::testing::TestRunRequest;
    pub use xrat_app::app::services::{
        AppServices, ConfigExportRequest, ConfigLifecycleService, ConfigListRequest,
        ConfigListResult, ConfigService, DeleteOutcome, RestoreOutcome, ToggleOutcome,
    };
}
