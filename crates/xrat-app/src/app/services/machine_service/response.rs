use super::{GATEWAY_VERSION, GatewayPayload};
use crate::app::daemon::ipc::{DaemonResponse, DaemonResponseCode};

pub(crate) fn encode_response(
    response: DaemonResponse<GatewayPayload>,
) -> crate::app::Result<Vec<u8>> {
    let bytes = serde_json::to_vec(&response)?;
    if bytes.len() <= crate::app::daemon::ipc::frame::MAX_MESSAGE_BYTES {
        return Ok(bytes);
    }
    Ok(serde_json::to_vec(&DaemonResponse::<GatewayPayload> {
        protocol_version:GATEWAY_VERSION,ok:false,code:DaemonResponseCode::MessageTooLarge,
        message:"machine operation completed, but its reply exceeds the IPC limit; query a smaller page instead of repeating a mutation".into(),payload:None,
    })?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::services::machine_service::CatalogPayload;

    #[test]
    fn oversized_catalog_reply_reports_completion_without_truncation_or_disconnect() {
        let response = DaemonResponse {
            protocol_version: GATEWAY_VERSION,
            ok: true,
            code: DaemonResponseCode::Ok,
            message: String::new(),
            payload: Some(GatewayPayload::Catalog(CatalogPayload::ExportPage {
                total: 1,
                items: vec!["a".repeat(crate::app::daemon::ipc::frame::MAX_MESSAGE_BYTES)],
            })),
        };
        let reply: DaemonResponse<GatewayPayload> =
            serde_json::from_slice(&encode_response(response).unwrap()).unwrap();
        assert!(!reply.ok);
        assert!(matches!(reply.code, DaemonResponseCode::MessageTooLarge));
        assert!(reply.message.contains("operation completed"));
        assert!(reply.payload.is_none());
    }
}
