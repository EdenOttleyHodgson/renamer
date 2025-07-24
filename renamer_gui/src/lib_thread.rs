use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
};

use crate::{SendableErr, slint_generatedRenamerWindow::RenamerWindow, state::RenamerState};
use log::trace;
use renamer_lib::{ActionGroup, error::ActionError, report::Report};

#[derive(Debug)]
pub enum ToLibMessage {
    ExecuteActions(Vec<ActionGroup>),
    Cleanup,
}
#[derive(Debug)]
pub enum FromLibMessage {
    SuccessfulActions(Vec<Report>),
    UnsuccessfulActions(Vec<SendableErr>),
}

impl FromLibMessage {}
pub type ToLibSender = mpsc::Sender<ToLibMessage>;
type ToLibReciever = mpsc::Receiver<ToLibMessage>;
type FromLibSender = mpsc::Sender<FromLibMessage>;
pub type FromLibReciever = mpsc::Receiver<FromLibMessage>;
static THREADS: usize = 8;

struct LibWrapper {
    sender: FromLibSender,
    receiver: ToLibReciever,
}
impl LibWrapper {
    fn new(sender: FromLibSender, receiver: ToLibReciever) -> Self {
        Self { sender, receiver }
    }
    fn event_loop(mut self) {
        loop {
            match self.receiver.recv() {
                Ok(msg) => {
                    if matches!(msg, ToLibMessage::Cleanup) {
                        break;
                    } else {
                        self.handle_message(msg)
                    }
                }
                Err(e) => {
                    log::error!("{e}");
                    break;
                }
            }
        }
    }
    fn handle_message(&mut self, msg: ToLibMessage) {
        match msg {
            ToLibMessage::ExecuteActions(act_groups) => {
                self.handle_execute_actions(act_groups);
            }
            ToLibMessage::Cleanup => unreachable!(),
        }
    }

    fn handle_execute_actions(&mut self, act_groups: Vec<ActionGroup>) {
        let results = act_groups.into_iter().map(|x| x.execute()).flatten();

        let (successes, errors) =
            results
                .into_iter()
                .fold((Vec::new(), Vec::new()), |(mut succs, mut errs), x| {
                    match x {
                        Ok(v) => succs.push(v),
                        Err(e) => errs.push(e),
                    };
                    (succs, errs)
                });
        if !successes.is_empty() {
            let _ = self
                .sender
                .send(FromLibMessage::SuccessfulActions(successes))
                .inspect_err(|x| log::error!("{x}"));
        };
        if !errors.is_empty() {
            let _ = self
                .sender
                .send(FromLibMessage::UnsuccessfulActions(errors))
                .inspect_err(|x| log::error!("{x}"));
        }
    }
}

pub fn setup() -> (JoinHandle<()>, ToLibSender, FromLibReciever) {
    let (lib_tx, gui_rx) = mpsc::channel::<FromLibMessage>();
    let (gui_tx, lib_rx) = mpsc::channel::<ToLibMessage>();
    let wrapper = LibWrapper::new(lib_tx, lib_rx);
    (thread::spawn(move || wrapper.event_loop()), gui_tx, gui_rx)
}

pub fn handle_gui_messages(gui_rx: FromLibReciever, state: RenamerState) {
    loop {
        match gui_rx.recv() {
            Ok(msg) => state.write().handle_message(msg),
            Err(e) => {
                log::error!("{e}");
                break;
            }
        }
    }
}
