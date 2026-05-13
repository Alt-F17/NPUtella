use crate::logger;
use crate::ui::AppEvent;
use crossbeam_channel::Sender;
use eframe::egui;
use once_cell::sync::OnceCell;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE,
    NIM_MODIFY, NIM_SETVERSION, NIN_SELECT, NOTIFYICONDATAW, NOTIFYICONDATAW_0,
    NOTIFYICON_VERSION_4,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIcon, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon,
    DestroyMenu, DispatchMessageW, GetCursorPos, GetMessageW, LoadIconW, LoadImageW, PostMessageW,
    PostQuitMessage, RegisterClassW, SetForegroundWindow, TrackPopupMenu, TranslateMessage,
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, HICON, HMENU, IDI_APPLICATION, IMAGE_ICON,
    LR_LOADFROMFILE, MF_SEPARATOR, MF_STRING, MSG, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_COMMAND, WM_LBUTTONDBLCLK,
    WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP, WNDCLASSW,
};

const TRAY_UID: u32 = 17;
const WM_TRAYICON: u32 = WM_APP + 17;
const MENU_DICTIONARY: usize = 1001;
const MENU_QUIT: usize = 1002;

static EVENT_TX: OnceCell<Sender<AppEvent>> = OnceCell::new();
static UI_CTX: OnceCell<egui::Context> = OnceCell::new();

pub fn spawn_tray(tx: Sender<AppEvent>) {
    let _ = EVENT_TX.set(tx);
    std::thread::spawn(|| unsafe {
        if let Err(err) = run_tray_loop() {
            logger::line(format!("tray init failed: {err:?}"));
        }
    });
}

pub fn register_ui_context(ctx: egui::Context) {
    let _ = UI_CTX.set(ctx);
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

    let owned_icon = load_embedded_icon()
        .or_else(|| load_nputella_icon())
        .or_else(|| nputella_icon());
    let tray_icon =
        owned_icon.unwrap_or_else(|| LoadIconW(None, IDI_APPLICATION).unwrap_or_default());
    let mut nid = notify_icon(hwnd, tray_icon);
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
    nid.uFlags = NIF_TIP | NIF_SHOWTIP;
    if !Shell_NotifyIconW(NIM_MODIFY, &nid).as_bool() {
        logger::line(format!(
            "Shell_NotifyIconW tooltip modify failed: {:?}",
            windows::core::Error::from_win32()
        ));
    }
    logger::line("tray icon installed");

    let mut msg = MSG::default();
    while GetMessageW(&mut msg, HWND::default(), 0, 0).into() {
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    let _ = Shell_NotifyIconW(NIM_DELETE, &notify_icon(hwnd, tray_icon));
    if owned_icon.is_some() {
        let _ = DestroyIcon(tray_icon);
    }
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
            let event = loword(lparam.0 as u32) as u32;
            match event {
                WM_LBUTTONUP | WM_LBUTTONDBLCLK => {
                    logger::line("tray left click");
                    open_dictionary_manager();
                }
                NIN_SELECT => {
                    logger::line("tray select");
                    open_dictionary_manager();
                }
                WM_RBUTTONUP => {
                    logger::line("tray right click");
                    show_menu(hwnd);
                }
                windows::Win32::UI::WindowsAndMessaging::WM_CONTEXTMENU | 0x405 => {
                    logger::line("tray context menu");
                    show_menu(hwnd);
                }
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
        let _ = PostMessageW(hwnd, WM_NULL, WPARAM(0), LPARAM(0));
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
    if let Some(ctx) = UI_CTX.get() {
        ctx.request_repaint();
    }
}

fn quit_app() {
    if let Some(tx) = EVENT_TX.get() {
        let _ = tx.send(AppEvent::Quit);
    }
    if let Some(ctx) = UI_CTX.get() {
        ctx.request_repaint();
    }
    unsafe {
        PostQuitMessage(0);
    }
}

unsafe fn notify_icon(hwnd: HWND, hicon: HICON) -> NOTIFYICONDATAW {
    let mut nid = NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_UID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_TRAYICON,
        hIcon: hicon,
        ..Default::default()
    };
    write_wide_buf(&mut nid.szTip, "NPUtella");
    nid
}

unsafe fn nputella_icon() -> Option<HICON> {
    let mut and_mask = vec![0xff; (32 * 32) / 8];
    let mut xor_bits = vec![0; 32 * 32 * 4];

    for y in 0..32 {
        for x in 0..32 {
            let idx = (y * 32 + x) as usize;
            let dx = x as f32 - 15.5;
            let dy = y as f32 - 15.5;
            let in_disc = dx * dx + dy * dy <= 14.5 * 14.5;
            let in_stem_left = (8..=11).contains(&x) && (8..=24).contains(&y);
            let in_stem_right = (20..=23).contains(&x) && (8..=24).contains(&y);
            let in_diagonal = (8..=23).contains(&x)
                && (8..=24).contains(&y)
                && ((x as i32 - y as i32 + 1).abs() <= 2);
            if in_disc || in_stem_left || in_stem_right || in_diagonal {
                let mask_byte = idx / 8;
                let mask_bit = 7 - (idx % 8);
                and_mask[mask_byte] &= !(1 << mask_bit);
            }

            let out = idx * 4;
            if in_disc {
                xor_bits[out] = 18;
                xor_bits[out + 1] = 18;
                xor_bits[out + 2] = 18;
            }
            if in_stem_left || in_stem_right || in_diagonal {
                xor_bits[out] = 235;
                xor_bits[out + 1] = 235;
                xor_bits[out + 2] = 235;
            }
        }
    }

    match CreateIcon(None, 32, 32, 1, 32, and_mask.as_ptr(), xor_bits.as_ptr()) {
        Ok(icon) => Some(icon),
        Err(err) => {
            logger::line(format!("custom tray icon creation failed: {err:?}"));
            None
        }
    }
}

unsafe fn load_nputella_icon() -> Option<HICON> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("nputella.ico");
    let mut wide = path.as_os_str().encode_wide().collect::<Vec<_>>();
    wide.push(0);
    let handle = LoadImageW(
        None,
        PCWSTR(wide.as_ptr()),
        IMAGE_ICON,
        32,
        32,
        LR_LOADFROMFILE,
    )
    .ok()?;
    Some(HICON(handle.0))
}

unsafe fn load_embedded_icon() -> Option<HICON> {
    let module = GetModuleHandleW(None).ok()?;
    LoadIconW(module, PCWSTR(1 as *const u16)).ok()
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
