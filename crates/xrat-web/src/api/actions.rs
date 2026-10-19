use crate::{Notice, Session, api};
use dioxus::prelude::*;
use xrat_web::types::{Action, Job};

pub fn run(action: Action) {
    let Session(session) = consume_context();
    let Notice(mut notice) = consume_context();
    spawn(async move {
        let csrf = session().map(|s| s.csrf_token).unwrap_or_default();
        match api::post::<_, Job>("actions", &action, &csrf).await {
            Ok(job) => notice.set(Some(crate::Feedback::job_started(&job.kind))),
            Err(error) => notice.set(Some(crate::Feedback::error(error))),
        }
    });
}
