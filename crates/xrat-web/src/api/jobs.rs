use super::client::get;

pub async fn wait_for_job(id: &str) -> Result<(), String> {
    for _ in 0..60 {
        gloo_timers::future::TimeoutFuture::new(1000).await;
        if let Some(job) = get::<Vec<xrat_web::types::Job>>("jobs")
            .await?
            .into_iter()
            .find(|job| job.id == id)
        {
            match job.state.as_str() {
                "succeeded" => return Ok(()),
                "failed" | "interrupted" | "cancelled" => {
                    return Err(job
                        .result
                        .unwrap_or_else(|| "The change did not complete.".into()));
                }
                _ => {}
            }
        }
    }
    Err("The change is still pending. Follow its progress in Jobs.".into())
}
