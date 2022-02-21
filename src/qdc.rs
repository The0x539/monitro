use windows::Win32::{Devices::Display::*, Graphics::Gdi::*};

use crate::{win_result, WinResult};

pub fn query_display_config(
    all_paths: bool,
) -> WinResult<(Vec<DISPLAYCONFIG_PATH_INFO>, Vec<DISPLAYCONFIG_MODE_INFO>)> {
    let flags = if all_paths {
        QDC_ALL_PATHS
    } else {
        QDC_ONLY_ACTIVE_PATHS
    };

    let (mut num_paths, mut num_modes) = (0, 0);
    unsafe {
        let ret = GetDisplayConfigBufferSizes(flags, &mut num_paths, &mut num_modes);
        win_result(ret)?;
    }

    let mut paths = Vec::with_capacity(num_paths as usize);
    let mut modes = Vec::with_capacity(num_modes as usize);
    unsafe {
        let ret = QueryDisplayConfig(
            flags,
            &mut num_paths,
            paths.as_mut_ptr(),
            &mut num_modes,
            modes.as_mut_ptr(),
            std::ptr::null_mut(),
        );
        win_result(ret)?;
        paths.set_len(num_paths as usize);
        modes.set_len(num_modes as usize);
    }

    Ok((paths, modes))
}
