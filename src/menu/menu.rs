use anyhow::Result;
use console::{Term, style};
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
pub fn show_main_menu(term: &Term) -> Result<bool> {
    term.clear_screen()?;

    println!("{}", style("=== 自動影片整理系統 ===").cyan().bold());

    let options = vec![ ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("請選擇功能")
        .items(&options)
        .default(0)
        .interact_on(term)?;

    match selection {
        6 => Ok(false),
        _ => unreachable!(),
    }
}
