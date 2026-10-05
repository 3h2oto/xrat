use super::*;

impl<'a> RuntimeService<'a> {
    #[tracing::instrument(skip_all, fields(config_id = request.config_id.0))]
    pub async fn connect(&self, request: ConnectRequest) -> crate::app::Result<ConnectResult> {
        let Some(config) = self.context.db.get_config_by_id(request.config_id).await? else {
            return Err(AppError::InvalidArgument(format!(
                "config {} was not found",
                request.config_id
            )));
        };
        if !config.is_enabled {
            return Err(AppError::InvalidArgument(format!(
                "config {} is disabled",
                request.config_id
            )));
        }
        if config.is_deleted {
            return Err(AppError::InvalidArgument(format!(
                "config {} is deleted",
                request.config_id
            )));
        }

        let tun_enabled = self.context.app_config.runtime.tun.enabled;
        let (launch, active_session) = match self.active_session_state().await? {
            ActiveSessionState::Running(session) => {
                if !self.context.app_config.runtime.replace_active_session {
                    tracing::warn!(
                        session_id = session.id,
                        "active runtime session blocks connect"
                    );
                    return Err(AppError::RuntimeSessionAlreadyActive);
                }

                (self.resolve_launch(&config)?, Some(session))
            }
            ActiveSessionState::Stale(session) => {
                tracing::warn!(
                    session_id = session.id,
                    "stale runtime session was reconciled before connect"
                );
                (self.resolve_launch(&config)?, None)
            }
            ActiveSessionState::None => (self.resolve_launch(&config)?, None),
        };
        let replace_running = active_session.is_some();
        let previous_active_config_id = active_session.as_ref().and_then(|s| s.config_id);

        if tun_enabled {
            crate::app::tun_privileges::ensure_engine_capability(&launch.binary_path)?;
            if !replace_running {
                let interface = self.context.app_config.runtime.tun.interface_name.trim();
                if !interface.is_empty()
                    && let Ok(Some(info)) = self.process_ports.tun.inspect_interface(interface)
                {
                    if !info.is_tun {
                        return Err(AppError::InvalidArgument(format!(
                            "network interface \"{interface}\" already exists and is not a TUN device; refusing to use existing interface"
                        )));
                    }
                    let owned =
                        tun_ownership::load_ownership(&self.context.runtime_paths.runtime_dir)
                            .is_some_and(|r| r.interface_name == interface);
                    if !owned {
                        return Err(AppError::InvalidArgument(format!(
                            "TUN interface \"{interface}\" already exists but its ownership by XRAT could not be verified; refusing to overwrite unowned interface. Remove it manually, for example with `sudo ip link del {interface}`."
                        )));
                    }
                }
            }
        }

        // Validate the replacement before tearing down a healthy session so a
        // failed preflight leaves the running runtime untouched.
        preflight_runtime_with_spawner(
            &launch,
            &self.context.runtime_paths.runtime_dir,
            self.process_ports.spawner.clone(),
        )?;

        if let Some(active) = &active_session {
            stop_session(self.context, active, self.process_ports.signals.as_ref()).await?;
            self.context.db.clear_active_config().await?;
        }

        if tun_enabled {
            self.cleanup_stale_tun_interface()?;
        }

        crate::app::runtime_service::log_retention::cleanup(self.context).await;
        let session_id = self
            .context
            .db
            .insert_runtime_session(&RuntimeSessionInsert {
                config_id: Some(config.id),
                status: RuntimeSessionStatus::Starting,
                socks_host: launch
                    .endpoints
                    .socks
                    .as_ref()
                    .map(|inbound| inbound.host.clone()),
                socks_port: launch
                    .endpoints
                    .socks
                    .as_ref()
                    .map(|inbound| i64::from(inbound.port)),
                http_host: launch
                    .endpoints
                    .http
                    .as_ref()
                    .map(|inbound| inbound.host.clone()),
                http_port: launch
                    .endpoints
                    .http
                    .as_ref()
                    .map(|inbound| i64::from(inbound.port)),
                shadowsocks_host: launch
                    .endpoints
                    .shadowsocks
                    .as_ref()
                    .map(|inbound| inbound.host.clone()),
                shadowsocks_port: launch
                    .endpoints
                    .shadowsocks
                    .as_ref()
                    .map(|inbound| i64::from(inbound.port)),
                process_id: None,
                failure_reason: None,
                started_at: None,
                stopped_at: None,
            })
            .await?;

        if tun_enabled {
            let _ = tun_ownership::save_ownership(
                &self.context.runtime_paths.runtime_dir,
                &tun_ownership::TunOwnershipRecord {
                    interface_name: self.context.app_config.runtime.tun.interface_name.clone(),
                    ifindex: None,
                    session_id,
                    engine: self.context.app_config.runtime.engine.clone(),
                },
            );
        }

        let process = match spawn_runtime_with_ports(
            &launch,
            &self.context.runtime_paths.runtime_dir,
            session_id,
            self.process_ports.clone(),
        )
        .await
        {
            Ok(process) => process,
            Err(error) => {
                self.context
                    .db
                    .update_runtime_session_state(
                        session_id,
                        RuntimeSessionStatus::Failed,
                        None,
                        None,
                        Some(&now_string()),
                        Some(&error.to_string()),
                    )
                    .await?;

                if let Some(prev_config_id) = previous_active_config_id {
                    let rollback = Box::pin(self.connect(ConnectRequest {
                        config_id: prev_config_id,
                    }))
                    .await;
                    return match rollback {
                        Ok(_) => Err(AppError::InvalidArgument(format!(
                            "{}; previous runtime was restored",
                            tun_startup_error(error, tun_enabled)
                        ))),
                        Err(rollback_err) => Err(AppError::InvalidArgument(format!(
                            "{}; rollback also failed: {rollback_err}",
                            tun_startup_error(error, tun_enabled)
                        ))),
                    };
                }

                return Err(tun_startup_error(error, tun_enabled));
            }
        };

        if tun_enabled {
            let interface = self.context.app_config.runtime.tun.interface_name.trim();
            if let Ok(Some(info)) = self.process_ports.tun.inspect_interface(interface) {
                let _ = tun_ownership::save_ownership(
                    &self.context.runtime_paths.runtime_dir,
                    &tun_ownership::TunOwnershipRecord {
                        interface_name: interface.to_string(),
                        ifindex: (info.ifindex > 0).then_some(info.ifindex),
                        session_id,
                        engine: self.context.app_config.runtime.engine.clone(),
                    },
                );
            }
        }

        self.context
            .db
            .update_runtime_session_state(
                session_id,
                RuntimeSessionStatus::Running,
                Some(i64::from(process.pid)),
                Some(&now_string()),
                None,
                None,
            )
            .await?;
        self.context
            .db
            .update_runtime_session_transition_metadata(
                session_id,
                Some("cli"),
                None,
                Some("manual_connect"),
                Some("runtime connect request succeeded"),
                Some("cli"),
            )
            .await?;
        self.context.db.set_active_config(config.id).await?;

        Ok(ConnectResult {
            config,
            session_id,
            pid: process.pid,
            runtime_config_path: process.config_path,
            endpoints: launch.endpoints,
        })
    }

    /// Remove the configured TUN interface when it exists and belongs to a
    /// previous XRAT session. Foreign interfaces and unverified devices are
    /// preserved with a descriptive error.
    pub(crate) fn cleanup_stale_tun_interface(&self) -> crate::app::Result<()> {
        let interface = self.context.app_config.runtime.tun.interface_name.trim();
        if interface.is_empty() {
            return Ok(());
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
            tun_ownership::clear_ownership(&self.context.runtime_paths.runtime_dir);
            return Ok(());
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

        if let Some(expected_ifindex) = record.ifindex
            && info.ifindex != 0
            && info.ifindex != expected_ifindex
        {
            return Err(AppError::InvalidArgument(format!(
                "TUN interface \"{interface}\" (index {}) does not match previously recorded interface index {expected_ifindex}; refusing to remove unverified interface.",
                info.ifindex
            )));
        }

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

/// Append TUN capability guidance to a startup failure when TUN capture is
/// enabled. An engine can pass native validation but still fail to start when it
/// cannot create the interface or system routes.
fn tun_startup_error(error: AppError, tun_enabled: bool) -> AppError {
    let message = error.to_string();
    let looks_like_startup =
        message.contains("exited during startup") || message.contains("startup timeout");
    if tun_enabled && looks_like_startup {
        return AppError::InvalidArgument(format!(
            "{message}; if this is a TUN capture failure, ensure capabilities with `xrat tun setup`"
        ));
    }
    error
}
