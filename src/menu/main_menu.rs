use crate::menu::handlers::{
    run_auto_move_by_type, run_contact_sheet_generator, run_duplication_checker,
    run_orphan_file_mover, run_video_encoder,
};
use anyhow::Result;
use console::{Term, style};
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub fn show_main_menu(term: &Term, shutdown_signal: &Arc<AtomicBool>) -> Result<bool> {
    term.clear_screen()?;

    println!("{}", style("=== 自動影片整理系統 ===").cyan().bold());

    let options = vec![
        "影片重新編碼",
        "資料分析紀錄與去重",
        "影片預覽圖生成",
        "自動依類型整理檔案",
        "移動孤立檔案（無對應檔案）",
        "離開",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("請選擇功能")
        .items(&options)
        .default(0)
        .interact_on(term)?;

    match selection {
        0 => {
            run_video_encoder(term, shutdown_signal)?;
            Ok(true)
        }
        1 => {
            run_duplication_checker(term, shutdown_signal)?;
            Ok(true)
        }
        2 => {
            run_contact_sheet_generator(term, shutdown_signal)?;
            Ok(true)
        }
        3 => {
            run_auto_move_by_type(term, shutdown_signal)?;
            Ok(true)
        }
        4 => {
            run_orphan_file_mover(term, shutdown_signal)?;
            Ok(true)
        }
        5 => Ok(false),
        _ => unreachable!(),
    }
}
