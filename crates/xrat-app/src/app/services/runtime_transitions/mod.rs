use crate::app::AppError;
use crate::app::context::AppContext;
use crate::app::runtime_service::{
    ConnectRequest, ConnectResult, DisconnectResult, RuntimeService,
};

mod transitions;
pub use transitions::DisconnectOutcome;
pub use transitions::RuntimeTransitionService;
