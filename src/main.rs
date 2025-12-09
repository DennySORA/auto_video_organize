use anyhow::Result;
use auto_video_organize::init;
use auto_video_organize::menu::show_main_menu;
use auto_video_organize::signal::setup_shutdown_signal;
use console::{Term, style};
use log::{info, warn};

fn main() -> Result<()> {
    init::init();
    let term = Term::stdout();
    let shutdown_signal = setup_shutdown_signal();

    loop {
        match show_main_menu(&term, &shutdown_signal) {
            Ok(true) => {}
            Ok(false) => {
                term.clear_screen()?;
                println!("\n{}", style("感謝使用，再見！").green().bold());
                info!("程式正常結束");
                break;
            }
            Err(e) => {
                warn!("程式錯誤: {e}");
                eprintln!("{} {}", style("錯誤:").red().bold(), e);
                break;
            }
        }
    }

    Ok(())
}
