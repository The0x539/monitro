use windows::Win32::Foundation::*;

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

fn main() {
    let enable = match std::env::args().nth(1).as_deref() {
        Some("on") => true,
        Some("off") => false,
        _ => return,
    };

    if !enable {
        icons::hide().unwrap();
    }

    let topologies = sdc::SdcDatabaseTopologies {
        internal: false,
        clone: false,
        extend: enable,
        external: !enable,
    };

    let sdc_config = sdc::Sdc {
        action: sdc::SdcAction::Apply {
            no_optimization: false,
        },
        config: sdc::SdcConfig::Database {
            topologies,
            path_persist_if_required: false,
        },
        allow_changes: false,
    };

    sdc::set_display_config(sdc_config).unwrap();

    if enable {
        icons::unhide().unwrap();
    }
}
