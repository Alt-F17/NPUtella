use crate::logger;
use crate::ui::AppEvent;
use crossbeam_channel::Sender;
use once_cell::sync::OnceCell;
use std::mem::size_of;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_SETVERSION,
    NOTIFYICONDATAW, NOTIFYICONDATAW_0, NOTIFYICON_VERSION_4,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    GetCursorPos, GetMessageW, LoadIconW, PostQuitMessage, RegisterClassW, SetForegroundWindow,
    TrackPopupMenu, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, HMENU,
    IDI_APPLICATION, MF_SEPARATOR, MF_STRING, MSG, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_COMMAND, WM_LBUTTONDBLCLK,
    WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSW,
};

const TRAY_UID: u32 = 17;
const WM_TRAYICON: u32 = WM_APP + 17;
const MENU_DICTIONARY: usize = 1001;
const MENU_QUIT: usize = 1002;

static EVENT_TX: OnceCell<Sender<AppEvent>> = OnceCell::new();

pub fn spawn_tray(tx: Sender<AppEvent>) {
    let _ = EVENT_TX.set(tx);
    std::thread::spawn(|| unsafe {
        if let Err(err) = run_tray_loop() {
            logger::line(format!("tray init failed: {err:?}"));
        }
    });
}

unsafe fn run_tray_loop() -> windows::core::Result<()> {
    let module = GetModuleHandleW(None)?;
    let class_name = w!("NPUtellaTrayWindow");
    let wc = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        hInstance: module.into(),
        lpszClassName: class_name,
        ..Default::default()
    };
    RegisterClassW(&wc);

    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        class_name,
        w!("NPUtella Tray"),
        WINDOW_STYLE::default(),
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        HWND::default(),
        HMENU::default(),
        module,
        None,
    )?;

    let mut nid = notify_icon(hwnd);
    if !Shell_NotifyIconW(NIM_ADD, &nid).as_bool() {
        logger::line(format!(
            "Shell_NotifyIconW add failed: {:?}",
            windows::core::Error::from_win32()
        ));
        return Ok(());
    }
    nid.Anonymous = NOTIFYICONDATAW_0 {
        uVersion: NOTIFYICON_VERSION_4,
    };
    let _ = Shell_NotifyIconW(NIM_SETVERSION, &nid);
    logger::line("tray icon installed");

    let mut msg = MSG::default();
    while GetMessageW(&mut msg, HWND::default(), 0, 0).into() {
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    let _ = Shell_NotifyIconW(NIM_DELETE, &notify_icon(hwnd));
    logger::line("tray message loop exited");
    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            match lparam.0 as u32 {
                WM_LBUTTONUP | WM_LBUTTONDBLCLK => open_dictionary_manager(),
                WM_RBUTTONUP => show_menu(hwnd),
                _ => {}
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            match loword(wparam.0 as u32) as usize {
                MENU_DICTIONARY => open_dictionary_manager(),
                MENU_QUIT => quit_app(),
                _ => {}
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_menu(hwnd: HWND) {
    let menu = match CreatePopupMenu() {
        Ok(menu) => menu,
        Err(err) => {
            logger::line(format!("tray menu creation failed: {err:?}"));
            return;
        }
    };
    let _ = AppendMenuW(menu, MF_STRING, MENU_DICTIONARY, w!("Open Dictionary"));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
    let _ = AppendMenuW(menu, MF_STRING, MENU_QUIT, w!("Quit"));

    let mut pos = POINT::default();
    if GetCursorPos(&mut pos).is_ok() {
        let _ = SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD,
            pos.x,
            pos.y,
            0,
            hwnd,
            None,
        );
        match cmd.0 as usize {
            MENU_DICTIONARY => open_dictionary_manager(),
            MENU_QUIT => quit_app(),
            _ => {}
        }
    }
    let _ = DestroyMenu(menu);
}

fn open_dictionary_manager() {
    if let Some(tx) = EVENT_TX.get() {
        let _ = tx.send(AppEvent::OpenDictionaryManager);
    }
}

fn quit_app() {
    if let Some(tx) = EVENT_TX.get() {
        let _ = tx.send(AppEvent::Quit);
    }
    unsafe {
        PostQuitMessage(0);
    }
}

unsafe fn notify_icon(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut nid = NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_UID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_TRAYICON,
        hIcon: LoadIconW(None, IDI_APPLICATION).unwrap_or_default(),
        ..Default::default()
    };
    write_wide_buf(&mut nid.szTip, "NPUtella");
    nid
}

fn write_wide_buf(buf: &mut [u16], text: &str) {
    for (idx, unit) in text
        .encode_utf16()
        .take(buf.len().saturating_sub(1))
        .enumerate()
    {
        buf[idx] = unit;
    }
}

fn loword(value: u32) -> u16 {
    (value & 0xffff) as u16
}
