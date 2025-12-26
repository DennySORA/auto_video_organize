#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "en-US");

pub mod component;
pub mod config;
pub mod init;
pub mod menu;
pub mod signal;
pub mod tools;

use anyhow::Result;
use console::{Term, style};
use rust_i18n::t;

pub fn pause(term: &Term) -> Result<()> {
    println!("\n{}", style(t!("common.press_enter")).dim());
    term.read_line()?;
    Ok(())
}
