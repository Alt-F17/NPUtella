use anyhow::{anyhow, Result};
use windows::core::w;
use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;

pub struct SingleInstance {
    handle: HANDLE,
}

impl SingleInstance {
    pub fn acquire() -> Result<Self> {
        let handle = unsafe { CreateMutexW(None, true, w!("Local\\NPUtellaNativeOverlay"))? };
        let last_error = windows::core::Error::from_win32();
        if last_error.code() == ERROR_ALREADY_EXISTS.to_hresult() {
            unsafe {
                let _ = CloseHandle(handle);
            }
            return Err(anyhow!("another nputella instance is already running"));
        }
        Ok(Self { handle })
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
