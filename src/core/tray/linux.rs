use super::TrayService;
use super::icon;
use crate::core::app_command::AppCommand;
use crate::core::assets::asset_theme;
use crate::core::display_target::active_output_name;
use crate::core::preferences::{AppPreferences, load_preferences};
use gtk::glib;
use log::{error, info};
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

const OPEN_LAUNCHER_ID: &str = "tray.open-launcher";
const OPEN_PREFERENCES_ID: &str = "tray.open-preferences";
const QUIT_ID: &str = "tray.quit";
const TRAY_ID: &str = "gamut.tray";
const TRAY_TOOLTIP: &str = "Gamut";

pub(super) fn start(
    command_tx: Sender<AppCommand>,
    preferences: AppPreferences,
) -> Result<TrayService, Box<dyn std::error::Error>> {
    info!("starting tray service");
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);

    let thread = thread::Builder::new()
        .name("gamut-tray".to_string())
        .spawn(move || run_tray_loop(ready_tx, command_tx, preferences))?;

    match ready_rx.recv() {
        Ok(Ok(())) => {
            info!("tray service ready");
            Ok(TrayService::from_thread(thread))
        }
        Ok(Err(error)) => Err(error.into()),
        Err(error) => {
            Err(format!("tray service exited before it finished starting: {error}").into())
        }
    }
}

fn run_tray_loop(
    ready_tx: mpsc::SyncSender<Result<(), String>>,
    command_tx: Sender<AppCommand>,
    preferences: AppPreferences,
) {
    if let Err(error) = run_tray_loop_inner(&ready_tx, command_tx, preferences) {
        let _ = ready_tx.send(Err(error.to_string()));
        error!("{error}");
    }
}

fn run_tray_loop_inner(
    ready_tx: &mpsc::SyncSender<Result<(), String>>,
    command_tx: Sender<AppCommand>,
    preferences: AppPreferences,
) -> Result<(), Box<dyn std::error::Error>> {
    gtk::init()?;

    install_event_handlers(command_tx);

    let tray_menu = build_tray_menu()?;
    let initial_theme = asset_theme(&preferences.appearance);
    let tray_icon = Rc::new(
        TrayIconBuilder::new()
            .with_id(TRAY_ID)
            .with_menu(Box::new(tray_menu))
            .with_menu_on_left_click(false)
            .with_tooltip(TRAY_TOOLTIP)
            .with_icon(icon::load(initial_theme)?)
            .build()?,
    );

    install_theme_refresh(Rc::clone(&tray_icon));

    ready_tx
        .send(Ok(()))
        .map_err(|error| format!("failed to report tray startup status: {error}"))?;

    let main_loop = glib::MainLoop::new(None, false);
    let _tray_icon = tray_icon;
    main_loop.run();

    Ok(())
}

fn build_tray_menu() -> Result<Menu, Box<dyn std::error::Error>> {
    let menu = Menu::new();
    let open_launcher =
        MenuItem::with_id(MenuId::new(OPEN_LAUNCHER_ID), "Open Launcher", true, None);
    let open_preferences =
        MenuItem::with_id(MenuId::new(OPEN_PREFERENCES_ID), "Preferences", true, None);
    let separator = PredefinedMenuItem::separator();
    let quit = MenuItem::with_id(MenuId::new(QUIT_ID), "Quit", true, None);

    menu.append_items(&[&open_launcher, &open_preferences, &separator, &quit])?;

    Ok(menu)
}

fn install_theme_refresh(tray_icon: Rc<TrayIcon>) {
    let mut last_theme = asset_theme(&load_preferences().appearance);

    glib::timeout_add_local(Duration::from_millis(900), move || {
        let next_theme = asset_theme(&load_preferences().appearance);

        if next_theme != last_theme {
            match icon::load(next_theme)
                .and_then(|icon| tray_icon.set_icon(Some(icon)).map_err(Into::into))
            {
                Ok(()) => last_theme = next_theme,
                Err(error) => error!("failed to update tray icon theme: {error}"),
            }
        }

        glib::ControlFlow::Continue
    });
}

fn install_event_handlers(command_tx: Sender<AppCommand>) {
    let tray_tx = command_tx.clone();
    TrayIconEvent::set_event_handler(Some(move |event| handle_tray_event(event, &tray_tx)));

    MenuEvent::set_event_handler(Some(move |event| handle_menu_event(event, &command_tx)));
}

fn handle_tray_event(event: TrayIconEvent, command_tx: &Sender<AppCommand>) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        dispatch(command_tx, show_command());
    }
}

fn handle_menu_event(event: MenuEvent, command_tx: &Sender<AppCommand>) {
    match event.id().as_ref() {
        OPEN_LAUNCHER_ID => dispatch(command_tx, show_command()),
        OPEN_PREFERENCES_ID => dispatch(command_tx, AppCommand::OpenPreferences),
        QUIT_ID => dispatch(command_tx, AppCommand::Quit),
        _ => {}
    }
}

fn show_command() -> AppCommand {
    AppCommand::ShowLauncher {
        target_output: active_output_name(),
    }
}

fn dispatch(command_tx: &Sender<AppCommand>, command: AppCommand) {
    if let Err(error) = command_tx.send(command.clone()) {
        error!("failed to dispatch tray command {:?}: {error}", command);
    }
}
