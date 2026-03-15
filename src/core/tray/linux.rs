use super::icon;
use super::{TrayController, TrayService};
use crate::core::app_command::AppCommand;
use crate::core::assets::asset_theme;
use crate::core::display::active_output_target;
use crate::core::preferences::AppPreferences;
use gtk::glib;
use gtk::prelude::*;
use log::{error, info};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

const OPEN_LAUNCHER_ID: &str = "tray.open-launcher";
const QUIT_ID: &str = "tray.quit";
const TRAY_ID: &str = "gamut.tray";
const TRAY_TOOLTIP: &str = "Gamut";

pub(super) fn start(
    command_tx: Sender<AppCommand>,
    preferences: AppPreferences,
) -> Result<(TrayService, TrayController), Box<dyn std::error::Error>> {
    info!("starting tray service");
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let (preferences_tx, preferences_rx) = mpsc::channel();

    let thread = thread::Builder::new()
        .name("gamut-tray".to_string())
        .spawn(move || run_tray_loop(ready_tx, command_tx, preferences, preferences_rx))?;

    match ready_rx.recv() {
        Ok(Ok(())) => {
            info!("tray service ready");
            Ok((
                TrayService::from_thread(thread),
                TrayController {
                    sender: preferences_tx,
                },
            ))
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
    preferences_rx: mpsc::Receiver<AppPreferences>,
) {
    if let Err(error) = run_tray_loop_inner(&ready_tx, command_tx, preferences, preferences_rx) {
        let _ = ready_tx.send(Err(error.to_string()));
        error!("{error}");
    }
}

fn run_tray_loop_inner(
    ready_tx: &mpsc::SyncSender<Result<(), String>>,
    command_tx: Sender<AppCommand>,
    preferences: AppPreferences,
    preferences_rx: mpsc::Receiver<AppPreferences>,
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

    install_theme_refresh(Rc::clone(&tray_icon), preferences, preferences_rx);

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
    let separator = PredefinedMenuItem::separator();
    let quit = MenuItem::with_id(MenuId::new(QUIT_ID), "Quit", true, None);

    menu.append_items(&[&open_launcher, &separator, &quit])?;

    Ok(menu)
}

fn install_theme_refresh(
    tray_icon: Rc<TrayIcon>,
    preferences: AppPreferences,
    preferences_rx: mpsc::Receiver<AppPreferences>,
) {
    let preferences = Rc::new(RefCell::new(preferences));
    #[allow(deprecated)]
    let (preferences_tx, preferences_updates) =
        glib::MainContext::channel::<AppPreferences>(glib::Priority::default());

    let tray_icon_for_updates = Rc::clone(&tray_icon);
    let preferences_for_updates = Rc::clone(&preferences);
    preferences_updates.attach(None, move |next_preferences| {
        *preferences_for_updates.borrow_mut() = next_preferences;
        refresh_tray_icon(&tray_icon_for_updates, &preferences_for_updates.borrow());
        glib::ControlFlow::Continue
    });

    if let Some(settings) = gtk::Settings::default() {
        let tray_icon_for_dark_notify = Rc::clone(&tray_icon);
        let preferences_for_dark_notify = Rc::clone(&preferences);
        settings.connect_gtk_application_prefer_dark_theme_notify(move |_| {
            refresh_tray_icon(
                &tray_icon_for_dark_notify,
                &preferences_for_dark_notify.borrow(),
            );
        });

        let tray_icon_for_theme_notify = Rc::clone(&tray_icon);
        let preferences_for_theme_notify = Rc::clone(&preferences);
        settings.connect_gtk_theme_name_notify(move |_| {
            refresh_tray_icon(
                &tray_icon_for_theme_notify,
                &preferences_for_theme_notify.borrow(),
            );
        });
    }

    thread::spawn(move || {
        while let Ok(next_preferences) = preferences_rx.recv() {
            if preferences_tx.send(next_preferences).is_err() {
                break;
            }
        }
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
        QUIT_ID => dispatch(command_tx, AppCommand::Quit),
        _ => {}
    }
}

fn show_command() -> AppCommand {
    AppCommand::ShowLauncher {
        target_output: active_output_target(),
    }
}

#[cfg(test)]
mod tests {
    use super::show_command;
    use crate::core::app_command::AppCommand;

    #[test]
    fn tray_show_command_keeps_show_launcher_variant() {
        assert!(matches!(show_command(), AppCommand::ShowLauncher { .. }));
    }
}

fn dispatch(command_tx: &Sender<AppCommand>, command: AppCommand) {
    if let Err(error) = command_tx.send(command.clone()) {
        error!("failed to dispatch tray command {:?}: {error}", command);
    }
}

fn refresh_tray_icon(tray_icon: &TrayIcon, preferences: &AppPreferences) {
    let next_theme = asset_theme(&preferences.appearance);

    if let Err(error) =
        icon::load(next_theme).and_then(|icon| tray_icon.set_icon(Some(icon)).map_err(Into::into))
    {
        error!("failed to update tray icon theme: {error}");
    }
}
