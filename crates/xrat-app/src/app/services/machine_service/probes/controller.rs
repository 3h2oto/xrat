use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use super::{ProbeProgressPage, ProbeRequest, ProbeSnapshot, ProbeStages, ProbeState};
use crate::app::context::AppContext;
use crate::app::services::testing::{self, TestProgressUpdate, TestRunRequest};
use crate::app::{AppError, Result};
use xrat_model::ConfigId;
use xrat_support::cancel::{CancellationFlag, CancellationReceiver};

const RETAINED_JOBS: usize = 16;
const MAX_CONFIGS: usize = 10_000;

#[derive(Clone)]
pub(in crate::app::services::machine_service) struct ProbeJobs {
    context: AppContext,
    state: Arc<Mutex<State>>,
}

#[derive(Default)]
struct State {
    closing: bool,
    jobs: VecDeque<ProbeSnapshot>,
    progress: HashMap<String, testing::TestActivityProgress>,
    active: Option<Active>,
    results: Option<(String, Vec<testing::TestOutputRow>)>,
}

struct Active {
    id: String,
    cancel: CancellationFlag,
    task: JoinHandle<()>,
}

impl ProbeJobs {
    #[cfg(any(windows, test))]
    pub fn new(context: AppContext) -> Self {
        Self {
            context,
            state: Arc::new(Mutex::new(State::default())),
        }
    }

    pub async fn dispatch(&self, request: ProbeRequest) -> Result<Option<ProbeSnapshot>> {
        match request {
            ProbeRequest::Start { config_ids, stages } => {
                self.start(config_ids, stages).await.map(Some)
            }
            ProbeRequest::Status { job_id } => {
                if let Some(id) = &job_id {
                    validate_id(id)?;
                }
                let mut state = self.state.lock().await;
                reap(&mut state).await;
                Ok(match job_id {
                    Some(id) => state.jobs.iter().find(|job| job.job_id == id),
                    None => state.jobs.back(),
                }
                .cloned())
            }
            ProbeRequest::Cancel { job_id } => {
                validate_id(&job_id)?;
                let mut state = self.state.lock().await;
                reap(&mut state).await;
                if let Some(active) = &state.active
                    && active.id == job_id
                {
                    active.cancel.cancel();
                    if let Some(job) = state.jobs.iter_mut().find(|job| job.job_id == job_id)
                        && matches!(job.state, ProbeState::Running | ProbeState::Cancelling)
                    {
                        job.state = ProbeState::Cancelling;
                    }
                }
                Ok(state.jobs.iter().find(|job| job.job_id == job_id).cloned())
            }
        }
    }

    pub async fn progress(
        &self,
        id: &str,
        offset: u32,
        limit: u16,
    ) -> Result<Option<ProbeProgressPage>> {
        validate_id(id)?;
        if offset > MAX_CONFIGS as u32 || !(1..=200).contains(&limit) {
            return Err(AppError::InvalidArgument(
                "invalid probe progress page".into(),
            ));
        }
        let state = self.state.lock().await;
        Ok(state.progress.get(id).map(|progress| ProbeProgressPage {
            total: progress.rows.len() as u32,
            rows: progress
                .rows
                .values()
                .skip(offset as usize)
                .take(usize::from(limit))
                .cloned()
                .collect(),
        }))
    }

    pub async fn results(
        &self,
        id: &str,
        offset: u32,
        limit: u16,
    ) -> Result<Option<super::ProbeResultPage>> {
        validate_id(id)?;
        if offset > MAX_CONFIGS as u32 || !(1..=200).contains(&limit) {
            return Err(AppError::InvalidArgument(
                "invalid probe result page".into(),
            ));
        }
        let state = self.state.lock().await;
        Ok(state
            .results
            .as_ref()
            .filter(|(job, _)| job == id)
            .map(|(_, rows)| super::ProbeResultPage {
                total: rows.len() as u32,
                rows: rows
                    .iter()
                    .skip(offset as usize)
                    .take(usize::from(limit))
                    .cloned()
                    .map(|output| super::ProbeResult { output })
                    .collect(),
            }))
    }

    async fn start(&self, ids: Vec<ConfigId>, stages: ProbeStages) -> Result<ProbeSnapshot> {
        let unique: HashSet<_> = ids.iter().collect();
        if ids.is_empty()
            || ids.len() > MAX_CONFIGS
            || ids.iter().any(|id| id.0 <= 0)
            || unique.len() != ids.len()
        {
            return Err(AppError::InvalidArgument(
                "probe jobs require 1..10000 distinct positive config IDs".into(),
            ));
        }
        let mut state = self.state.lock().await;
        reap(&mut state).await;
        if state.closing || state.active.is_some() {
            return Err(AppError::InvalidArgument(
                "machine probe runner is stopping or already has an active job".into(),
            ));
        }
        state.results = None;
        let snapshot = ProbeSnapshot {
            job_id: uuid::Uuid::new_v4().to_string(),
            state: ProbeState::Running,
            run_id: None,
            completed: 0,
            total: ids.len(),
            last_config_id: None,
            error: None,
        };
        state.jobs.push_back(snapshot.clone());
        state.progress.insert(
            snapshot.job_id.clone(),
            testing::TestActivityProgress::default(),
        );
        while state.jobs.len() > RETAINED_JOBS {
            if let Some(expired) = state.jobs.pop_front() {
                state.progress.remove(&expired.job_id);
            }
        }
        let cancel = CancellationFlag::new();
        let task = tokio::spawn(run(
            self.context.clone(),
            self.state.clone(),
            snapshot.job_id.clone(),
            ids,
            stages,
            cancel.clone(),
        ));
        state.active = Some(Active {
            id: snapshot.job_id.clone(),
            cancel,
            task,
        });
        Ok(snapshot)
    }

    pub async fn shutdown(&self) -> Result<()> {
        let active = {
            let mut state = self.state.lock().await;
            state.closing = true;
            state.active.take()
        };
        if let Some(active) = active {
            active.cancel.cancel();
            active.task.await?;
        }
        Ok(())
    }
}

async fn reap(state: &mut State) {
    if state
        .active
        .as_ref()
        .is_some_and(|active| active.task.is_finished())
        && let Some(active) = state.active.take()
        && let Err(error) = active.task.await
        && let Some(job) = state.jobs.iter_mut().find(|job| job.job_id == active.id)
    {
        job.state = ProbeState::Failed;
        job.error = Some(error.to_string());
    }
}

fn validate_id(id: &str) -> Result<()> {
    uuid::Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| AppError::InvalidArgument("invalid machine probe job ID".into()))
}

async fn run(
    context: AppContext,
    state: Arc<Mutex<State>>,
    job_id: String,
    ids: Vec<ConfigId>,
    stages: ProbeStages,
    cancel: CancellationFlag,
) {
    let request = TestRunRequest {
        skip_icmp: stages.skip_icmp,
        skip_tcp: stages.skip_tcp,
        skip_real_delay: stages.skip_real_delay,
        skip_download: stages.skip_download,
        skip_upload: stages.skip_upload,
        ..Default::default()
    };
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let (activity, mut activities) = tokio::sync::mpsc::unbounded_channel();
    let (started, mut run_started) = tokio::sync::oneshot::channel();
    let operation = testing::run_machine_probe(
        &request,
        &context,
        &ids,
        CancellationReceiver::from_flag(&cancel),
        sender,
        started,
        activity,
    );
    tokio::pin!(operation);
    let mut run_announced = false;
    let result = loop {
        tokio::select! {
            result = &mut operation => break result,
            id = &mut run_started, if !run_announced => {
                run_announced = true;
                if let Ok(id) = id {
                    update_run_id(&state, &job_id, id).await;
                }
            }
            Some(update) = receiver.recv() => update_progress(&state, &job_id, update).await,
            Some(update) = activities.recv() => update_activity(&state, &job_id, update).await,
        }
    };
    if !run_announced && let Ok(id) = run_started.try_recv() {
        update_run_id(&state, &job_id, id).await;
    }
    while let Ok(update) = receiver.try_recv() {
        update_progress(&state, &job_id, update).await;
    }
    while let Ok(update) = activities.try_recv() {
        update_activity(&state, &job_id, update).await;
    }
    let mut state = state.lock().await;
    let result = result.and_then(|mut outcome| {
        let rows = outcome.local_results.take().ok_or_else(|| {
            AppError::InvalidArgument("machine probe runner returned no local results".into())
        })?;
        state.results = Some((job_id.clone(), rows));
        Ok(outcome)
    });
    if let Some(job) = state.jobs.iter_mut().find(|job| job.job_id == job_id) {
        match result {
            Ok(outcome) => {
                job.run_id = outcome.run_id;
                job.completed = outcome.completed;
                job.total = outcome.total;
                job.state = if cancel.is_cancelled() {
                    ProbeState::Cancelled
                } else {
                    ProbeState::Completed
                };
            }
            Err(error) => {
                job.state = ProbeState::Failed;
                // Keep status responses bounded even when an external tool emits a large error.
                job.error = Some(error.to_string().chars().take(4096).collect());
            }
        }
    }
}

async fn update_progress(state: &Mutex<State>, id: &str, update: TestProgressUpdate) {
    let mut state = state.lock().await;
    if let Some(job) = state.jobs.iter_mut().find(|job| job.job_id == id) {
        job.completed = update.done;
        job.total = update.total;
        job.last_config_id = Some(update.config_id);
    }
}

async fn update_run_id(state: &Mutex<State>, id: &str, run_id: i64) {
    let mut state = state.lock().await;
    if let Some(job) = state.jobs.iter_mut().find(|job| job.job_id == id) {
        job.run_id = Some(run_id);
    }
}

async fn update_activity(state: &Mutex<State>, id: &str, activity: testing::TestActivity) {
    let mut state = state.lock().await;
    if let Some(progress) = state.progress.get_mut(id) {
        progress.apply(activity);
    }
}
