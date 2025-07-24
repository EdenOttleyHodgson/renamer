pub mod error;
pub mod patterns;
pub mod report;
use std::{collections::HashMap, fs, path::PathBuf};

use error::SendableErr;
pub use patterns::{PatternParseError, RenamePattern};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use report::Report;

#[derive(Default, Debug, Clone)]
pub struct ActionGroup {
    id: i32,
    files: HashMap<i32, PathBuf>,
    next_file_id: i32,
    patterns: HashMap<i32, RenamePattern>,
    next_action_id: i32,
}

impl ActionGroup {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn files_mut(&mut self) -> &mut HashMap<i32, PathBuf> {
        &mut self.files
    }
    pub fn add_file(&mut self, file: PathBuf) {
        self.files.insert(self.next_file_id, file);
        self.next_file_id += 1;
    }
    pub fn add_pattern(&mut self, pattern: RenamePattern) {
        self.patterns.insert(self.next_action_id, pattern);
        self.next_action_id += 1;
    }

    pub fn patterns_mut(&mut self) -> &mut HashMap<i32, RenamePattern> {
        &mut self.patterns
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn files(&self) -> &HashMap<i32, PathBuf> {
        &self.files
    }

    pub fn patterns(&self) -> &HashMap<i32, RenamePattern> {
        &self.patterns
    }

    pub fn set_patterns(&mut self, actions: HashMap<i32, RenamePattern>) {
        self.patterns = actions;
    }

    fn generate_actions(&self) -> Vec<Result<Action, SendableErr>> {
        // for pattern in self.patterns.values() {
        //     for file in self.files.values() {
        //         match Action::new(file.clone(), pattern) {
        //             Ok(act) => actions.push(act),
        //             Err(e) => errors.push(e),
        //         }
        //     }
        // }
        self.patterns
            .par_iter()
            .map(|(_, pat)| {
                self.files
                    .par_iter()
                    .map(|(_, path)| Action::new(path.clone(), pat))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }
    pub fn execute(&self) -> Vec<Result<Report, SendableErr>> {
        self.generate_actions()
            .into_par_iter()
            .map(|res| match res {
                Ok(act) => act.execute(),
                Err(e) => Err(e),
            })
            .collect()
    }
}

struct Action {
    old: PathBuf,
    new: PathBuf,
}
impl Action {
    fn new(old: PathBuf, pattern: &RenamePattern) -> Result<Action, SendableErr> {
        let new = pattern.apply_to_file_name(&old)?;
        Ok(Self { old, new })
    }
    fn execute(&self) -> Result<Report, SendableErr> {
        let mut new = self.new.clone();
        let mut count = 0;
        while fs::exists(&new)? {
            count += 1;
            new = append_to_path(new, &count.to_string());
        }
        fs::rename(&self.old, &new)?;
        Ok(Report::Renamed {
            from: self.old.clone(),
            to: new.clone(),
            overwrote: false,
        })
    }
}

fn append_to_path(p: PathBuf, s: &str) -> PathBuf {
    let mut p = p.into_os_string();
    p.push(s);
    p.into()
}
