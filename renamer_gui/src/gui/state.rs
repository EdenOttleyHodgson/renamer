use std::{cell::OnceCell, sync::mpsc::Sender};

use renamer_lib::{Action, ActionGroup};

pub enum StateChange<'a> {
    RemoveActionGroup(&'a ActionGroup),
}
