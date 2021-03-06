#![windows_subsystem = "windows"]
#![deny(rust_2018_idioms)]

use std::io::Write;
use std::path::Path;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use windows::Win32::Devices::Display::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use anyhow::{anyhow, Context, Result};
use tauri_hotkey::{Hotkey, HotkeyManager, Key, Modifier};
use tray_item::{TIError, TrayItem};

trait WinErrorExt {
    fn win_result(self) -> windows::core::Result<()>;
}

impl WinErrorExt for i32 {
    fn win_result(self) -> windows::core::Result<()> {
        WIN32_ERROR(self as u32).ok()
    }
}

pub mod icons;
pub mod qdc;
pub mod sdc;

fn sdc_config(focus_mode: bool) -> sdc::Sdc<'static> {
    let topologies = sdc::SdcDatabaseTopologies {
        internal: focus_mode,
        clone: false,
        extend: !focus_mode,
        external: false,
    };

    sdc::Sdc {
        action: sdc::SdcAction::Apply {
            no_optimization: false,
        },
        config: sdc::SdcConfig::Database {
            topologies,
            path_persist_if_required: false,
        },
        allow_changes: false,
    }
}

static FOCUS_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

fn enter_focus_mode() -> Result<()> {
    FOCUS_MODE_ACTIVE.store(true, Ordering::Relaxed);
    icons::hide()?;
    sdc::set_display_config(sdc_config(true))?;
    Ok(())
}

fn exit_focus_mode() -> Result<()> {
    FOCUS_MODE_ACTIVE.store(false, Ordering::Relaxed);
    sdc::set_display_config(sdc_config(false))?;
    icons::unhide()?;
    Ok(())
}

fn toggle_focus_mode() -> Result<()> {
    if FOCUS_MODE_ACTIVE.load(Ordering::Relaxed) {
        exit_focus_mode()?;
    } else {
        enter_focus_mode()?;
    }
    Ok(())
}

fn sleep_displays(secs: u64) -> Result<()> {
    std::thread::sleep(std::time::Duration::from_secs(secs));

    let hwnd = HWND(0xFFFF);
    let msg = WM_SYSCOMMAND;
    let wp = WPARAM(SC_MONITORPOWER as usize);
    let lp = LPARAM(2);
    unsafe {
        SendMessageA(hwnd, msg, wp, lp).ok()?;
    }
    Ok(())
}

fn log_err(err: anyhow::Error) -> Result<()> {
    let homedir = std::env::var("USERPROFILE")?;
    let log_path = Path::new(&homedir).join("monitro_error.txt");
    let mut log_file = std::fs::File::create(log_path)?;
    writeln!(log_file, "{}", err)?;
    writeln!(log_file)?;
    writeln!(log_file, "{}", err.backtrace())?;
    Ok(())
}

fn wrap(f: fn() -> Result<()>) -> impl Fn() + Send + Sync + 'static {
    move || f().or_else(log_err).unwrap_or_else(|_| exit(1))
}

fn setup_tray() -> Result<TrayItem, TIError> {
    let mut tray = TrayItem::new("monitro", "tray-icon")?;
    tray.add_menu_item("Toggle focus mode", wrap(toggle_focus_mode))?;
    tray.add_menu_item("Sleep displays", wrap(|| sleep_displays(3)))?;
    tray.add_menu_item("Quit", || exit(0))?;
    Ok(tray)
}

fn setup_hotkeys() -> Result<HotkeyManager, tauri_hotkey::Error> {
    let mut manager = HotkeyManager::new();

    let hyper = |key| Hotkey {
        modifiers: vec![Modifier::CTRL, Modifier::SUPER, Modifier::ALT],
        keys: vec![key],
    };

    manager.register(hyper(Key::F), wrap(toggle_focus_mode))?;
    manager.register(hyper(Key::M), wrap(|| sleep_displays(0)))?;

    Ok(manager)
}

fn setup() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");

    let (topo_id, _, _) = qdc::database_current()?;
    if topo_id == DISPLAYCONFIG_TOPOLOGY_INTERNAL {
        icons::hide()?;
        FOCUS_MODE_ACTIVE.store(true, Ordering::Relaxed);
    } else {
        icons::unhide()?;
    }

    let tray = setup_tray().map_err(|e| anyhow!("Error setting up tray: {e}"));
    std::mem::forget(tray);

    let hotkeys = setup_hotkeys().context("Error setting up hotkeys")?;
    std::mem::forget(hotkeys);

    Ok(())
}

fn main() -> ! {
    setup().unwrap();
    loop {
        std::thread::sleep(Duration::MAX);
    }
}
