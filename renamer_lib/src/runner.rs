use std::{
    error::Error,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::error::ActionError;
use crate::file_actions::{FileAction, FileActionContext, FileActionType};
use crate::report::Report;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

pub fn run_actions(
    actions: Vec<FileAction>,
    config: RunnerConfig,
) -> Vec<Result<Report, ActionError>> {
    log::trace!("Running actions: {actions:?}");
    if config.num_threads < 2 {
        run_singlethreaded(actions)
    } else {
        run_multithreaded(actions, config.num_threads)
    }
}

pub struct RunnerConfig {
    num_threads: usize,
}

impl RunnerConfig {
    pub fn new(num_threads: usize) -> Self {
        Self { num_threads }
    }
}

fn run_multithreaded(
    actions: Vec<FileAction>,
    num_threads: usize,
) -> Vec<Result<Report, ActionError>> {
    log::trace!("Running actions multithreaded");
    let ctx = Arc::new(FileActionContext::new());
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("Thread pool should be buildable!");
    let results = pool.install(move || {
        let results = Mutex::new(Vec::new());
        actions.into_par_iter().for_each(|action| {
            let res = action
                .run(ctx.clone())
                .map_err(Into::into)
                .inspect(|x| log::info!("Action completed succesfully: {x:?}"))
                .inspect_err(|x| log::error!("Action failed to complete: {x}"));
            match results.lock() {
                Ok(mut results) => results.push(res),
                Err(e) => log::error!("Thread poisoned!: {e}"),
            }
        });

        results
    });
    match results.into_inner() {
        Ok(res) => res,
        Err(e) => {
            log::error!("Thread poisoned!: {e}");
            e.into_inner()
        }
    }
}

fn run_singlethreaded(actions: Vec<FileAction>) -> Vec<Result<Report, ActionError>> {
    log::trace!("Running actions single threaded");
    let ctx = Arc::new(FileActionContext::new());
    let mut results = Vec::new();
    for action in actions {
        let res = action
            .run(ctx.clone())
            .map_err(Into::into)
            .inspect(|x| log::info!("Action completed succesfully: {x:?}"))
            .inspect_err(|x| log::error!("Action failed to complete: {x}"));
        results.push(res);
    }
    results
}
