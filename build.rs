use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/nputella.ico");
    if env::var("CARGO_CFG_WINDOWS").is_err() {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let icon = manifest_dir.join("assets").join("nputella.ico");
    if !icon.exists() {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let rc = out_dir.join("nputella.rc");
    let res = out_dir.join("nputella.res");
    let escaped_icon = icon.display().to_string().replace('\\', "\\\\");
    fs::write(&rc, format!("1 ICON \"{escaped_icon}\"\n")).unwrap();

    let Some(rc_exe) = find_rc_exe() else {
        println!("cargo:warning=rc.exe not found; nputella.exe icon was not embedded");
        return;
    };

    let status = Command::new(rc_exe)
        .arg("/nologo")
        .arg(format!("/fo{}", res.display()))
        .arg(&rc)
        .status()
        .unwrap();

    if status.success() {
        println!("cargo:rustc-link-arg-bins={}", res.display());
    } else {
        println!("cargo:warning=rc.exe failed; nputella.exe icon was not embedded");
    }
}

fn find_rc_exe() -> Option<PathBuf> {
    env::var_os("PATH")
        .and_then(|path| {
            env::split_paths(&path)
                .map(|dir| dir.join("rc.exe"))
                .find(|candidate| candidate.is_file())
        })
        .or_else(find_windows_kit_rc)
}

fn find_windows_kit_rc() -> Option<PathBuf> {
    let root = Path::new(r"C:\Program Files (x86)\Windows Kits\10\bin");
    let arch = env::var("HOST")
        .ok()
        .and_then(|host| host.split('-').next().map(str::to_string))
        .unwrap_or_else(|| "x64".to_string());
    let arch_dir = match arch.as_str() {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        "i686" => "x86",
        _ => "x64",
    };

    let mut versions = fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .ok()
                .map(|ty| ty.is_dir())
                .unwrap_or(false)
        })
        .map(|entry| entry.file_name())
        .filter_map(|name| os_str_to_string(name.as_os_str()))
        .collect::<Vec<_>>();
    versions.sort();
    versions.reverse();

    versions
        .into_iter()
        .map(|version| root.join(version).join(arch_dir).join("rc.exe"))
        .find(|candidate| candidate.is_file())
}

fn os_str_to_string(value: &OsStr) -> Option<String> {
    value.to_str().map(ToString::to_string)
}
