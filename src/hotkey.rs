use crate::logger;
use crate::ui::AppEvent;
use crossbeam_channel::Sender;
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VK_F17, VK_LCONTROL, VK_LWIN, VK_RCONTROL, VK_RWIN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, HC_ACTION,
    HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

static EVENT_TX: OnceCell<Sender<AppEvent>> = OnceCell::new();
static HOOK_STATE: OnceCell<Mutex<HookState>> = OnceCell::new();

pub fn spawn_hotkey_hook(tx: Sender<AppEvent>) {
    let _ = EVENT_TX.set(tx);
    let _ = HOOK_STATE.set(Mutex::new(HookState::default()));
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
        logger::line("hotkey hook installed for Ctrl+Win and F17");
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
        if let Some(state) = HOOK_STATE.get() {
            if let Ok(mut state) = state.lock() {
                let decision = state.handle(kb.vkCode, wparam.0 as u32);
                if let Some(event) = decision.event {
                    if let Some(tx) = EVENT_TX.get() {
                        let _ = tx.send(event);
                    }
                }
                if decision.swallow {
                    return LRESULT(1);
                }
            }
        }
    }
    CallNextHookEx(HHOOK::default(), code, wparam, lparam)
}

#[derive(Default)]
struct HookState {
    ctrl_down: bool,
    win_down: bool,
    ctrl_down_swallowed: bool,
    win_down_swallowed: bool,
    push_to_talk_active: bool,
    f17_down: bool,
}

struct HookDecision {
    event: Option<AppEvent>,
    swallow: bool,
}

impl HookState {
    fn handle(&mut self, vk_code: u32, message: u32) -> HookDecision {
        if vk_code == VK_F17.0 as u32 {
            return self.handle_f17(message);
        }
        if is_ctrl_vk(vk_code) || is_win_vk(vk_code) {
            return self.handle_ctrl_win(vk_code, message);
        }
        HookDecision {
            event: None,
            swallow: false,
        }
    }

    fn handle_f17(&mut self, message: u32) -> HookDecision {
        if is_key_down(message) {
            let send = !self.f17_down;
            self.f17_down = true;
            return HookDecision {
                event: send.then_some(AppEvent::HotkeyDown),
                swallow: true,
            };
        }
        if is_key_up(message) {
            let send = self.f17_down;
            self.f17_down = false;
            return HookDecision {
                event: send.then_some(AppEvent::HotkeyUp),
                swallow: true,
            };
        }
        HookDecision {
            event: None,
            swallow: false,
        }
    }

    fn handle_ctrl_win(&mut self, vk_code: u32, message: u32) -> HookDecision {
        let key_down = is_key_down(message);
        let key_up = is_key_up(message);
        let is_ctrl = is_ctrl_vk(vk_code);
        let is_win = is_win_vk(vk_code);

        if key_down {
            if is_ctrl {
                if self.ctrl_down {
                    return HookDecision {
                        event: None,
                        swallow: self.push_to_talk_active || self.ctrl_down_swallowed,
                    };
                }
                self.ctrl_down = true;
                self.ctrl_down_swallowed = false;
                return HookDecision {
                    event: None,
                    swallow: self.push_to_talk_active,
                };
            }
            if is_win {
                if self.win_down {
                    return HookDecision {
                        event: None,
                        swallow: self.push_to_talk_active || self.win_down_swallowed,
                    };
                }
                self.win_down = true;
                if self.ctrl_down && !self.push_to_talk_active {
                    self.push_to_talk_active = true;
                    self.win_down_swallowed = true;
                    return HookDecision {
                        event: Some(AppEvent::HotkeyDown),
                        swallow: true,
                    };
                }
                self.win_down_swallowed = false;
                return HookDecision {
                    event: None,
                    swallow: false,
                };
            }
        }

        if key_up {
            if is_ctrl {
                let swallow = self.ctrl_down_swallowed;
                self.ctrl_down = false;
                self.ctrl_down_swallowed = false;
                if self.push_to_talk_active {
                    self.push_to_talk_active = false;
                    return HookDecision {
                        event: Some(AppEvent::HotkeyUp),
                        swallow,
                    };
                }
                return HookDecision {
                    event: None,
                    swallow,
                };
            }
            if is_win {
                let swallow = self.win_down_swallowed;
                self.win_down = false;
                self.win_down_swallowed = false;
                if self.push_to_talk_active {
                    self.push_to_talk_active = false;
                    return HookDecision {
                        event: Some(AppEvent::HotkeyUp),
                        swallow,
                    };
                }
                return HookDecision {
                    event: None,
                    swallow,
                };
            }
        }

        HookDecision {
            event: None,
            swallow: false,
        }
    }
}

fn is_ctrl_vk(vk_code: u32) -> bool {
    vk_code == VK_LCONTROL.0 as u32 || vk_code == VK_RCONTROL.0 as u32
}

fn is_win_vk(vk_code: u32) -> bool {
    vk_code == VK_LWIN.0 as u32 || vk_code == VK_RWIN.0 as u32
}

fn is_key_down(message: u32) -> bool {
    matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN)
}

fn is_key_up(message: u32) -> bool {
    matches!(message, WM_KEYUP | WM_SYSKEYUP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_then_win_triggers_push_to_talk() {
        let mut state = HookState::default();

        let first = state.handle(VK_LCONTROL.0 as u32, WM_KEYDOWN);
        assert!(first.event.is_none());
        assert!(!first.swallow);

        let second = state.handle(VK_LWIN.0 as u32, WM_KEYDOWN);
        assert!(matches!(second.event, Some(AppEvent::HotkeyDown)));
        assert!(second.swallow);

        let release = state.handle(VK_LWIN.0 as u32, WM_KEYUP);
        assert!(matches!(release.event, Some(AppEvent::HotkeyUp)));
        assert!(release.swallow);

        let trailing = state.handle(VK_LCONTROL.0 as u32, WM_KEYUP);
        assert!(trailing.event.is_none());
        assert!(!trailing.swallow);
    }

    #[test]
    fn win_then_ctrl_is_passed_through() {
        let mut state = HookState::default();

        let first = state.handle(VK_LWIN.0 as u32, WM_KEYDOWN);
        assert!(first.event.is_none());
        assert!(!first.swallow);

        let second = state.handle(VK_LCONTROL.0 as u32, WM_KEYDOWN);
        assert!(second.event.is_none());
        assert!(!second.swallow);

        let release = state.handle(VK_LCONTROL.0 as u32, WM_KEYUP);
        assert!(release.event.is_none());
        assert!(!release.swallow);

        let trailing = state.handle(VK_LWIN.0 as u32, WM_KEYUP);
        assert!(trailing.event.is_none());
        assert!(!trailing.swallow);
    }

    #[test]
    fn right_alt_is_not_captured() {
        let mut state = HookState::default();

        let result = state.handle(0xA5, WM_SYSKEYDOWN);
        assert!(result.event.is_none());
        assert!(!result.swallow);
    }

    #[test]
    fn repeated_keydown_keeps_release_swallow_balanced() {
        let mut state = HookState::default();

        assert!(!state.handle(VK_LCONTROL.0 as u32, WM_KEYDOWN).swallow);
        assert!(state.handle(VK_LWIN.0 as u32, WM_KEYDOWN).swallow);
        assert!(state.handle(VK_LWIN.0 as u32, WM_KEYDOWN).swallow);

        let release = state.handle(VK_LWIN.0 as u32, WM_KEYUP);
        assert!(matches!(release.event, Some(AppEvent::HotkeyUp)));
        assert!(release.swallow);

        let trailing = state.handle(VK_LCONTROL.0 as u32, WM_KEYUP);
        assert!(trailing.event.is_none());
        assert!(!trailing.swallow);
    }

    #[test]
    fn ctrl_first_release_order_keeps_messages_balanced() {
        let mut state = HookState::default();

        assert!(!state.handle(VK_LCONTROL.0 as u32, WM_KEYDOWN).swallow);
        assert!(state.handle(VK_LWIN.0 as u32, WM_KEYDOWN).swallow);

        let ctrl_release = state.handle(VK_LCONTROL.0 as u32, WM_KEYUP);
        assert!(matches!(ctrl_release.event, Some(AppEvent::HotkeyUp)));
        assert!(!ctrl_release.swallow);

        let win_release = state.handle(VK_LWIN.0 as u32, WM_KEYUP);
        assert!(win_release.event.is_none());
        assert!(win_release.swallow);
    }

    #[test]
    fn f17_remains_available_for_development() {
        let mut state = HookState::default();

        let down = state.handle(VK_F17.0 as u32, WM_KEYDOWN);
        assert!(matches!(down.event, Some(AppEvent::HotkeyDown)));
        assert!(down.swallow);

        let up = state.handle(VK_F17.0 as u32, WM_KEYUP);
        assert!(matches!(up.event, Some(AppEvent::HotkeyUp)));
        assert!(up.swallow);
    }
}
