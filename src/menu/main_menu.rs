use crate::config::save::save_settings;
use crate::config::types::{Config, Language};
use crate::menu::handlers::{
    run_auto_move_by_type, run_contact_sheet_generator, run_duplication_checker,
    run_orphan_file_mover, run_video_encoder, run_video_renamer,
};
use anyhow::Result;
use console::{Term, style};
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use rust_i18n::t;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub fn show_main_menu(
    term: &Term,
    shutdown_signal: &Arc<AtomicBool>,
    config: &mut Config,
) -> Result<bool> {
    term.clear_screen()?;

    println!("{}", style(t!("main_menu.title")).cyan().bold());

    let options = vec![
        t!("main_menu.opt_encoder"),
        t!("main_menu.opt_dedup"),
        t!("main_menu.opt_contact"),
        t!("main_menu.opt_auto_move"),
        t!("main_menu.opt_orphan"),
        t!("main_menu.opt_renamer"),
        t!("main_menu.opt_language"),
        t!("main_menu.exit"),
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(t!("main_menu.prompt"))
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
        5 => {
            run_video_renamer(term, shutdown_signal)?;
            Ok(true)
        }
        6 => {
            show_language_menu(term, config)?;
            Ok(true)
        }
        7 => Ok(false),
        _ => unreachable!(),
    }
}

fn show_language_menu(term: &Term, config: &mut Config) -> Result<()> {
    term.clear_screen()?;

    let languages = vec![
        Language::EnUs,
        Language::ZhTw,
        Language::ZhCn,
        Language::JaJp,
    ];

    let items: Vec<String> = languages.iter().map(|l: &Language| l.to_string()).collect();

    let default_index = languages
        .iter()
        .position(|&l| l == config.settings.language)
        .unwrap_or(0);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(t!("main_menu.opt_language"))
        .items(&items)
        .default(default_index)
        .interact_on(term)?;

    let selected_lang = languages[selection];

    if selected_lang != config.settings.language {
        config.settings.language = selected_lang;
        rust_i18n::set_locale(selected_lang.as_str());
        save_settings(&config.settings)?;
        println!(
            "\n{} {}",
            style("Language changed to:").green(),
            selected_lang
        );
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}
