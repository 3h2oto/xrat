use crate::app::{AppError, Result, config::TunSettings};
use xrat_engines::xray::config::XrayConfig;

pub(crate) fn configure_xray_dns(config: &mut XrayConfig, tun: &TunSettings) -> Result<()> {
    if !tun.auto_route {
        return Ok(());
    }
    let dns = super::policy::dns_addresses(&tun.address).map_err(AppError::InvalidArgument)?;
    let settings = config
        .inbounds
        .iter_mut()
        .find(|inbound| inbound.tag == "tun-in" && inbound.protocol == "tun")
        .and_then(|inbound| inbound.settings.as_mut())
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| AppError::InvalidArgument("Windows Xray TUN settings are missing".into()))?;
    settings.insert("dns".into(), serde_json::json!(dns));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_dns_is_explicit_and_missing_tun_is_an_error() {
        let mut config: XrayConfig = serde_json::from_value(serde_json::json!({
            "inbounds": [{"tag":"tun-in","protocol":"tun","settings":{}}],
            "outbounds": [], "log": {"loglevel":"warning"}
        }))
        .unwrap();
        let tun = TunSettings {
            auto_route: true,
            address: vec!["172.19.0.1/30".into(), "fd00::1/126".into()],
            ..Default::default()
        };
        configure_xray_dns(&mut config, &tun).unwrap();
        assert_eq!(
            config.inbounds[0].settings.as_ref().unwrap()["dns"],
            serde_json::json!(["172.19.0.2", "fd00::2"])
        );
        config.inbounds.clear();
        assert!(configure_xray_dns(&mut config, &tun).is_err());
        assert!(
            configure_xray_dns(
                &mut config,
                &TunSettings {
                    auto_route: false,
                    ..tun
                }
            )
            .is_ok()
        );
    }
}
