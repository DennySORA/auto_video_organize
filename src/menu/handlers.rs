use crate::component::{
    AutoMoveByType, ContactSheetGenerator, DuplicationChecker, OrphanFileMover, VideoEncoder,
    VideoRenamer,
};
use crate::config::Config;
use crate::pause;
use anyhow::Result;
use console::{Term, style};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub fn run_video_encoder(term: &Term, shutdown_signal: &Arc<AtomicBool>) -> Result<()> {
    let config = Config::new()?;
    let encoder = VideoEncoder::new(config, Arc::clone(shutdown_signal));

    if let Err(e) = encoder.run() {
        eprintln!("{} {}", style("錯誤:").red().bold(), e);
    }

    pause(term)?;
    Ok(())
}

pub fn run_duplication_checker(term: &Term, shutdown_signal: &Arc<AtomicBool>) -> Result<()> {
    let checker = DuplicationChecker::new(Arc::clone(shutdown_signal));

    if let Err(e) = checker.run() {
        eprintln!("{} {}", style("錯誤:").red().bold(), e);
    }

    pause(term)?;
    Ok(())
}

pub fn run_contact_sheet_generator(term: &Term, shutdown_signal: &Arc<AtomicBool>) -> Result<()> {
    let config = Config::new()?;
    let generator = ContactSheetGenerator::new(config, Arc::clone(shutdown_signal));

    if let Err(e) = generator.run() {
        eprintln!("{} {}", style("錯誤:").red().bold(), e);
    }

    pause(term)?;
    Ok(())
}

pub fn run_auto_move_by_type(term: &Term, shutdown_signal: &Arc<AtomicBool>) -> Result<()> {
    let config = Config::new()?;
    let mover = AutoMoveByType::new(config, Arc::clone(shutdown_signal));

    if let Err(e) = mover.run() {
        eprintln!("{} {}", style("錯誤:").red().bold(), e);
    }

    pause(term)?;
    Ok(())
}

pub fn run_orphan_file_mover(term: &Term, shutdown_signal: &Arc<AtomicBool>) -> Result<()> {
    let mover = OrphanFileMover::new(Arc::clone(shutdown_signal));

    if let Err(e) = mover.run() {
        eprintln!("{} {}", style("錯誤:").red().bold(), e);
    }

    pause(term)?;
    Ok(())
}

pub fn run_video_renamer(term: &Term, shutdown_signal: &Arc<AtomicBool>) -> Result<()> {
    let config = Config::new()?;
    let renamer = VideoRenamer::new(config, Arc::clone(shutdown_signal));

    if let Err(e) = renamer.run() {
        eprintln!("{} {}", style("錯誤:").red().bold(), e);
    }

    pause(term)?;
    Ok(())
}
