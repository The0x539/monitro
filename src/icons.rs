use windows::core::Interface;
use windows::Win32::System::{Com::*, Ole::*};
use windows::Win32::UI::Shell::*;

pub fn hide() -> windows::core::Result<()> {
    unsafe { pain(FWF_NOICONS.0 as u32) }
}

pub fn unhide() -> windows::core::Result<()> {
    unsafe { pain(0) }
}

unsafe fn pain(flags: u32) -> windows::core::Result<()> {
    CoInitialize(std::ptr::null())?;

    let sp_shell_windows: IShellWindows = CoCreateInstance(&ShellWindows, None, CLSCTX_ALL)?;

    let mut vt_loc = VARIANT::default();
    (*vt_loc.Anonymous.Anonymous).vt = VT_UI4.0 as _;
    (*vt_loc.Anonymous.Anonymous).Anonymous.uintVal = CSIDL_DESKTOP;
    let vt_empty = Default::default();
    let mut hwnd = 0;
    let mut sp_disp = None;
    sp_shell_windows.FindWindowSW(
        &vt_loc,
        &vt_empty,
        SWC_DESKTOP.0,
        &mut hwnd,
        SWFO_NEEDDISPATCH.0,
        &mut sp_disp,
    )?;

    let sp_disp = sp_disp.unwrap();
    let sp_prov = sp_disp.cast::<IServiceProvider>()?;

    let mut sp_browser = std::mem::MaybeUninit::<IShellBrowser>::uninit();
    sp_prov.QueryService(
        &SID_STopLevelBrowser,
        &IShellBrowser::IID,
        sp_browser.as_mut_ptr() as *mut *mut _,
    )?;
    let sp_browser = sp_browser.assume_init();

    let sp_view = sp_browser.QueryActiveShellView()?;
    let sp_view = sp_view.cast::<IFolderView2>()?;

    sp_view.SetCurrentFolderFlags(FWF_NOICONS.0 as u32, flags)?;

    Ok(())
}
