mod gui;
mod lib_thread;
use std::error::Error;

use lib_thread::{FromLibReciever, ToLibSender};
use renamer_lib::ActionGroup;

struct Renamer {
    sender: ToLibSender,
    reciever: FromLibReciever,
    action_groups: Vec<ActionGroup>,
}

impl Renamer {
    fn new(sender: ToLibSender, reciever: FromLibReciever) -> Self {
        Self {
            sender,
            reciever,
            action_groups: Vec::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    env_logger::init();

    let (lib_handle, sender, reciever) = lib_thread::setup();
    eframe::run_native(
        "Renamer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            Ok(Box::new(Renamer::new(sender, reciever)))
        }),
    )?;
    let _ = lib_handle.join();
    Ok(())
}
