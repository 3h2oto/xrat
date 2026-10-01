use crate::app::context::AppContext;

pub(super) use crate::app::tests::TestAppBuilder;
pub(super) use crate::app::tests::fixtures::{test_node_with as test_node, test_source};

pub(super) async fn test_context(prefix: &str) -> AppContext {
    TestAppBuilder::new(prefix).build().await
}
