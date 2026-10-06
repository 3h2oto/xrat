use super::*;

impl RuntimeService<'_> {
    pub(crate) fn verify_tun_interface(
        &self,
    ) -> crate::app::Result<Option<xrat_support::net::KernelInterfaceInfo>> {
        let interface = self.context.app_config.runtime.tun.interface_name.trim();
        if interface.is_empty() {
            return Ok(None);
        }
        let Some(info) = self
            .process_ports
            .tun
            .inspect_interface(interface)
            .map_err(|error| {
                AppError::InvalidArgument(format!(
                    "failed to inspect network interface \"{interface}\": {error}"
                ))
            })?
        else {
            return Ok(None);
        };

        if !info.is_tun {
            return Err(AppError::InvalidArgument(format!(
                "network interface \"{interface}\" already exists and is not a TUN device; refusing to remove non-TUN interface"
            )));
        }

        let Some(record) = tun_ownership::load_ownership(&self.context.runtime_paths.runtime_dir)
        else {
            return Err(AppError::InvalidArgument(format!(
                "TUN interface \"{interface}\" already exists but its ownership by XRAT could not be verified; refusing to remove unowned interface. Remove it manually, for example with `sudo ip link del {interface}`."
            )));
        };

        if record.interface_name != interface {
            return Err(AppError::InvalidArgument(format!(
                "TUN interface \"{interface}\" already exists but does not match owned interface \"{}\"; refusing to remove unowned interface.",
                record.interface_name
            )));
        }

        let Some(expected_ifindex) = record.ifindex.filter(|index| *index > 0) else {
            return Err(AppError::InvalidArgument(format!(
                "TUN interface \"{interface}\" has no verified kernel index; refusing to remove unverified interface"
            )));
        };
        if info.ifindex != expected_ifindex {
            return Err(AppError::InvalidArgument(format!(
                "TUN interface \"{interface}\" (index {}) does not match previously recorded interface index {expected_ifindex}; refusing to remove unverified interface.",
                info.ifindex
            )));
        }
        Ok(Some(info))
    }

    pub(crate) fn cleanup_stale_tun_interface(&self) -> crate::app::Result<()> {
        let Some(info) = self.verify_tun_interface()? else {
            tun_ownership::clear_ownership(&self.context.runtime_paths.runtime_dir);
            return Ok(());
        };
        let interface = self.context.app_config.runtime.tun.interface_name.trim();
        self.process_ports
            .tun
            .delete_interface(interface, info.ifindex)
            .map_err(|error| {
                AppError::InvalidArgument(format!(
                    "stale TUN interface \"{interface}\" could not be removed ({error}). Ensure CAP_NET_ADMIN or remove it manually, for example with `sudo ip link del {interface}`."
                ))
            })?;

        tracing::info!(
            interface,
            ifindex = info.ifindex,
            "removed stale TUN interface before launch"
        );
        tun_ownership::clear_ownership(&self.context.runtime_paths.runtime_dir);
        Ok(())
    }
}
