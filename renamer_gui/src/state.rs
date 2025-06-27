use parking_lot::RwLock;
use slint::ModelRc;
use std::{
    cell::OnceCell,
    collections::HashMap,
    fmt::Debug,
    path::PathBuf,
    rc::Weak,
    sync::{Arc, LazyLock, Mutex},
    thread::JoinHandle,
};

use crate::lib_thread::{self, FromLibReciever, ToLibSender};
use crate::slint_generatedRenamerWindow::{S_Action, S_ActionGroup, S_File};
use renamer_lib::{ActionGroup, ActionType};

// slint::include_modules!();

pub type RenamerState = Arc<RwLock<Renamer>>;
pub type WeakRenamerState = Weak<RwLock<Renamer>>;

pub fn init_state() -> (RenamerState, JoinHandle<()>) {
    let (lib_handle, sender, reciever) = lib_thread::setup();
    let renamer = Renamer::new(sender, reciever);
    (Arc::new(RwLock::new(renamer)), lib_handle)
}

pub struct Renamer {
    sender: ToLibSender,
    reciever: FromLibReciever,
    action_groups: HashMap<i32, ActionGroup>,
}
impl Debug for Renamer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Action Groups: {:?}\n", self.action_groups)
    }
}

impl Renamer {
    fn new(sender: ToLibSender, reciever: FromLibReciever) -> Self {
        Self {
            sender,
            reciever,
            action_groups: HashMap::new(),
        }
    }
    pub fn compute_ui_data(&self) -> ModelRc<S_ActionGroup> {
        self.action_groups
            .iter()
            .map(|x| x.into())
            .collect::<Vec<_>>()
            .as_slice()
            .into()
    }
}

impl Into<S_Action> for (&i32, &renamer_lib::Action) {
    fn into(self) -> S_Action {
        let action_info = match self.1 {
            renamer_lib::Action::Randomize => "".into(),
            renamer_lib::Action::Rename(renaming_pattern) => "Rename: Pattern TODO",
        }
        .into();
        S_Action {
            action_info,
            action_type: self.1.get_type().to_string().into(),
            id: *self.0,
        }
    }
}
impl Into<S_File> for (&i32, &PathBuf) {
    fn into(self) -> S_File {
        S_File {
            id: *self.0,
            path: self.1.to_string_lossy().to_string().into(),
        }
    }
}
impl Into<S_ActionGroup> for (&i32, &ActionGroup) {
    fn into(self) -> S_ActionGroup {
        let (id, group) = self;
        S_ActionGroup {
            actions: group
                .actions()
                .iter()
                .map(|x| x.into())
                .collect::<Vec<S_Action>>()
                .as_slice()
                .into(),
            files: group
                .files()
                .iter()
                .map(|x| x.into())
                .collect::<Vec<_>>()
                .as_slice()
                .into(),
            id: *id,
        }
    }
}
