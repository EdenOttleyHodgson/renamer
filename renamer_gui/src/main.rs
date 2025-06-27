mod lib_thread;
use std::fmt::Debug;
use std::{error::Error, rc::Rc};

use lib_thread::{FromLibReciever, ToLibSender};
use renamer_lib::{ActionGroup, ActionType};
use slint::{ComponentHandle, ModelRc, VecModel};

struct Renamer {
    sender: ToLibSender,
    reciever: FromLibReciever,
    action_groups: Vec<ActionGroup>,
    next_action_group_id: usize,
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
            action_groups: Vec::new(),
            next_action_group_id: 0,
        }
    }
}
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter(Some("renamer_gui"), log::LevelFilter::Trace)
        .init();
    log::debug!("Hello !");

    let (lib_handle, sender, reciever) = lib_thread::setup();
    let window = RenamerWindow::new()?;
    window.set_action_groups(ModelRc::from([dummy_action_group()]));

    window.run()?;

    let _ = lib_handle.join();
    Ok(())
}

fn dummy_action_group() -> S_ActionGroup {
    S_ActionGroup {
        id: 1,
        files: Rc::new(VecModel::from(vec![S_File {
            id: 1,
            path: "hi".into(),
        }]))
        .into(),
        actions: Rc::new(VecModel::from(vec![S_Action {
            action_info: "Action Info".into(),
            action_type: "Randomize".into(),
            id: 1,
        }]))
        .into(),
    }
}
