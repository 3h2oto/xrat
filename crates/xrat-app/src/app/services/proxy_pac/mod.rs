mod model;
mod render;

pub use model::{PacEndpoints, PacRules};
pub use render::render_pac;

mod service;
pub use service::{ActiveEndpoints, active_endpoints, active_pac, context_pac};

mod proxy_service;
pub(crate) use proxy_service::ProxyService;

mod frontend_url;
pub(crate) use frontend_url::frontend_pac_url;
