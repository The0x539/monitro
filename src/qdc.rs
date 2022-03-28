use windows::Win32::{Devices::Display::*, Graphics::Gdi::*};

use anyhow::Result;

use crate::WinErrorExt;

fn query(
    flags: u32,
    topology_id: Option<&mut DISPLAYCONFIG_TOPOLOGY_ID>,
) -> Result<(Vec<DISPLAYCONFIG_PATH_INFO>, Vec<DISPLAYCONFIG_MODE_INFO>)> {
    let (mut num_paths, mut num_modes) = (0, 0);
    unsafe {
        GetDisplayConfigBufferSizes(flags, &mut num_paths, &mut num_modes).win_result()?;
    }

    let mut paths = Vec::with_capacity(num_paths as usize);
    let mut modes = Vec::with_capacity(num_modes as usize);
    let topo_id_ptr = topology_id.map_or(std::ptr::null_mut(), |x| x);
    unsafe {
        QueryDisplayConfig(
            flags,
            &mut num_paths,
            paths.as_mut_ptr(),
            &mut num_modes,
            modes.as_mut_ptr(),
            topo_id_ptr,
        )
        .win_result()?;

        paths.set_len(num_paths as usize);
        modes.set_len(num_modes as usize);
    }

    Ok((paths, modes))
}

pub fn query_display_config(
    all_paths: bool,
) -> Result<(Vec<DISPLAYCONFIG_PATH_INFO>, Vec<DISPLAYCONFIG_MODE_INFO>)> {
    let flags = if all_paths {
        QDC_ALL_PATHS
    } else {
        QDC_ONLY_ACTIVE_PATHS
    };
    query(flags, None)
}

pub fn database_current() -> Result<(
    DISPLAYCONFIG_TOPOLOGY_ID,
    Vec<DISPLAYCONFIG_PATH_INFO>,
    Vec<DISPLAYCONFIG_MODE_INFO>,
)> {
    let mut topo_id = Default::default();
    let (paths, modes) = query(QDC_DATABASE_CURRENT, Some(&mut topo_id))?;
    Ok((topo_id, paths, modes))
}
