mod callbacks;
mod lib_thread;
mod state;
use std::error::Error;

pub(crate) type SendableErr = Box<dyn Error + Send + Sync>;
use callbacks::set_callbacks;
use slint::ComponentHandle;
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter(Some("renamer_gui"), log::LevelFilter::Trace)
        .init();
    log::debug!("Hello !");
    let window = RenamerWindow::new()?;
    let (state, lib_handle) = state::init_state(window.as_weak());

    set_callbacks(&window, state.clone());
    window.invoke_refresh_state();
    window.run()?;
    window.invoke_cleanup();

    let _ = lib_handle.join();
    Ok(())
}
