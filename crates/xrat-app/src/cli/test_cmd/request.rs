use crate::app::services::testing::TestRunRequest;

use super::TestArgs;

impl From<&TestArgs> for TestRunRequest {
    fn from(args: &TestArgs) -> Self {
        Self {
            enabled_only: args.enabled_only,
            active_only: args.active_only,
            skip_icmp: args.skip_icmp,
            skip_tcp: args.skip_tcp,
            skip_real_delay: args.skip_real_delay,
            skip_download: args.skip_download,
            skip_upload: args.skip_upload,
            test_url: args.test_url.clone(),
            download_url: args.download_url.clone(),
            upload_url: args.upload_url.clone(),
            icmp_timeout_ms: args.icmp_timeout_ms,
            tcp_timeout_ms: args.tcp_timeout_ms,
            real_delay_timeout_ms: args.real_delay_timeout_ms,
            download_timeout_ms: args.download_timeout_ms,
            upload_timeout_ms: args.upload_timeout_ms,
            concurrency: args.concurrency,
        }
    }
}
