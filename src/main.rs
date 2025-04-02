#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod http_server;
mod keyboard_conf;
#[cfg(target_os = "windows")]
mod keyboard_hook;

use log::{error, info};

use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger};
#[cfg(target_os = "windows")]
use keyboard_hook::KeyboardHook;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, TranslateMessage,
};

use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};

const MENU_PREF_ID: &str = "1000";
const MENU_QUIT_ID: &str = "1001";
const MENU_RELOAD_ID: &str = "1002";

fn main() {

    init_logger();

    set_panic_hook();

    keyboard_conf::read();

    let port = {
        let conf = keyboard_conf::CONF.lock().unwrap();
        conf.port
    };

    let http_server = http_server::create_http_server(port);

    if let Err(err) = http_server {
        error!("cannot start http server: {}, maybe another instance is running, or the port {} is in use by other process", err, port);
        return;
    }

    #[cfg(target_os = "windows")]
    let kh = KeyboardHook::new();

    let icon_result = tray_icon::Icon::from_resource(32512, None);
    let icon = match icon_result {
        Ok(x) => x,
        Err(err) => {
            error!("Failed to load icon from resource, using full white icon, err: {}", err);
            let icon_rgbas: Vec<u8> = vec![255; 4096];
            tray_icon::Icon::from_rgba(icon_rgbas, 32, 32).unwrap()
        }
    };

    let tray_menu = Menu::new();
    tray_menu
        .append_items(&[
            &MenuItem::with_id(MENU_PREF_ID, "Preferences", true, None),
            &MenuItem::with_id(MENU_RELOAD_ID, "Reload Config", true, None),
            &MenuItem::with_id(MENU_QUIT_ID, "Quit", true, None),
        ])
        .unwrap();

    let _tray_icon = TrayIconBuilder::new()
        .with_tooltip("Hyper-Key")
        .with_icon(icon)
        .with_menu(Box::new(tray_menu))
        .build()
        .unwrap();

    info!("starting!");

    #[cfg(target_os = "windows")]
    kh.start();
    info!("started!");

    run_message_loop();

    info!("stopping!");
    #[cfg(target_os = "windows")]
    kh.stop();
    // _ = file.unlock();
}

fn init_logger() {
    let config = ConfigBuilder::new()
        .set_time_offset_to_local().unwrap()
        .build();

    let file = std::fs::OpenOptions::new().create(true).append(true).open(keyboard_conf::get_config_folder_path().join("error.log")).unwrap();
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Trace, config.clone(), TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(LevelFilter::Error, config.clone(), file),
    ]).unwrap();
}

fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        error!("{}", info.to_string());
    }));
}

#[cfg(target_os = "windows")]
fn run_message_loop() {
    let mut msg = MSG::default();

    loop {
        if unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            unsafe {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
                if handle_menu_event() {
                    break;
                }
            }
        } else {
            break;
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn run_message_loop() {
    loop {
        if handle_menu_event() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(33));
    }
}

fn handle_menu_event() -> bool {
    if let Ok(event) = MenuEvent::receiver().try_recv() {
        return match &event.id.0[..] {
            MENU_QUIT_ID => true,
            MENU_PREF_ID => {
                let config = keyboard_conf::CONF.lock();
                if config.is_err() {
                    return false;
                }
                let config = config.unwrap();
                _ = open::that(format!("http://{}:{}/config", http_server::SERVER_ADDR, config.port));
                false
            }
            MENU_RELOAD_ID => {
                keyboard_conf::read();
                false
            }
            _ => false,
        };
    }
    false
}
