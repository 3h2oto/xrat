use super::{ActiveEndpoints, PacRules, active_endpoints, active_pac};
use crate::app::{Result, config::ServerSettings, context::AppContext};

#[derive(Clone)]
pub(crate) struct ProxyService {
    backend: Backend,
    frontend: Frontend,
}

#[derive(Clone)]
enum Backend {
    Local {
        db: xrat_db::Database,
        rules: PacRules,
    },
    #[cfg(any(windows, test))]
    Machine(crate::app::services::machine_service::MachineClient),
}
#[derive(Clone)]
enum Frontend {
    Settings(Box<ServerSettings>),
    #[cfg(any(windows, test))]
    Operator,
}

impl ProxyService {
    pub fn from_context(context: &AppContext) -> Self {
        #[cfg(any(windows, test))]
        if let Some(client) = &context.machine_client {
            return Self {
                backend: Backend::Machine(client.clone()),
                frontend: Frontend::Settings(Box::new(context.app_config.server.clone())),
            };
        }
        Self {
            backend: Backend::Local {
                db: context.db.clone(),
                rules: PacRules::from_routing(&context.app_config.routing),
            },
            frontend: Frontend::Settings(Box::new(context.app_config.server.clone())),
        }
    }

    #[cfg(windows)]
    pub fn for_machine(
        client: crate::app::services::machine_service::MachineClient,
    ) -> Result<Self> {
        crate::app::context::require_ordinary_user()?;
        Ok(Self {
            backend: Backend::Machine(client),
            frontend: Frontend::Operator,
        })
    }

    #[cfg(test)]
    pub(crate) fn for_machine_test(
        client: crate::app::services::machine_service::MachineClient,
    ) -> Self {
        Self {
            backend: Backend::Machine(client),
            frontend: Frontend::Operator,
        }
    }

    pub fn is_machine(&self) -> bool {
        #[cfg(any(windows, test))]
        {
            matches!(&self.backend, Backend::Machine(_))
        }
        #[cfg(not(any(windows, test)))]
        {
            false
        }
    }

    pub fn server(&self) -> Result<ServerSettings> {
        match &self.frontend {
            Frontend::Settings(settings) => Ok(settings.as_ref().clone()),
            #[cfg(any(windows, test))]
            Frontend::Operator => {
                let path = crate::app::app_paths::resolve()?
                    .root_dir
                    .join("machine-client/config.toml");
                operator_settings(&path)
            }
        }
    }

    pub async fn endpoints(&self) -> Result<ActiveEndpoints> {
        match &self.backend {
            Backend::Local { db, .. } => active_endpoints(db).await,
            #[cfg(any(windows, test))]
            Backend::Machine(client) => {
                let overview = client.overview().await?;
                Ok(super::service::from_session(
                    overview
                        .runtime
                        .session
                        .as_ref()
                        .filter(|_| overview.runtime.pid_running),
                ))
            }
        }
    }

    pub async fn pac(&self) -> Result<String> {
        match &self.backend {
            Backend::Local { db, rules } => active_pac(db, rules).await,
            #[cfg(any(windows, test))]
            Backend::Machine(client) => {
                use crate::app::services::diagnostics::{DiagnosticsPayload, DiagnosticsRequest};
                match client.diagnostics(DiagnosticsRequest::Pac {}).await? {
                    DiagnosticsPayload::Pac(pac) => Ok(pac),
                    _ => Err(crate::app::AppError::InvalidArgument(
                        "unexpected machine PAC response".into(),
                    )),
                }
            }
        }
    }
}

#[cfg(any(windows, test))]
fn operator_settings(path: &std::path::Path) -> Result<ServerSettings> {
    #[derive(Default, serde::Deserialize)]
    #[serde(default)]
    struct Preferences {
        server: ServerSettings,
    }
    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ServerSettings::default());
        }
        Err(error) => return Err(error.into()),
    };
    Ok(toml::from_str::<Preferences>(&contents)?.server)
}

#[cfg(test)]
mod tests;
