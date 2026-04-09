use size::Size;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use gethostname::gethostname;

use crate::config::TITLE_COLOR;

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
    let mut content = fs::read_to_string("/proc/sys/kernel/osrelease").ok()?;
    content.truncate(content.trim_end().len());
    Some(format!("Linux {content}"))
}

pub fn host() -> Option<String> {
    let content = fs::read_to_string("/sys/class/dmi/id/product_name").ok()?;
    let product = content.trim();
    Some(product.to_owned())
}

pub fn uptime() -> Option<String> {
    let content = fs::read_to_string("/proc/uptime").ok()?;
    let seconds_str = content.split_whitespace().next()?.split('.').next()?;
    let seconds = seconds_str.parse::<u64>().ok()?;
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

    Some(out)
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

fn parse_kb(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

pub fn memory() -> (Option<String>, Option<String>) {
    let Ok(file) = File::open("/proc/meminfo") else {
        return (None, None);
    };
    let reader = BufReader::new(file);

    let mut total = 0;
    let mut available = 0;
    let mut total_swap = 0;
    let mut free_swap = 0;

    let mut found: usize = 0;

    for line in reader.lines().map_while(Result::ok) {
        if line.starts_with("MemTotal:") {
            total = parse_kb(&line);
            found += 1;
        } else if line.starts_with("MemAvailable:") {
            available = parse_kb(&line);
            found += 1;
        } else if line.starts_with("SwapTotal:") {
            total_swap = parse_kb(&line);
            found += 1;
        } else if line.starts_with("SwapFree:") {
            free_swap = parse_kb(&line);
            found += 1;
        }
        if found == 4 {
            break;
        }
    }

    let used = Size::from_kibibytes(total - available);
    let total_gib = Size::from_kibibytes(total);
    let used_swap = Size::from_kibibytes(total_swap - free_swap);
    let total_swap_gib = Size::from_kibibytes(total_swap);

    (
        Some(format!("{} / {}", used, total_gib)),
        Some(format!("{} / {}", used_swap, total_swap_gib)),
    )
}

pub fn gpu() -> Option<String> {
    getgpuname::get_gpu_name()
}

pub fn locale() -> Option<String> {
    std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LANG"))
        .ok()
}
