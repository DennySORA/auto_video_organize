use anyhow::Result;
use auto_video_organize::config::types::Config;
use auto_video_organize::init;
use auto_video_organize::menu::show_main_menu;
use auto_video_organize::signal::setup_shutdown_signal;
use console::{Term, style};
use log::{info, warn};
use rust_i18n::t;

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "en-US");

fn main() -> Result<()> {
    init::init();
    let term = Term::stdout();
    let shutdown_signal = setup_shutdown_signal();

    // Load config and set locale
    let mut config = Config::new()?;
    rust_i18n::set_locale(config.settings.language.as_str());

    loop {
        // We pass the config to show_main_menu so it can update settings
        match show_main_menu(&term, &shutdown_signal, &mut config) {
            Ok(true) => {}
            Ok(false) => {
                term.clear_screen()?;
                println!("\n{}", style(t!("main_menu.goodbye")).green().bold());
                info!("Program exited normally");
                break;
            }
            Err(e) => {
                warn!("Program error: {e}");
                eprintln!("{} {}", style(t!("main_menu.error_prefix")).red().bold(), e);
                break;
            }
        }
    }

    Ok(())
}
