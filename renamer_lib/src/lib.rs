pub mod error;
mod file_actions;
pub mod report;
mod runner;
use std::path::PathBuf;

use file_actions::{FileAction, FileActionType};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use runner::RunnerConfig;
pub use runner::run_actions;

pub struct ActionGroup {
    files: Vec<PathBuf>,
    actions: Vec<Action>,
}

impl Into<Vec<FileAction>> for ActionGroup {
    fn into(self) -> Vec<FileAction> {
        self.files
            .into_iter()
            .map(|p| {
                self.actions
                    .iter()
                    .map(|act| act.make_file_action(p.clone()))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }
}

impl ActionGroup {
    pub fn run(self, num_threads: usize) -> Vec<Result<report::Report, error::ActionError>> {
        run_actions(self.into(), RunnerConfig::new(num_threads))
    }
    pub fn files(&mut self) -> &mut Vec<PathBuf> {
        &mut self.files
    }
    pub fn add_file(&mut self, file: PathBuf) {
        self.files.push(file);
    }

    pub fn actions(&mut self) -> &mut Vec<Action> {
        &mut self.actions
    }
}
pub enum Action {
    Randomize,
    Rename(RenamingPattern),
}
impl Action {
    fn make_file_action(&self, file: PathBuf) -> FileAction {
        let action_type = match self {
            Action::Randomize => FileActionType::Randomize,
            Action::Rename(renaming_pattern) => todo!(), //Do like, pattern processing here
        };
        FileAction::new(action_type, file)
    }
}

#[derive(Debug)]
pub struct RenamingPattern;
