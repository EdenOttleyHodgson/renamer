use slint::{ComponentHandle, Weak};

use crate::state::{RenamerState, WeakRenamerState};

use crate::slint_generatedRenamerWindow::{RenamerWindow, S_Action};

fn add_file(group_id: i32, state: RenamerState) {
    log::trace!("Adding File callback triggered");
    if let Some(files) = rfd::FileDialog::new().pick_files() {
        state.write().add_files_to_group(group_id, files);
    } else {
        log::warn!("File adding failed, user may have hit cancel")
    }
}
fn remove_file(group_id: i32, file_id: i32, state: RenamerState) {
    log::trace!("Removing file callback triggered");
    state.write().remove_file_from_group(group_id, file_id);
}
fn add_action(group_id: i32, action: S_Action, state: RenamerState) {
    match renamer_lib::Action::try_from(action) {
        Ok(new_action) => state.write().add_action_to_group(group_id, new_action),
        Err(e) => log::error!("Error adding action!: {e}"),
    }
}
fn remove_action(group_id: i32, action_id: i32, state: RenamerState) {
    log::trace!("Remove action callback triggered");
    state.write().remove_action_from_group(group_id, action_id);
}
fn add_action_group(state: RenamerState) {
    state.write().new_action_group();
}
fn remove_action_group(group_id: i32, state: RenamerState) {
    log::trace!("Remove action group callback triggered");
    state.write().delete_action_group(group_id);
}
fn refresh_state(window: Weak<RenamerWindow>, state: RenamerState) {
    let new_ui_state = state.read().compute_ui_data();
    log::trace!("Setting new ui state: {state:?}, {new_ui_state:?}");
    if let Some(w) = window.upgrade() {
        w.set_action_groups(new_ui_state);
    }
}

pub fn set_callbacks(window: &RenamerWindow, state: RenamerState) {
    let s = state.clone();
    window.on_add_file(move |group_id| add_file(group_id, s.clone()));
    let s = state.clone();
    window.on_remove_file(move |group_id, file_id| remove_file(group_id, file_id, s.clone()));
    let s = state.clone();
    window.on_add_action(move |group_id, s_action| add_action(group_id, s_action, s.clone()));
    let s = state.clone();
    window
        .on_remove_action(move |group_id, action_id| remove_action(group_id, action_id, s.clone()));
    let s = state.clone();
    window.on_add_action_group(move || add_action_group(s.clone()));
    let s = state.clone();
    window.on_remove_action_group(move |group_id| remove_action_group(group_id, s.clone()));
    let weak_window = window.as_weak();
    window.on_refresh_state(move || refresh_state(weak_window.clone(), state.clone()));
}
