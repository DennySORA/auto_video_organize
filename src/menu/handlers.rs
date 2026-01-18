use crate::component::{
    AutoMoveByType, ContactSheetGenerator, DuplicationChecker, OrphanFileMover, VideoEncoder,
    VideoRenamer,
};
use crate::config::Config;
use crate::pause;
use anyhow::Result;
use console::{Term, style};
use rust_i18n::t;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub fn run_video_encoder(
    term: &Term,
    shutdown_signal: &Arc<AtomicBool>,
    config: &Config,
) -> Result<()> {
    let encoder = VideoEncoder::new(config.clone(), Arc::clone(shutdown_signal));

    if let Err(e) = encoder.run() {
        eprintln!("{} {}", style(t!("main_menu.error_prefix")).red().bold(), e);
    }

    pause(term)?;
    Ok(())
}

pub fn run_duplication_checker(
    term: &Term,
    shutdown_signal: &Arc<AtomicBool>,
    config: &Config,
) -> Result<()> {
    let checker = DuplicationChecker::new(config.clone(), Arc::clone(shutdown_signal));

    if let Err(e) = checker.run() {
        eprintln!("{} {}", style(t!("main_menu.error_prefix")).red().bold(), e);
    }

    pause(term)?;
    Ok(())
}

pub fn run_contact_sheet_generator(
    term: &Term,
    shutdown_signal: &Arc<AtomicBool>,
    config: &Config,
) -> Result<()> {
    let generator = ContactSheetGenerator::new(config.clone(), Arc::clone(shutdown_signal));

    if let Err(e) = generator.run() {
        eprintln!("{} {}", style(t!("main_menu.error_prefix")).red().bold(), e);
    }

    pause(term)?;
    Ok(())
}

pub fn run_auto_move_by_type(term: &Term, shutdown_signal: &Arc<AtomicBool>) -> Result<()> {
    let config = Config::new()?;
    let mover = AutoMoveByType::new(config, Arc::clone(shutdown_signal));

    if let Err(e) = mover.run() {
        eprintln!("{} {}", style(t!("main_menu.error_prefix")).red().bold(), e);
    }

    pause(term)?;
    Ok(())
}

pub fn run_orphan_file_mover(term: &Term, shutdown_signal: &Arc<AtomicBool>) -> Result<()> {
    let mover = OrphanFileMover::new(Arc::clone(shutdown_signal));

    if let Err(e) = mover.run() {
        eprintln!("{} {}", style(t!("main_menu.error_prefix")).red().bold(), e);
    }

    pause(term)?;
    Ok(())
}

pub fn run_video_renamer(term: &Term, shutdown_signal: &Arc<AtomicBool>) -> Result<()> {
    let config = Config::new()?;
    let renamer = VideoRenamer::new(config, Arc::clone(shutdown_signal));

    if let Err(e) = renamer.run() {
        eprintln!("{} {}", style(t!("main_menu.error_prefix")).red().bold(), e);
    }

    pause(term)?;
    Ok(())
}
