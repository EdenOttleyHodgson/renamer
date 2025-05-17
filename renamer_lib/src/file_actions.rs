use std::{error::Error, fs, path::PathBuf, sync::Arc};

use rand::Rng;

use crate::{error::SendableErr, report::Report};

#[derive(Debug)]
pub struct FileAction {
    action_type: FileActionType,
    target: PathBuf,
}
impl FileAction {
    pub fn new(action_type: FileActionType, target: PathBuf) -> Self {
        Self {
            action_type,
            target,
        }
    }

    pub(crate) fn run(self, ctx: Arc<FileActionContext>) -> Result<Report, SendableErr> {
        log::trace!("Running action: {self:?}");
        Ok(match &self.action_type {
            FileActionType::Randomize => self.run_randomize(ctx.clone())?,
            FileActionType::Rename(new_name) => {
                let new_name = new_name.clone();
                self.run_rename(new_name)?
            }
        })
    }
    fn run_randomize(self, ctx: Arc<FileActionContext>) -> Result<Report, SendableErr> {
        log::trace!("Randomizing file: {}", self.target.to_string_lossy());
        let mut new_name = gen_rand_name();
        while fs::exists(&new_name).unwrap_or(true) {
            log::trace!("Encountered random clash while randomizing!");
            new_name = gen_rand_name();
        }
        let mut new_path = self.target.clone();
        new_path.pop();
        new_path.push(new_name);
        fs::rename(&self.target, &new_path)?;
        log::trace!("Randomize complete!");
        Ok(Report::Renamed {
            from: self.target,
            to: new_path,
            overwrote: false,
        })
    }
    fn run_rename(self, new_name: PathBuf) -> Result<Report, SendableErr> {
        log::trace!(
            "Renaming {} to {}",
            self.target.to_string_lossy(),
            new_name.to_string_lossy()
        );
        let mut overwrote = false;
        if fs::exists(&new_name)? {
            overwrote = true;
        }
        fs::rename(&self.target, &new_name)?;
        log::trace!("Rename complete!");
        Ok(Report::Renamed {
            from: self.target,
            to: new_name.clone(),
            overwrote,
        })
    }
}

#[derive(Debug)]
pub enum FileActionType {
    Randomize,
    Rename(PathBuf),
}

#[derive(Debug)]
pub(crate) struct FileActionContext {
    // used_names: RwLock<Vec<&'a String>>,
}

impl FileActionContext {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

fn gen_rand_name() -> String {
    rand::rng().random::<u32>().to_string()
}
