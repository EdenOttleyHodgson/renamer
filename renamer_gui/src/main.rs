mod callbacks;
mod lib_thread;
mod state;
use std::fmt::Debug;
use std::{error::Error, rc::Rc};

use callbacks::set_callbacks;
use slint::{ComponentHandle, ModelRc, VecModel};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter(Some("renamer_gui"), log::LevelFilter::Trace)
        .init();
    log::debug!("Hello !");
    let (state, lib_handle) = state::init_state_debug();

    let window = RenamerWindow::new()?;
    set_callbacks(&window, state.clone());
    window.invoke_refresh_state();
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
