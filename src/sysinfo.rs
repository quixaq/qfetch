// SPDX-FileCopyrightText: 2026 Quixaq
// SPDX-License-Identifier: GPL-3.0-or-later

use nix::sys::{statvfs, sysinfo, utsname::uname};
#[cfg(target_arch = "x86_64")]
use raw_cpuid::CpuId;
use size::Size;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use gethostname::gethostname;

use crate::{KEYS_COLOR, MOUNTS_KEY, SEPARATOR_COLOR, TITLE_COLOR, VALUES_COLOR};

pub fn title() -> (Option<String>, Option<String>) {
    let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("LOGNAME")) else {
        return (None, None);
    };
    let Ok(hostname) = gethostname().into_string() else {
        return (None, None);
    };
    if user.is_empty() || hostname.is_empty() {
        return (None, None);
    }
    let title = format!("{}@{}", user, hostname);
    let sep = "-".repeat(title.len());
    (
        Some(format!("{}{}", TITLE_COLOR, title)),
        Some(format!("\x1b[0m{}", sep)),
    )
}

pub fn distro() -> (Option<String>, Option<String>, Option<String>) {
    let Ok(content) = fs::read_to_string("/etc/os-release") else {
        return (None, None, None);
    };
    let (mut pretty, mut id, mut id_like) = (None, None, None);

    for line in content.lines() {
        if let Some(distro) = line.strip_prefix("PRETTY_NAME=") {
            pretty = Some(distro.trim_matches('"').to_owned());
        } else if let Some(val) = line.strip_prefix("ID=") {
            id = Some(val.trim_matches('"').to_owned());
        } else if let Some(val) = line.strip_prefix("ID_LIKE=") {
            id_like = Some(val.trim_matches('"').to_owned());
        }

        if pretty.is_some() && id.is_some() && id_like.is_some() {
            break;
        }
    }
    (pretty, id, id_like)
}

pub fn kernel() -> Option<String> {
    let uts = uname().ok()?;
    Some(format!(
        "{} {}",
        uts.sysname().to_str()?,
        uts.release().to_str()?
    ))
}

pub fn host() -> Option<String> {
    let content = fs::read_to_string("/sys/class/dmi/id/product_name").ok()?;
    let product = content.trim();
    Some(product.to_owned())
}

pub fn shell() -> Option<String> {
    std::env::var("SHELL").ok().and_then(|path| {
        Path::new(&path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    })
}

pub fn de() -> Option<String> {
    std::env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .map(|s| s.split(':').last().unwrap_or(&s).to_string())
        .or_else(|| std::env::var("XDG_SESSION_DESKTOP").ok())
}

pub fn theme() -> Option<String> {
    std::env::var("GTK_THEME").ok()
}

pub fn cursor() -> Option<String> {
    if let Ok(xcursor) = std::env::var("XCURSOR_THEME") {
        if let Ok(size) = std::env::var("XCURSOR_SIZE") {
            return Some(format!("{} ({}px)", xcursor, size));
        }
        return Some(xcursor);
    }
    if let Ok(hyprcursor) = std::env::var("HYPRCURSOR_THEME") {
        if let Ok(size) = std::env::var("HYPRCURSOR_SIZE") {
            return Some(format!("{} ({}px)", hyprcursor, size));
        }
        return Some(hyprcursor);
    }
    None
}

pub fn cpu() -> Option<String> {
    #[cfg(target_arch = "x86_64")]
    {
        let cpuid = CpuId::new();

        cpuid
            .get_processor_brand_string()
            .map(|brand| brand.as_str().to_string())
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        let file = std::fs::File::open("/proc/cpuinfo").ok()?;
        let reader = std::io::BufReader::new(file);

        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with("model name") {
                return line
                    .split_once(':')
                    .map(|(_, name)| name.trim().to_string());
            }
        }
        None
    }
}

pub fn sysinfo() -> (Option<String>, Option<String>, Option<String>) {
    match sysinfo::sysinfo() {
        Ok(info) => {
            let seconds = info.uptime().as_secs();
            let days = seconds / 86400;
            let hours = (seconds % 86400) / 3600;
            let minutes = (seconds % 3600) / 60;

            let mut out = String::with_capacity(32);

            if days > 0 {
                let _ = write!(out, "{} day{}, ", days, if days == 1 { "" } else { "s" });
            }
            if hours > 0 {
                let _ = write!(out, "{} hour{}, ", hours, if hours == 1 { "" } else { "s" });
            }
            if minutes > 0 || out.is_empty() {
                let _ = write!(
                    out,
                    "{} min{}, ",
                    minutes,
                    if minutes == 1 { "" } else { "s" }
                );
            }
            if out.ends_with(", ") {
                out.truncate(out.len() - 2);
            }

            let total = info.ram_total();
            let available = info.ram_unused();
            let total_swap = info.swap_total();
            let free_swap = info.swap_free();

            let used = Size::from_bytes(total - available);
            let total_gib = Size::from_bytes(total);
            let used_swap = Size::from_bytes(total_swap - free_swap);
            let total_swap_gib = Size::from_bytes(total_swap);

            (
                Some(out),
                Some(format!("{} / {}", used, total_gib)),
                Some(format!("{} / {}", used_swap, total_swap_gib)),
            )
        }
        Err(_) => (None, None, None),
    }
}

pub fn gpu() -> Option<String> {
    getgpuname::get_gpu_name()
}

pub fn locale() -> Option<String> {
    std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LANG"))
        .ok()
}

pub fn mounts() -> Option<String> {
    let Ok(file) = File::open("/proc/mounts") else {
        return None;
    };
    let reader = BufReader::new(file);
    let mut seen_sources = std::collections::HashSet::new();
    let mounts: Vec<(String, String)> = reader
        .lines()
        .filter_map(Result::ok)
        .map(|line| {
            let mut parts = line.split_whitespace();
            let source = parts.next().unwrap_or("").to_string();
            let path = parts.next().unwrap_or("").to_string();
            let fs = parts.next().unwrap_or("").to_string();
            (source, path, fs)
        })
        .filter(|(source, path, fs)| {
            let allowed_fs = matches!(
                fs.as_str(),
                "ext4"
                    | "ext3"
                    | "ext2"
                    | "xfs"
                    | "btrfs"
                    | "f2fs"
                    | "zfs"
                    | "ntfs"
                    | "vfat"
                    | "exfat"
            );

            let allowed_source = source.starts_with("/dev/") && !source.starts_with("/loop");

            let allowed_path =
                !(path.starts_with("/boot") || path == "/var/lib/containers/storage/overlay");

            let unique_source = seen_sources.insert(source.clone());

            allowed_fs && allowed_source && allowed_path && unique_source
        })
        .map(|(_, path, fs)| (fs, path))
        .collect();

    let mut out: String = "".to_string();
    for (fs, mount) in mounts {
        let Ok(stats) = statvfs::statvfs(mount.as_str()) else {
            continue;
        };
        let block_size = stats.fragment_size() as u64;
        let total = stats.blocks() * block_size;
        let used = (stats.blocks() - stats.blocks_free()) * block_size;
        let full = used * 100 / total;
        out.push_str(&format!("{KEYS_COLOR}{MOUNTS_KEY} (\x1b[0m{mount}{KEYS_COLOR}){SEPARATOR_COLOR}:{VALUES_COLOR} {} / {} (\x1b[0m{}%{VALUES_COLOR}) - {fs}\n", Size::from_bytes(used), Size::from_bytes(total), full));
    }

    Some(out).filter(|s| !s.is_empty())
}
