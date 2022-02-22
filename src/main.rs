#![windows_subsystem = "windows"]

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

type WinResult<T> = Result<T, WIN32_ERROR>;

fn win_result(ret: i32) -> WinResult<()> {
    let ret = WIN32_ERROR(ret as u32);
    if ret == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(ret)
    }
}

pub mod icons;
pub mod qdc;
pub mod sdc;

fn sdc_config(focus_mode: bool) -> sdc::Sdc<'static> {
    let topologies = sdc::SdcDatabaseTopologies {
        internal: false,
        clone: false,
        extend: !focus_mode,
        external: focus_mode,
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

fn enter_focus_mode() {
    icons::hide().unwrap();
    sdc::set_display_config(sdc_config(true)).unwrap();
}

fn exit_focus_mode() {
    sdc::set_display_config(sdc_config(false)).unwrap();
    icons::unhide().unwrap();
}

fn toggle_focus_mode() {
    static ON: AtomicBool = AtomicBool::new(false);
    if ON.fetch_xor(true, Ordering::Relaxed) {
        exit_focus_mode();
    } else {
        enter_focus_mode();
    }
}

fn sleep_displays() {
    std::thread::sleep(std::time::Duration::from_secs(3));
    
    let hwnd = HWND(0xFFFF);
    let msg = WM_SYSCOMMAND;
    let wp = WPARAM(SC_MONITORPOWER as usize);
    let lp = LPARAM(2);
    unsafe { SendMessageA(hwnd, msg, wp, lp) }.ok().unwrap();
}

fn quit() {
    std::process::exit(0);
}

fn main() {
    let mut tray = tray_item::TrayItem::new("monitro", "tray-icon").unwrap();
    tray.add_menu_item("Toggle focus mode", toggle_focus_mode)
        .unwrap();
    tray.add_menu_item("Sleep displays", sleep_displays)
        .unwrap();
    tray.add_menu_item("Quit", quit).unwrap();
    std::thread::sleep(Duration::MAX);
}
