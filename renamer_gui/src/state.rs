use parking_lot::RwLock;
use std::{
    cell::OnceCell,
    collections::HashMap,
    fmt::Debug,
    rc::Weak,
    sync::{Arc, LazyLock, Mutex},
    thread::JoinHandle,
};

use crate::lib_thread::{self, FromLibReciever, ToLibSender};
use renamer_lib::{ActionGroup, ActionType};

type RenamerState = Arc<RwLock<Renamer>>;
type WeakRenamerState = Weak<RwLock<Renamer>>;

pub fn init_state() -> (RenamerState, JoinHandle<()>) {
    let (lib_handle, sender, reciever) = lib_thread::setup();
    let renamer = Renamer::new(sender, reciever);
    (Arc::new(RwLock::new(renamer)), lib_handle)
}

pub struct Renamer {
    sender: ToLibSender,
    reciever: FromLibReciever,
    action_groups: HashMap<usize, ActionGroup>,
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
}
