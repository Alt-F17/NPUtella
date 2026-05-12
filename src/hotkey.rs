use crate::logger;
use crate::ui::AppEvent;
use crossbeam_channel::Sender;
use once_cell::sync::OnceCell;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F17;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, HC_ACTION,
    HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

static EVENT_TX: OnceCell<Sender<AppEvent>> = OnceCell::new();

pub fn spawn_hotkey_hook(tx: Sender<AppEvent>) {
    let _ = EVENT_TX.set(tx);
    std::thread::spawn(|| unsafe {
        let module = GetModuleHandleW(None).unwrap_or_default();
        let hook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), module, 0) {
            Ok(hook) => hook,
            Err(err) => {
                logger::line(format!("hotkey hook install failed: {err:?}"));
                return;
            }
        };
        if hook.0.is_null() {
            logger::line("hotkey hook install returned null hook");
            return;
        }
        logger::line("hotkey hook installed for F17");
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        logger::line("hotkey hook message loop exited");
    });
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let kb = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        if kb.vkCode == VK_F17.0 as u32 {
            let event = match wparam.0 as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => Some(AppEvent::HotkeyDown),
                WM_KEYUP | WM_SYSKEYUP => Some(AppEvent::HotkeyUp),
                _ => None,
            };
            if let Some(event) = event {
                if let Some(tx) = EVENT_TX.get() {
                    let _ = tx.send(event);
                }
                return LRESULT(1);
            }
        }
    }
    CallNextHookEx(HHOOK::default(), code, wparam, lparam)
}
