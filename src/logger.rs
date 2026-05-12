use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

pub fn init(root: &Path) {
    let path = root.join("nputella.log");
    if let Ok(file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = LOG_FILE.set(Mutex::new(file));
        line(format!("log initialized at {}", path.display()));
    }
}

pub fn line(message: impl AsRef<str>) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or_default();
    let line = format!("[{ts:.3}] {}\n", message.as_ref());
    if let Some(file) = LOG_FILE.get() {
        if let Ok(mut file) = file.lock() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }
}
