use slint::{ComponentHandle, Weak};

use crate::state::{RenamerState, WeakRenamerState};

use crate::slint_generatedRenamerWindow::RenamerWindow;

fn add_file(group_id: i32, state: RenamerState) {}
fn remove_file(group_id: i32, file_id: i32, state: RenamerState) {}
fn add_action(group_id: i32, state: RenamerState) {}
fn remove_action(group_id: i32, action_id: i32, state: RenamerState) {}
fn add_action_group(state: RenamerState) {}
fn remove_action_group(group_id: i32, state: RenamerState) {}
fn refresh_state(window: Weak<RenamerWindow>, state: RenamerState) {
    let new_ui_state = state.read().compute_ui_data();
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
    window.on_add_action(move |group_id| add_action(group_id, s.clone()));
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
