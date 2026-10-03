use super::*;

mod connect_flow;

impl<'a> RuntimeService<'a> {
    pub fn new(context: &'a AppContext) -> Self {
        Self::with_process_ports(
            context,
            xrat_support::readiness::RuntimeProcessPorts::default(),
        )
    }

    pub fn with_process_ports(
        context: &'a AppContext,
        process_ports: xrat_support::readiness::RuntimeProcessPorts,
    ) -> Self {
        Self {
            context,
            process_ports,
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn disconnect(&self) -> crate::app::Result<DisconnectResult> {
        let stopped_session =
            stop_active_session(self.context, self.process_ports.signals.as_ref()).await?;
        Ok(DisconnectResult { stopped_session })
    }
}
