mod config;
mod init;
mod menu;
mod component;
mod tools;

use crate::menu::show_main_menu;
use anyhow::Result;
use console::{Term, style};
use log::{info, warn};

fn main() -> Result<()> {
    init::init();
    let term = Term::stdout();

    loop {
        match show_main_menu(&term) {
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

fn pause(term: &Term) -> Result<()> {
    println!("\n{}", style("按 Enter 繼續...").dim());
    term.read_line()?;
    Ok(())
}
