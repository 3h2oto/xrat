pub const SYSTEM_SID: &str = "S-1-5-18";
pub const ENDPOINT: &str = r"\\.\pipe\xrat-machine-v1";

#[cfg(any(windows, test))]
pub(crate) fn validate_operator(sid: &str) -> crate::app::Result<()> {
    let components = sid.strip_prefix("S-1-").and_then(|rest| {
        rest.split('-')
            .map(str::parse::<u64>)
            .collect::<Result<Vec<_>, _>>()
            .ok()
    });
    if !components.is_some_and(|parts| {
        (2..=16).contains(&parts.len())
            && parts[0] < (1 << 48)
            && parts[1..].iter().all(|part| *part <= u64::from(u32::MAX))
            && format!(
                "S-1-{}",
                parts
                    .iter()
                    .map(u64::to_string)
                    .collect::<Vec<_>>()
                    .join("-")
            ) == sid
    }) || matches!(sid, "S-1-5-18" | "S-1-5-19" | "S-1-5-20" | "S-1-5-32-544")
    {
        return Err(crate::app::AppError::InvalidArgument(
            "expected an installing-user SID".into(),
        ));
    }
    Ok(())
}
