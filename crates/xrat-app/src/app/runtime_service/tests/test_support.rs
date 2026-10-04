use super::super::*;
use xrat_model::Node;

pub(super) use crate::app::tests::TestAppBuilder;
pub(super) use crate::app::tests::fixtures::{test_node_with, test_source};

pub(super) async fn test_context() -> AppContext {
    TestAppBuilder::new("runtime-service").build().await
}

pub(super) fn test_node() -> Node {
    test_node_with("example.com", "test")
}
