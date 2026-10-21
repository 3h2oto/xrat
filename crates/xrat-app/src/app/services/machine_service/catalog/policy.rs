use super::{CatalogRequest, PAGE_LIMIT};
use crate::app::services::{ConfigListRequest, validate_top};
use crate::app::{AppError, Result};
use xrat_db::{ImportSource, SourceKind};
use xrat_model::Node;

const TEXT_LIMIT: usize = 4 * 1024 * 1024;
const FIELD_LIMIT: usize = 4096;

pub(super) fn validate(request: &CatalogRequest) -> Result<()> {
    match request {
        CatalogRequest::Profile {} => {}
        CatalogRequest::Reference { config_id } => {
            if config_id.0 <= 0 {
                return Err(invalid("config ID must be positive"));
            }
        }
        CatalogRequest::List { filter } | CatalogRequest::ExportPage { filter } => {
            validate_filter(filter)?;
        }
        CatalogRequest::DeleteMany { ids, .. } | CatalogRequest::RestoreMany { ids } => {
            if ids.len() > 10_000 || ids.iter().any(|id| id.0 <= 0) {
                return Err(invalid(
                    "machine catalog batches require at most 10000 positive config IDs",
                ));
            }
        }
        CatalogRequest::Config { reference }
        | CatalogRequest::ResolveConfig { reference }
        | CatalogRequest::Enable { reference }
        | CatalogRequest::Disable { reference }
        | CatalogRequest::Delete { reference, .. }
        | CatalogRequest::Restore { reference } => {
            validate_reference(reference.as_ref())?;
        }
        CatalogRequest::Subscription { reference }
        | CatalogRequest::ResolveSubscription { reference }
        | CatalogRequest::DeleteSubscription { reference } => validate_reference(reference)?,
        CatalogRequest::RenameSubscription { reference, name } => {
            validate_reference(reference)?;
            if name.len() > FIELD_LIMIT {
                return Err(invalid("subscription name exceeds 4096 bytes"));
            }
        }
        CatalogRequest::Import {
            contents,
            source,
            name,
            headers,
        } => {
            if contents.len() > 1024 || contents.iter().map(String::len).sum::<usize>() > TEXT_LIMIT
            {
                return Err(invalid(
                    "machine import exceeds 4 MiB or 1024 subscription bodies",
                ));
            }
            validate_source(source)?;
            if name.as_ref().is_some_and(|name| name.len() > FIELD_LIMIT) {
                return Err(invalid("subscription name exceeds 4096 bytes"));
            }
            if headers.len() > 32
                || headers
                    .iter()
                    .map(|(key, value)| key.len() + value.len())
                    .sum::<usize>()
                    > 64 * 1024
            {
                return Err(invalid("machine subscription headers exceed their limit"));
            }
            if !headers.is_empty() && source.kind != SourceKind::Url {
                return Err(invalid("subscription headers require URL source metadata"));
            }
            crate::app::import::parse_request_headers(headers)?;
        }
        CatalogRequest::Subscriptions { offset, limit } => {
            if usize::try_from(*offset).is_err() || !(1..=PAGE_LIMIT as u16).contains(limit) {
                return Err(invalid("invalid subscription page"));
            }
        }
    }
    Ok(())
}

fn validate_reference(reference: &str) -> Result<()> {
    if reference.is_empty() || reference.len() > 128 {
        return Err(invalid("invalid machine catalog reference"));
    }
    Ok(())
}

fn validate_source(source: &ImportSource) -> Result<()> {
    // This metadata is persisted for identity/display; the privileged host never opens it.
    let limit = if source.kind == SourceKind::RawText {
        TEXT_LIMIT
    } else {
        FIELD_LIMIT
    };
    if source.value.len() > limit
        || source
            .name
            .as_ref()
            .is_some_and(|name| name.len() > FIELD_LIMIT)
    {
        return Err(invalid("machine import source metadata exceeds its limit"));
    }
    if source.kind == SourceKind::Url {
        let url = url::Url::parse(&source.value)
            .map_err(|_| invalid("invalid subscription URL metadata"))?;
        if !matches!(url.scheme(), "http" | "https") || url.host().is_none() {
            return Err(invalid("subscription URL metadata requires HTTP(S)"));
        }
    }
    Ok(())
}

fn validate_filter(filter: &ConfigListRequest) -> Result<()> {
    if filter.subscription_id.is_some_and(|id| id.0 <= 0)
        || filter
            .protocol
            .as_ref()
            .is_some_and(|value| value.len() > 32)
    {
        return Err(invalid("invalid machine catalog filter"));
    }
    if let Some(top) = filter.top_by_real_delay {
        validate_top(top)?;
    }
    if filter.offset.is_none_or(|offset| offset < 0)
        || filter
            .limit
            .is_none_or(|limit| !(1..=PAGE_LIMIT).contains(&limit))
    {
        return Err(invalid(
            "machine catalog queries require a nonnegative offset and a limit of 1..200",
        ));
    }
    Ok(())
}

pub(super) fn validate_nodes(nodes: &[Node]) -> Result<()> {
    if nodes.len() > 10_000 {
        return Err(invalid("machine import exceeds 10000 nodes"));
    }
    for node in nodes {
        for value in [
            Some(node.address.as_str()),
            Some(node.network.as_str()),
            node.username.as_deref(),
            node.uuid.as_deref(),
            node.password.as_deref(),
            node.method.as_deref(),
            node.tls.as_deref(),
            node.sni.as_deref(),
            node.host.as_deref(),
            node.path.as_deref(),
            node.name.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if value.len() > FIELD_LIMIT {
                return Err(invalid("machine config field exceeds 4096 bytes"));
            }
        }
        for (key, value) in node.extensions.iter().flatten() {
            if !SAFE_EXTENSIONS.contains(&key.as_str())
                || !matches!(
                    value,
                    serde_json::Value::String(_)
                        | serde_json::Value::Bool(_)
                        | serde_json::Value::Number(_)
                )
                || value
                    .as_str()
                    .is_some_and(|value| value.len() > FIELD_LIMIT)
            {
                return Err(invalid(&format!(
                    "link parameter {key:?} is not permitted in the machine catalog; structured engine extensions require administrator provisioning"
                )));
            }
        }
    }
    Ok(())
}

const SAFE_EXTENSIONS: &[&str] = &[
    "email",
    "group",
    "name",
    "remark",
    "remarks",
    "v",
    "aid",
    "scy",
    "security",
    "flow",
    "encryption",
    "allowInsecure",
    "insecure",
    "fp",
    "alpn",
    "cipherSuites",
    "cs",
    "ech",
    "pcs",
    "vcn",
    "pbk",
    "password",
    "sid",
    "spx",
    "mldsa65Verify",
    "pqv",
    "headerType",
    "heartbeatPeriod",
    "mode",
    "authority",
    "serviceName",
    "idleTimeout",
    "idle_timeout",
    "health_check_timeout",
    "healthCheckTimeout",
    "permit_without_stream",
    "permitWithoutStream",
    "initial_windows_size",
    "initialWindowsSize",
    "userAgent",
    "user_agent",
    "seed",
    "congestion",
    "readBufferSize",
    "writeBufferSize",
    "cwndMultiplier",
    "maxSendingWindow",
    "mtu",
    "tti",
    "uplinkCapacity",
    "downlinkCapacity",
    "pinSHA256",
    "ed",
    "obfs",
    "obfs-password",
    "upmbps",
    "downmbps",
    "x_padding_bytes",
    "x_padding bytes",
    "xPaddingBytes",
    "xPaddingKey",
    "xPaddingHeader",
    "xPaddingPlacement",
    "xPaddingMethod",
    "uplinkHTTPMethod",
    "sessionIDPlacement",
    "sessionIDKey",
    "sessionIDTable",
    "sessionIDLength",
    "seqPlacement",
    "seqKey",
    "uplinkDataPlacement",
    "uplinkDataKey",
    "uplinkChunkSize",
    "scMaxEachPostBytes",
    "scMinPostsIntervalMs",
    "scStreamUpServerSecs",
    "xPaddingObfsMode",
    "noGRPCHeader",
    "noSSEHeader",
    "scMaxBufferedPosts",
    "serverMaxHeaderBytes",
    "support-x25519mlkem768",
    "support_x25519mlkem768",
    "supportX25519Mlkem768",
    "supportX25519mlkem768",
];

fn invalid(message: &str) -> AppError {
    AppError::InvalidArgument(message.into())
}
