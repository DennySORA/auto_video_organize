use crate::config::save::save_settings;
use crate::config::types::{
    Config, ContactSheetOutputMode, Language, PostEncodeAction, VideoEncoderSettings,
};
use crate::menu::handlers::{
    run_auto_move_by_type, run_contact_sheet_generator, run_duplication_checker,
    run_orphan_file_mover, run_video_encoder, run_video_renamer,
};
use anyhow::Result;
use console::{Term, style};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use rust_i18n::t;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;

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
    loop {
        term.clear_screen()?;
        println!("{}", style(t!("settings.encoder.title")).cyan().bold());
        println!("{}", style(t!("common.esc_hint")).dim());
        println!();
        render_encoder_overview(config);
        println!();

        let back = t!("settings.back").to_string();
        let options = vec!["檔案處理設定".to_string(), "轉檔數量設定".to_string(), back];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("選擇設定分類")
            .items(&options)
            .default(0)
            .interact_on_opt(term)?;

        match selection {
            Some(0) => show_encoder_file_settings(term, config)?,
            Some(1) => show_encoder_parallel_settings(term, config)?,
            Some(2) | None => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn format_initial_limit(settings: &VideoEncoderSettings) -> String {
    match settings.initial_max_parallel {
        Some(v) => v.to_string(),
        None => "自動 (CPU 1/4)".to_string(),
    }
}

fn format_max_limit(settings: &VideoEncoderSettings) -> String {
    match settings.max_parallel {
        Some(v) => v.to_string(),
        None => "無限制".to_string(),
    }
}

fn render_encoder_overview(config: &Config) {
    let enc = &config.settings.video_encoder;
    println!(
        "{:<18} {}",
        style("轉檔後處理").dim(),
        enc.post_encode_action
    );
    println!(
        "{:<18} {}",
        style("初始最大數").dim(),
        format_initial_limit(enc)
    );
    println!(
        "{:<18} {}",
        style("最大同時數").dim(),
        format_max_limit(enc)
    );
}

fn show_encoder_file_settings(term: &Term, config: &mut Config) -> Result<()> {
    term.clear_screen()?;
    println!("{}", style("檔案處理設定").cyan().bold());
    println!("{}", style(t!("common.esc_hint")).dim());
    println!();
    render_encoder_overview(config);
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
        .with_prompt("選擇轉檔後處理")
        .items(&items)
        .default(default_index)
        .interact_on_opt(term)?;

    if let Some(idx) = selection {
        let selected_action = actions[idx];
        if selected_action != config.settings.video_encoder.post_encode_action {
            config.settings.video_encoder.post_encode_action = selected_action;
            save_settings(&config.settings)?;
            println!("\n{}", style(t!("settings.saved")).green());
            thread::sleep(Duration::from_secs(1));
        }
    }

    Ok(())
}

fn show_encoder_parallel_settings(term: &Term, config: &mut Config) -> Result<()> {
    term.clear_screen()?;
    println!("{}", style("轉檔數量設定").cyan().bold());
    println!("{}", style("左側為項目，右側為目前值").dim());
    println!();
    render_encoder_overview(config);
    println!();

    // 初始最大轉檔數
    let current_initial = config
        .settings
        .video_encoder
        .initial_max_parallel
        .map(|v| v as i64)
        .unwrap_or(-1);
    let initial_limit: i64 = Input::new()
        .with_prompt("初始最大轉檔數（-1 = CPU 1/4）")
        .default(current_initial)
        .interact_text()?;
    config.settings.video_encoder.initial_max_parallel = if initial_limit <= 0 {
        None
    } else {
        Some(initial_limit as usize)
    };

    // 最大同時轉檔數
    let current_max = config
        .settings
        .video_encoder
        .max_parallel
        .map(|v| v as i64)
        .unwrap_or(-1);
    let max_limit: i64 = Input::new()
        .with_prompt("最大同時轉檔數（-1 = 無限制）")
        .default(current_max)
        .interact_text()?;
    config.settings.video_encoder.max_parallel = if max_limit <= 0 {
        None
    } else {
        Some(max_limit as usize)
    };

    save_settings(&config.settings)?;
    println!("\n{}", style(t!("settings.saved")).green());
    thread::sleep(Duration::from_secs(1));

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
