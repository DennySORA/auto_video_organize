use crate::config::save::save_settings;
use crate::config::types::{Config, ContactSheetOutputMode, Language, PostEncodeAction};
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
    println!("{}", style(t!("common.esc_hint")).dim());

    let options = vec![
        t!("main_menu.opt_encoder"),
        t!("main_menu.opt_dedup"),
        t!("main_menu.opt_contact"),
        t!("main_menu.opt_auto_move"),
        t!("main_menu.opt_orphan"),
        t!("main_menu.opt_renamer"),
        t!("main_menu.opt_settings"),
        t!("main_menu.exit"),
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(t!("main_menu.prompt"))
        .items(&options)
        .default(0)
        .interact_on_opt(term)?;

    match selection {
        Some(0) => {
            run_video_encoder(term, shutdown_signal, config)?;
            Ok(true)
        }
        Some(1) => {
            run_duplication_checker(term, shutdown_signal, config)?;
            Ok(true)
        }
        Some(2) => {
            run_contact_sheet_generator(term, shutdown_signal, config)?;
            Ok(true)
        }
        Some(3) => {
            run_auto_move_by_type(term, shutdown_signal, config)?;
            Ok(true)
        }
        Some(4) => {
            run_orphan_file_mover(term, shutdown_signal, config)?;
            Ok(true)
        }
        Some(5) => {
            run_video_renamer(term, shutdown_signal, config)?;
            Ok(true)
        }
        Some(6) => {
            show_settings_menu(term, config)?;
            Ok(true)
        }
        Some(7) => Ok(false),
        None => Ok(false), // ESC pressed - exit
        _ => unreachable!(),
    }
}

/// 設定選單
fn show_settings_menu(term: &Term, config: &mut Config) -> Result<()> {
    loop {
        term.clear_screen()?;

        println!("{}", style(t!("settings.title")).cyan().bold());
        println!("{}", style(t!("common.esc_hint")).dim());

        let options = vec![
            t!("settings.opt_encoder"),
            t!("settings.opt_contact_sheet"),
            t!("settings.opt_language"),
            t!("settings.back"),
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(t!("settings.prompt"))
            .items(&options)
            .default(0)
            .interact_on_opt(term)?;

        match selection {
            Some(0) => show_encoder_settings_menu(term, config)?,
            Some(1) => show_contact_sheet_settings_menu(term, config)?,
            Some(2) => show_language_menu(term, config)?,
            Some(3) | None => break, // ESC or back
            _ => unreachable!(),
        }
    }

    Ok(())
}

/// 影片轉檔設定選單
fn show_encoder_settings_menu(term: &Term, config: &mut Config) -> Result<()> {
    term.clear_screen()?;

    println!("{}", style(t!("settings.encoder.title")).cyan().bold());
    println!("{}", style(t!("common.esc_hint")).dim());

    // 顯示當前設定
    println!(
        "\n{} {}",
        style(t!("settings.encoder.current")).dim(),
        config.settings.video_encoder.post_encode_action
    );
    println!();

    let actions = [
        PostEncodeAction::None,
        PostEncodeAction::MoveOldToFinish,
        PostEncodeAction::MoveNewToFinish,
    ];

    let items: Vec<String> = vec![
        t!("settings.encoder.action_none").to_string(),
        t!("settings.encoder.action_move_old").to_string(),
        t!("settings.encoder.action_move_new").to_string(),
    ];

    let default_index = actions
        .iter()
        .position(|&a| a == config.settings.video_encoder.post_encode_action)
        .unwrap_or(0);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(t!("settings.encoder.prompt"))
        .items(&items)
        .default(default_index)
        .interact_on_opt(term)?;

    // ESC pressed - return without saving
    let Some(selection) = selection else {
        return Ok(());
    };

    let selected_action = actions[selection];

    if selected_action != config.settings.video_encoder.post_encode_action {
        config.settings.video_encoder.post_encode_action = selected_action;
        save_settings(&config.settings)?;
        println!(
            "\n{} {}",
            style(t!("settings.saved")).green(),
            selected_action
        );
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}

/// 縮圖產生設定選單
fn show_contact_sheet_settings_menu(term: &Term, config: &mut Config) -> Result<()> {
    term.clear_screen()?;

    println!(
        "{}",
        style(t!("settings.contact_sheet.title")).cyan().bold()
    );
    println!("{}", style(t!("common.esc_hint")).dim());

    // 顯示當前設定
    println!(
        "\n{} {}",
        style(t!("settings.contact_sheet.current")).dim(),
        config.settings.contact_sheet.output_mode
    );
    println!();

    let modes = [
        ContactSheetOutputMode::SubDirectory,
        ContactSheetOutputMode::SameDirectory,
    ];

    let items: Vec<String> = vec![
        t!("settings.contact_sheet.mode_sub_directory").to_string(),
        t!("settings.contact_sheet.mode_same_directory").to_string(),
    ];

    let default_index = modes
        .iter()
        .position(|&m| m == config.settings.contact_sheet.output_mode)
        .unwrap_or(0);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(t!("settings.contact_sheet.prompt"))
        .items(&items)
        .default(default_index)
        .interact_on_opt(term)?;

    // ESC pressed - return without saving
    let Some(selection) = selection else {
        return Ok(());
    };

    let selected_mode = modes[selection];

    if selected_mode != config.settings.contact_sheet.output_mode {
        config.settings.contact_sheet.output_mode = selected_mode;
        save_settings(&config.settings)?;
        println!(
            "\n{} {}",
            style(t!("settings.saved")).green(),
            selected_mode
        );
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}

/// 語言設定選單
fn show_language_menu(term: &Term, config: &mut Config) -> Result<()> {
    term.clear_screen()?;

    println!("{}", style(t!("settings.language.title")).cyan().bold());
    println!("{}", style(t!("common.esc_hint")).dim());

    let languages = [
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
        .with_prompt(t!("settings.language.prompt"))
        .items(&items)
        .default(default_index)
        .interact_on_opt(term)?;

    // ESC pressed - return without saving
    let Some(selection) = selection else {
        return Ok(());
    };

    let selected_lang = languages[selection];

    if selected_lang != config.settings.language {
        config.settings.language = selected_lang;
        rust_i18n::set_locale(selected_lang.as_str());
        save_settings(&config.settings)?;
        println!(
            "\n{} {}",
            style(t!("settings.saved")).green(),
            selected_lang
        );
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}
