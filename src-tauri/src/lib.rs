use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::{NewWindowResponse, WebviewWindowBuilder},
    AppHandle, Manager, PhysicalPosition, PhysicalSize, Position, RunEvent, Size, WebviewUrl,
    WebviewWindow, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const WHATSAPP_URL: &str = "https://web.whatsapp.com";
const WINDOW_LABEL: &str = "main";
const TOGGLE_MENU_ID: &str = "toggle-window";
const HIDE_MENU_ID: &str = "minimize-to-tray";
const AUTOSTART_MENU_ID: &str = "launch-at-login";
const NOTIFICATIONS_MENU_ID: &str = "desktop-notifications";
const QUIT_MENU_ID: &str = "quit";

const NOTIFICATION_BRIDGE: &str = include_str!("notification_bridge.js");

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct WindowGeometry {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SavedPreferences {
    #[serde(default)]
    geometry: Option<WindowGeometry>,
    #[serde(default = "default_true")]
    notifications_enabled: bool,
}

impl Default for SavedPreferences {
    fn default() -> Self {
        Self {
            geometry: None,
            notifications_enabled: true,
        }
    }
}

fn default_true() -> bool {
    true
}

struct PreferencesStore {
    path: PathBuf,
    value: Mutex<SavedPreferences>,
}

impl PreferencesStore {
    fn load(path: PathBuf) -> Self {
        let value = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<SavedPreferences>(&bytes).ok())
            .unwrap_or_default();
        Self {
            path,
            value: Mutex::new(value),
        }
    }

    fn snapshot(&self) -> SavedPreferences {
        self.value
            .lock()
            .map(|value| value.clone())
            .unwrap_or_default()
    }

    fn update(&self, update: impl FnOnce(&mut SavedPreferences)) {
        if let Ok(mut value) = self.value.lock() {
            update(&mut value);
            if let Some(parent) = self.path.parent() {
                if let Err(error) = fs::create_dir_all(parent) {
                    eprintln!("Could not create preferences directory: {error}");
                    return;
                }
            }
            match serde_json::to_vec_pretty(&*value) {
                Ok(bytes) => {
                    if let Err(error) = fs::write(&self.path, bytes) {
                        eprintln!("Could not save preferences: {error}");
                    }
                }
                Err(error) => eprintln!("Could not serialize preferences: {error}"),
            }
        }
    }
}

fn toggle_shortcut() -> Shortcut {
    #[cfg(target_os = "macos")]
    {
        Shortcut::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyW)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyW)
    }
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_window(window: &WebviewWindow) {
    save_window_geometry(window, &window.app_handle());
    let _ = window.hide();
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        let visible = window.is_visible().unwrap_or(false);
        let minimized = window.is_minimized().unwrap_or(false);
        if visible && !minimized {
            hide_window(&window);
        } else {
            show_window(app);
        }
    }
}

fn save_window_geometry(window: &WebviewWindow, app: &AppHandle) {
    let (Ok(position), Ok(size)) = (window.outer_position(), window.outer_size()) else {
        return;
    };
    let store = app.state::<PreferencesStore>();
    store.update(|preferences| {
        preferences.geometry = Some(WindowGeometry {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        });
    });
}

fn build_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let preferences = app.state::<PreferencesStore>().snapshot();
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);

    let open = MenuItem::with_id(app, TOGGLE_MENU_ID, "Open WhatsApp", true, None::<&str>)?;
    let minimize = MenuItem::with_id(app, HIDE_MENU_ID, "Minimize to tray", true, None::<&str>)?;
    let separator_one = PredefinedMenuItem::separator(app)?;
    let autostart = CheckMenuItem::with_id(
        app,
        AUTOSTART_MENU_ID,
        "Launch at login",
        true,
        autostart_enabled,
        None::<&str>,
    )?;
    let notifications = CheckMenuItem::with_id(
        app,
        NOTIFICATIONS_MENU_ID,
        "Desktop notifications",
        true,
        preferences.notifications_enabled,
        None::<&str>,
    )?;
    let separator_two = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, QUIT_MENU_ID, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &open,
            &minimize,
            &separator_one,
            &autostart,
            &notifications,
            &separator_two,
            &quit,
        ],
    )?;

    let autostart_item = autostart.clone();
    let notifications_item = notifications.clone();
    let mut builder = TrayIconBuilder::new()
        .tooltip("WhatsApp Lite")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            TOGGLE_MENU_ID => show_window(app),
            HIDE_MENU_ID => {
                if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
                    hide_window(&window);
                }
            }
            AUTOSTART_MENU_ID => {
                let manager = app.autolaunch();
                let currently_enabled = manager.is_enabled().unwrap_or(false);
                let new_state = !currently_enabled;
                let result = if new_state {
                    manager.enable()
                } else {
                    manager.disable()
                };
                match result {
                    Ok(()) => {
                        let _ = autostart_item.set_checked(new_state);
                    }
                    Err(error) => eprintln!("Could not update launch-at-login setting: {error}"),
                }
            }
            NOTIFICATIONS_MENU_ID => {
                let store = app.state::<PreferencesStore>();
                let new_state = !store.snapshot().notifications_enabled;
                store.update(|preferences| preferences.notifications_enabled = new_state);
                let _ = notifications_item.set_checked(new_state);
                if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
                    let script = format!(
                        "window.__WHATSAPP_LITE_NOTIFICATIONS_ENABLED__ = {};",
                        if new_state { "true" } else { "false" }
                    );
                    let _ = window.eval(&script);
                }
            }
            QUIT_MENU_ID => {
                if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
                    save_window_geometry(&window, app);
                }
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => show_window(tray.app_handle()),
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => toggle_window(tray.app_handle()),
            _ => {}
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    let _tray = builder.build(app)?;
    Ok(())
}

pub fn run() {
    let toggle = toggle_shortcut();
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_window(app);
                    }
                })
                .build(),
        )
        .setup(move |app| {
            app.global_shortcut().register(toggle)?;

            let preferences_path = app.path().app_config_dir()?.join("preferences.json");
            app.manage(PreferencesStore::load(preferences_path));
            let preferences = app.state::<PreferencesStore>().snapshot();
            let launched_by_autostart = std::env::args().any(|arg| arg == "--autostart");
            let initial_script = NOTIFICATION_BRIDGE.replace(
                "__WHATSAPP_LITE_NOTIFICATIONS_ENABLED__ = true;",
                if preferences.notifications_enabled {
                    "__WHATSAPP_LITE_NOTIFICATIONS_ENABLED__ = true;"
                } else {
                    "__WHATSAPP_LITE_NOTIFICATIONS_ENABLED__ = false;"
                },
            );

            let url = WHATSAPP_URL
                .parse()
                .expect("WhatsApp URL must be a valid HTTPS URL");
            let window = WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::External(url))
                .title("WhatsApp Lite")
                .inner_size(1200.0, 800.0)
                .min_inner_size(800.0, 600.0)
                .resizable(true)
                .theme(None)
                .visible(!launched_by_autostart)
                .initialization_script(initial_script)
                .on_navigation(|url| {
                    url.scheme() == "https" && url.host_str() == Some("web.whatsapp.com")
                })
                .on_new_window(|url, _features| {
                    eprintln!("Blocked a WhatsApp Web popup request: {url}");
                    NewWindowResponse::Deny
                })
                .build()?;

            if let Some(geometry) = preferences.geometry.filter(|geometry| {
                (800..=10_000).contains(&geometry.width)
                    && (600..=10_000).contains(&geometry.height)
            }) {
                let _ = window.set_size(Size::Physical(PhysicalSize::new(
                    geometry.width,
                    geometry.height,
                )));
                let _ = window.set_position(Position::Physical(PhysicalPosition::new(
                    geometry.x, geometry.y,
                )));
            }

            let event_window = window.clone();
            let app_handle = app.handle().clone();
            window.on_window_event(move |event| match event {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    save_window_geometry(&event_window, &app_handle);
                    let _ = event_window.hide();
                }
                WindowEvent::Moved(_) | WindowEvent::Resized(_) => {
                    save_window_geometry(&event_window, &app_handle);
                }
                _ => {}
            });

            build_tray(app)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to initialize WhatsApp Lite")
        .run(|app, event| {
            if matches!(event, RunEvent::Exit | RunEvent::ExitRequested { .. }) {
                if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
                    save_window_geometry(&window, app);
                }
            }
        });
}
