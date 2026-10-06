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
        match Box::pin(self.connect(ConnectRequest { config_id })).await {
            Ok(_) => AppError::InvalidArgument(format!("{error}; previous runtime was restored")),
            Err(rollback_error) => AppError::InvalidArgument(format!(
                "{error}; rollback also failed: {rollback_error}"
            )),
        }
    }
}
