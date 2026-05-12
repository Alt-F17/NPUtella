use crate::pipeline::InsertPlan;
use anyhow::{Context, Result};
use arboard::Clipboard;
use std::thread;
use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY,
    VK_CONTROL, VK_RETURN, VK_V,
};

pub fn insert_text(plan: &InsertPlan) -> Result<()> {
    let mut clipboard = Clipboard::new().context("opening clipboard")?;
    clipboard
        .set_text(plan.text.to_string())
        .context("setting clipboard text")?;
    thread::sleep(Duration::from_millis(50));
    unsafe {
        key_down(VK_CONTROL.0 as u16);
        key_down(VK_V.0 as u16);
        key_up(VK_V.0 as u16);
        key_up(VK_CONTROL.0 as u16);
        if plan.press_enter {
            thread::sleep(Duration::from_millis(25));
            key_down(VK_RETURN.0 as u16);
            key_up(VK_RETURN.0 as u16);
        }
    }
    Ok(())
}

unsafe fn key_down(vk: u16) {
    send_key(vk, false);
}

unsafe fn key_up(vk: u16) {
    send_key(vk, true);
}

unsafe fn send_key(vk: u16, up: bool) {
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    Default::default()
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let _ = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
}
