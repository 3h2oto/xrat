use std::ptr;
use windows_sys::Win32::System::EventLog::{
    DeregisterEventSource, EVENTLOG_ERROR_TYPE, RegisterEventSourceW, ReportEventW,
};

pub(super) fn describe(error: &dyn std::error::Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

pub(super) fn failure(message: &str) {
    let source = "xrat-daemon\0".encode_utf16().collect::<Vec<_>>();
    let message = message
        .chars()
        .filter(|character| *character != '\0')
        .take(16_000)
        .collect::<String>();
    let text = message.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let handle = unsafe { RegisterEventSourceW(ptr::null(), source.as_ptr()) };
    if handle.is_null() {
        return;
    }
    unsafe {
        ReportEventW(
            handle,
            EVENTLOG_ERROR_TYPE,
            0,
            1,
            ptr::null_mut(),
            1,
            0,
            [text.as_ptr()].as_ptr(),
            ptr::null(),
        );
        DeregisterEventSource(handle);
    }
}
