use super::*;

pub(super) fn ok_response<T>(message: &str, payload: T) -> DaemonResponse<T> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: message.to_string(),
        payload: Some(payload),
    }
}

pub(super) fn error_response<T>(code: DaemonResponseCode, message: String) -> DaemonResponse<T> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: false,
        code,
        message,
        payload: None,
    }
}
