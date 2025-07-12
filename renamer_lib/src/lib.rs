pub mod error;
mod file_actions;
pub mod patterns;
pub mod report;
mod runner;
use std::{collections::HashMap, path::PathBuf};

use file_actions::{FileAction, FileActionType};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use runner::RunnerConfig;
pub use runner::run_actions;

#[derive(Default, Debug, Clone)]
pub struct ActionGroup {
    id: i32,
    files: HashMap<i32, PathBuf>,
    next_file_id: i32,
    actions: HashMap<i32, Action>,
    next_action_id: i32,
}

impl Into<Vec<FileAction>> for ActionGroup {
    fn into(self) -> Vec<FileAction> {
        self.files
            .into_iter()
            .map(|(_, p)| {
                self.actions
                    .iter()
                    .map(|(_, act)| act.make_file_action(p.clone()))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }
}

impl ActionGroup {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn run(self, num_threads: usize) -> Vec<Result<report::Report, error::ActionError>> {
        run_actions(self.into(), RunnerConfig::new(num_threads))
    }
    pub fn files_mut(&mut self) -> &mut HashMap<i32, PathBuf> {
        &mut self.files
    }
    pub fn add_file(&mut self, file: PathBuf) {
        self.files.insert(self.next_file_id, file);
        self.next_file_id += 1;
    }
    pub fn add_action(&mut self, action: Action) {
        self.actions.insert(self.next_action_id, action);
        self.next_action_id += 1;
    }

    pub fn actions_mut(&mut self) -> &mut HashMap<i32, Action> {
        &mut self.actions
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn files(&self) -> &HashMap<i32, PathBuf> {
        &self.files
    }

    pub fn actions(&self) -> &HashMap<i32, Action> {
        &self.actions
    }
}
#[derive(Debug, Clone)]
pub enum Action {
    Randomize,
    Rename(patterns::RenamePattern),
}

impl Action {
    fn make_file_action(&self, file: PathBuf) -> FileAction {
        let action_type = match self {
            Action::Randomize => FileActionType::Randomize,
            Action::Rename(renaming_pattern) => {} //Do like, pattern processing here
        };
        FileAction::new(action_type, file)
    }
    pub fn get_type(&self) -> ActionType {
        match self {
            Action::Randomize => ActionType::Randomize,
            Action::Rename(_) => ActionType::Rename,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ActionType {
    Randomize,
    Rename,
}
impl From<Action> for ActionType {
    fn from(value: Action) -> Self {
        match value {
            Action::Randomize => Self::Randomize,
            Action::Rename(renaming_pattern) => Self::Rename,
        }
    }
}
impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone)]
pub struct RenamingPattern;
