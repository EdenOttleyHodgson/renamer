use parking_lot::RwLock;
use slint::{ModelRc, SharedString, ToSharedString, Weak, format};
use std::{
    cell::OnceCell,
    collections::HashMap,
    fmt::{Debug, Display},
    path::PathBuf,
    rc::Rc,
    sync::{Arc, LazyLock, Mutex},
    thread::{self, JoinHandle},
};

use crate::lib_thread::{self, FromLibMessage, FromLibReciever, ToLibMessage, ToLibSender};
use crate::slint_generatedRenamerWindow::{RenamerWindow, S_Action, S_ActionGroup, S_File};
use renamer_lib::{
    Action, ActionGroup, ActionType, RenamingPattern, patterns::RenamePattern, report::Report,
};

// slint::include_modules!();

pub type RenamerState = Arc<RwLock<Renamer>>;
// pub type WeakRenamerState = Weak<RwLock<Renamer>>;

pub fn init_state(window: Weak<RenamerWindow>) -> (RenamerState, JoinHandle<()>) {
    let (lib_handle, sender, reciever) = lib_thread::setup();
    let renamer = Renamer::new(sender, window);
    (Arc::new(RwLock::new(renamer)), lib_handle)
}

pub fn init_state_debug(window: Weak<RenamerWindow>) -> (RenamerState, JoinHandle<()>) {
    let (lib_handle, sender, reciever) = lib_thread::setup();
    let mut renamer = Renamer::new(sender, window);
    renamer.new_action_group();
    renamer.add_actions_to_group(0, vec![Action::Randomize]);
    let renamer_state = Arc::new(RwLock::new(renamer));
    let s = renamer_state.clone();
    let reciever_handle = thread::spawn(move || lib_thread::handle_gui_messages(reciever, s));
    renamer_state.write().set_reciever_handle(reciever_handle);
    (renamer_state, lib_handle)
}
pub struct Renamer {
    sender: ToLibSender,
    reciever_handle: Option<JoinHandle<()>>,
    action_groups: HashMap<i32, ActionGroup>,
    next_action_group_id: i32,
    window: Weak<RenamerWindow>,
}
impl Debug for Renamer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Action Groups: {:?}\n", self.action_groups)
    }
}

impl Renamer {
    fn new(sender: ToLibSender, window: Weak<RenamerWindow>) -> Self {
        Self {
            sender,
            reciever_handle: None,
            action_groups: HashMap::new(),
            next_action_group_id: 0,
            window,
        }
    }
    fn set_reciever_handle(&mut self, handle: JoinHandle<()>) {
        self.reciever_handle = Some(handle);
    }
    pub fn compute_ui_data(&self) -> ModelRc<S_ActionGroup> {
        self.action_groups
            .iter()
            .map(|x| x.into())
            .collect::<Vec<_>>()
            .as_slice()
            .into()
    }
    pub fn new_action_group(&mut self) {
        self.action_groups.insert(
            self.next_action_group_id,
            ActionGroup::new(self.next_action_group_id),
        );
        self.next_action_group_id += 1;
    }
    pub fn delete_action_group(&mut self, group_id: i32) {
        self.action_groups.remove(&group_id);
    }
    pub fn add_files_to_group(&mut self, id: i32, files: Vec<PathBuf>) {
        if let Some(group) = self.action_groups.get_mut(&id) {
            for file in files {
                group.add_file(file);
            }
        } else {
            log::error!("Non existent action group id!: {} for state {:?}", id, self)
        }
    }
    pub fn add_actions_to_group(&mut self, id: i32, actions: Vec<Action>) {
        if let Some(group) = self.action_groups.get_mut(&id) {
            for action in actions {
                group.add_action(action);
            }
        } else {
            log::error!("Non existent action group id!: {} for state {:?}", id, self)
        }
    }
    pub fn add_action_to_group(&mut self, id: i32, action: Action) {
        if let Some(group) = self.action_groups.get_mut(&id) {
            group.add_action(action);
        } else {
            log::error!("Non existent action group id!: {} for state {:?}", id, self)
        }
    }
    pub fn remove_file_from_group(&mut self, group_id: i32, file_id: i32) {
        if let Some(group) = self.action_groups.get_mut(&group_id) {
            group.files_mut().remove(&file_id);
        }
    }
    pub fn remove_action_from_group(&mut self, group_id: i32, action_id: i32) {
        if let Some(group) = self.action_groups.get_mut(&group_id) {
            group.actions_mut().remove(&action_id);
        }
    }
    pub fn send_message(&mut self, msg: ToLibMessage) {
        let _ = self.sender.send(msg).inspect_err(|e| log::error!("{e}"));
    }
    pub fn execute_actions(&mut self) {
        self.send_message(ToLibMessage::ExecuteActions(
            self.action_groups.values().map(|x| x.clone()).collect(),
        ));
    }
    pub fn handle_message(&mut self, msg: FromLibMessage) {
        log::trace!("Handling Message: {msg:?}");
        match msg {
            FromLibMessage::SuccessfulActions(vec) => {
                let successes = vec
                    .into_iter()
                    .map(report_to_slint_string)
                    .collect::<Vec<_>>();
                let _ = self
                    .window
                    .upgrade_in_event_loop(move |window| {
                        window.set_successes(successes.as_slice().into());
                        window.set_state_flag(crate::StateFlag::Finished);
                    })
                    .inspect_err(|e| log::error!("Error handling message!: {e}"));
            }
            FromLibMessage::UnsuccessfulActions(vec) => {
                let failures = vec
                    .into_iter()
                    .map(|x| x.to_shared_string())
                    .collect::<Vec<_>>();
                let _ = self
                    .window
                    .upgrade_in_event_loop(move |window| {
                        window.set_failures(failures.as_slice().into());
                        window.set_state_flag(crate::StateFlag::Finished);
                    })
                    .inspect_err(|e| log::error!("Error handling message!: {e}"));
            }
        };
    }
}

fn report_to_slint_string(report: Report) -> SharedString {
    match report {
        Report::Renamed {
            from,
            to,
            overwrote,
        } => {
            let mut out = format!("Renamed: \n {from:?} \n to \n {to:?}");
            if overwrote {
                out.push_str("\n (OVERWROTE)");
            }
            SharedString::from(out)
        }
        Report::Nothing => SharedString::from(""),
    }
}

impl TryFrom<S_Action> for renamer_lib::Action {
    fn try_from(value: S_Action) -> Result<Self, String> {
        match value.action_type.as_str() {
            "Randomize" => Ok(Action::Randomize),
            "Rename" => {
                let pattern = RenamePattern::try_from(value.action_info.as_str())
                    .map_err(|x| x.to_string())?;
                Ok(Action::Rename(pattern))
            }
            _ => {
                log::error!("Bad S_Action type!");
                Err("Bad S_Action Type!".to_owned())
            }
        }
    }

    type Error = String;
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
