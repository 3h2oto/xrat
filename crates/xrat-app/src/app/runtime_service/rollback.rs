use super::*;

impl RuntimeService<'_> {
    pub(crate) async fn rollback_runtime_error(
        &self,
        previous_config_id: Option<ConfigId>,
        error: AppError,
    ) -> AppError {
        let Some(config_id) = previous_config_id else {
            return error;
        };
        let rollback = RuntimeService::with_process_ports(
            self.rollback_context.unwrap_or(self.context),
            self.process_ports.clone(),
        );
        match Box::pin(rollback.connect(ConnectRequest { config_id })).await {
            Ok(_) => AppError::InvalidArgument(format!("{error}; previous runtime was restored")),
            Err(rollback_error) => AppError::InvalidArgument(format!(
                "{error}; rollback also failed: {rollback_error}"
            )),
        }
    }
}
