use crate::app::Result;

pub(crate) fn ensure_acquisition_allowed() -> Result<()> {
    #[cfg(windows)]
    return validate_windows_identity(&xrat_support::windows::current_user_sid()?);
    #[cfg(not(windows))]
    Ok(())
}

#[cfg(any(windows, test))]
fn validate_windows_identity(sid: &str) -> Result<()> {
    if sid == "S-1-5-18" {
        return Err(crate::app::AppError::InvalidArgument(
            "LocalSystem import input acquisition is disabled; fetch as the installing user and submit validated bodies through the machine gateway".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn machine_host_cannot_acquire_import_inputs() {
        assert!(validate_windows_identity("S-1-5-18").is_err());
        assert!(validate_windows_identity("S-1-5-21-1-2-3-1001").is_ok());
    }
}
