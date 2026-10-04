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
        let (launch, replace_running) = match self.active_session_state().await? {
            ActiveSessionState::Running(session) => {
                if !self.context.app_config.runtime.replace_active_session {
                    tracing::warn!(
                        session_id = session.id,
                        "active runtime session blocks connect"
                    );
                    return Err(AppError::RuntimeSessionAlreadyActive);
                }

                (self.resolve_launch(&config)?, true)
            }
            ActiveSessionState::Stale(session) => {
                tracing::warn!(
                    session_id = session.id,
                    "stale runtime session was reconciled before connect"
                );
                (self.resolve_launch(&config)?, false)
            }
            ActiveSessionState::None => (self.resolve_launch(&config)?, false),
        };
        // A running TUN session owns the configured interface name, and both
        // preflight and the launch need that name free. Release the live session
        // and any stale interface before validating in that case.
        if tun_enabled {
            if replace_running {
                self.disconnect().await?;
            }
            self.cleanup_stale_tun_interface()?;
            crate::app::tun_privileges::ensure_engine_capability(&launch.binary_path)?;
        }
        // Validate the replacement before tearing down a healthy session so a
        // failed preflight leaves the running runtime untouched.
        preflight_runtime_with_spawner(
            &launch,
            &self.context.runtime_paths.runtime_dir,
            self.process_ports.spawner.clone(),
        )?;
        if replace_running && !tun_enabled {
            self.disconnect().await?;
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
                return Err(tun_startup_error(error, tun_enabled));
            }
        };

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

    /// Remove the configured TUN interface when it exists but no live session
    /// owns it. Xray and sing-box create the device by name and fail to start
    /// when the name is already taken, so a leftover interface blocks launch.
    fn cleanup_stale_tun_interface(&self) -> crate::app::Result<()> {
        let interface = self.context.app_config.runtime.tun.interface_name.trim();
        if interface.is_empty() || !xrat_support::net::interface_exists(interface) {
            return Ok(());
        }
        let output = xrat_support::process::Command::with_spawner(
            "ip",
            self.process_ports.spawner.clone(),
        )
        .args(["link", "del", interface])
        .output()
        .map_err(|error| {
            AppError::InvalidArgument(format!(
                "TUN interface \"{interface}\" already exists and could not be removed automatically: {error}. Remove it manually, for example with `sudo ip link del {interface}`."
            ))
        })?;
        if output.status.success() {
            tracing::info!(interface, "removed stale TUN interface before launch");
            return Ok(());
        }
        Err(AppError::InvalidArgument(format!(
            "TUN interface \"{interface}\" already exists and could not be removed ({}): {}. Remove it manually, for example with `sudo ip link del {interface}`.",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim(),
        )))
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
