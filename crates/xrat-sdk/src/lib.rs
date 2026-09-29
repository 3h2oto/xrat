//! Stable public facade over xrat's reusable proxy-management logic.
//!
//! This crate intentionally exposes a curated subset of the workspace: domain
//! types, subscription/protocol parsing, probing result types, and the
//! interface-neutral application services. It does not expose raw repository
//! rows, CLI argument structs, Axum DTOs, or process internals, so internal
//! refactors do not immediately become SDK breaking changes.

pub mod model {
    pub use xrat_model::{Node, NodeDedupKey, Protocol};
}

pub mod config {
    pub use xrat_config::{
        ConfigParseError, ImportMode, ImportResult, SubscriptionMetadata, parse_import, parse_link,
        parse_text,
    };
}

pub mod prober {
    pub use xrat_prober::{
        AcceptedHttpStatuses, DownloadResult, FailureKind, IcmpResult, RealDelayResult, TcpResult,
        TestResult, UploadResult,
    };
}

pub mod services {
    pub use xrat_app::app::read_models::{
        ConfigDetail, ConfigSummary, EndpointLocation, LatestTestSummary,
    };
    pub use xrat_app::app::services::testing::TestRunRequest;
    pub use xrat_app::app::services::{
        AppServices, ConfigLifecycleService, ConfigListRequest, ConfigListResult, ConfigService,
        DeleteOutcome, RestoreOutcome, ToggleOutcome,
    };
}
