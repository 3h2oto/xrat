use super::*;

fn assert_label(mutations: &dioxus::core::Mutations, expected: &str) {
    assert!(mutations.edits.iter().any(|edit| matches!(edit, dioxus::core::Mutation::CreateTextNode { value, .. } if value == expected)));
}

#[test]
fn daemon_distinguishes_stopped_unknown_and_stale_states() {
    for (running, stale, expected) in [
        (Some(true), false, "Daemon running"),
        (Some(false), false, "Daemon stopped"),
        (None, false, "Daemon unavailable"),
        (Some(true), true, "Daemon running · Updates paused"),
    ] {
        let mut dom =
            VirtualDom::new_with_props(DaemonStatus, DaemonStatusProps { running, stale });
        dom.provide_root_context(crate::ui::interaction::Interactions::default());
        assert_label(&dom.rebuild_to_vec(), expected);
    }
}

#[test]
fn jobs_keep_both_counts_and_identify_idle_and_stale_activity() {
    for (running, queued, stale, expected) in [
        (0, 0, false, "Jobs idle"),
        (2, 0, false, "Jobs: 2 running · 0 queued"),
        (0, 3, false, "Jobs: 0 running · 3 queued"),
        (2, 3, false, "Jobs: 2 running · 3 queued"),
        (2, 3, true, "Jobs: 2 running · 3 queued · Updates paused"),
    ] {
        let mut dom = VirtualDom::new_with_props(
            JobsStatus,
            JobsStatusProps {
                running,
                queued,
                stale,
            },
        );
        dom.provide_root_context(crate::ui::interaction::Interactions::default());
        assert_label(&dom.rebuild_to_vec(), expected);
    }
}
