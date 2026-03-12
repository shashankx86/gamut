use super::TrayService;
use super::icon;
use crate::core::app_command::AppCommand;
use crate::core::display_target::active_output_name;
use gtk::glib;
use log::{error, info};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

const OPEN_LAUNCHER_ID: &str = "tray.open-launcher";
const OPEN_PREFERENCES_ID: &str = "tray.open-preferences";
const QUIT_ID: &str = "tray.quit";
const TRAY_ID: &str = "gamut.tray";
const TRAY_TOOLTIP: &str = "Gamut";

pub(super) fn start(
    command_tx: Sender<AppCommand>,
) -> Result<TrayService, Box<dyn std::error::Error>> {
    info!("starting tray service");
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);

    let thread = thread::Builder::new()
        .name("gamut-tray".to_string())
        .spawn(move || run_tray_loop(ready_tx, command_tx))?;

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

fn run_tray_loop(ready_tx: mpsc::SyncSender<Result<(), String>>, command_tx: Sender<AppCommand>) {
    if let Err(error) = run_tray_loop_inner(&ready_tx, command_tx) {
        let _ = ready_tx.send(Err(error.to_string()));
        error!("{error}");
    }
}

fn run_tray_loop_inner(
    ready_tx: &mpsc::SyncSender<Result<(), String>>,
    command_tx: Sender<AppCommand>,
) -> Result<(), Box<dyn std::error::Error>> {
    gtk::init()?;

    install_event_handlers(command_tx);

    let tray_menu = build_tray_menu()?;
    let tray_icon = TrayIconBuilder::new()
        .with_id(TRAY_ID)
        .with_menu(Box::new(tray_menu))
        .with_menu_on_left_click(false)
        .with_tooltip(TRAY_TOOLTIP)
        .with_icon(icon::load()?)
        .build()?;

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
