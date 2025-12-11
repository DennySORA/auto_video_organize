pub mod component;
pub mod config;
pub mod init;
pub mod menu;
pub mod signal;
pub mod tools;

use anyhow::Result;
use console::{Term, style};

pub fn pause(term: &Term) -> Result<()> {
    println!("\n{}", style("按 Enter 繼續...").dim());
    term.read_line()?;
    Ok(())
}
