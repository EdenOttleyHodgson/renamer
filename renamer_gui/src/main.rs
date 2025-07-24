mod callbacks;
mod lib_thread;
mod state;
use std::error::Error;
use std::fmt::Debug;

pub(crate) type SendableErr = Box<dyn Error + Send + Sync>;
use callbacks::set_callbacks;
use slint::{ComponentHandle, ModelRc, VecModel};
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter(Some("renamer_gui"), log::LevelFilter::Trace)
        .init();
    log::debug!("Hello !");
    let window = RenamerWindow::new()?;
    let (state, lib_handle) = state::init_state_debug(window.as_weak());

    set_callbacks(&window, state.clone());
    window.invoke_refresh_state();
    window.run()?;
    let _ = lib_handle.join();
    Ok(())
}
