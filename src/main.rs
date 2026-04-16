/*
 *     qfetch v0.1.9
 * Copyright (C) 2026  Quixaq
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

mod sysinfo;
use console::{measure_text_width, truncate_str};
use std::fmt::Write;
use terminal_size::{Width, terminal_size};

use crate::config::{
    BRIGHT_PALETTE_ENABLED, BRIGHT_PALETTE_KEY, CPU_ENABLED, CPU_KEY, CPU_PRIORITY, CURSOR_ENABLED,
    CURSOR_KEY, CURSOR_PRIORITY, DE_ENABLED, DE_KEY, DE_PRIORITY, GPU_ENABLED, GPU_KEY,
    GPU_PRIORITY, HOST_ENABLED, HOST_KEY, HOST_PRIORITY, KERNEL_ENABLED, KERNEL_KEY,
    KERNEL_PRIORITY, LOCALE_ENABLED, LOCALE_KEY, LOCALE_PRIORITY, LOGO_ENABLED, OS_ENABLED, OS_KEY,
    OS_PRIORITY, RAM_ENABLED, RAM_KEY, RAM_PRIORITY, SHELL_ENABLED, SHELL_KEY, SHELL_PRIORITY,
    STANDARD_PALETTE_ENABLED, STANDARD_PALETTE_KEY, SWAP_ENABLED, SWAP_KEY, SWAP_PRIORITY,
    THEME_ENABLED, THEME_KEY, THEME_PRIORITY, TITLE_ENABLED, TITLE_KEY, UPTIME_ENABLED, UPTIME_KEY,
    UPTIME_PRIORITY, VALUES_COLOR,
};
mod config;

const STANDARD_PALETTE: &str = "\x1b[40m   \x1b[41m   \x1b[42m   \x1b[43m   \x1b[44m   \x1b[45m   \x1b[46m   \x1b[47m   \x1b[0m";
const BRIGHT_PALETTE: &str = "\x1b[100m   \x1b[101m   \x1b[102m   \x1b[103m   \x1b[104m   \x1b[105m   \x1b[106m   \x1b[107m   \x1b[0m";

fn main() {
    let (mut pretty, id, id_like) = sysinfo::distro();
    if !OS_ENABLED {
        pretty = None
    }
    let (title, sep) = if TITLE_ENABLED {
        sysinfo::title()
    } else {
        (None, None)
    };
    let host = if HOST_ENABLED { sysinfo::host() } else { None };
    let uptime = if UPTIME_ENABLED {
        sysinfo::uptime()
    } else {
        None
    };
    let shell = if SHELL_ENABLED {
        sysinfo::shell()
    } else {
        None
    };
    let kernel = if KERNEL_ENABLED {
        sysinfo::kernel()
    } else {
        None
    };
    let de = if DE_ENABLED { sysinfo::de() } else { None };
    let theme = if THEME_ENABLED {
        sysinfo::theme()
    } else {
        None
    };
    let cursor = if CURSOR_ENABLED {
        sysinfo::cursor()
    } else {
        None
    };
    let cpu = if CPU_ENABLED { sysinfo::cpu() } else { None };
    let gpu = if GPU_ENABLED { sysinfo::gpu() } else { None };
    let locale = if LOCALE_ENABLED {
        sysinfo::locale()
    } else {
        None
    };
    let (ram, swap) = if RAM_ENABLED || SWAP_ENABLED {
        let (r, s) = sysinfo::memory();
        (
            if RAM_ENABLED { r } else { None },
            if SWAP_ENABLED { s } else { None },
        )
    } else {
        (None, None)
    };
    let palette_sep = if STANDARD_PALETTE_ENABLED || BRIGHT_PALETTE_ENABLED {
        Some("".to_string())
    } else {
        None
    };
    let standard_palette = if STANDARD_PALETTE_ENABLED {
        Some(STANDARD_PALETTE.to_string())
    } else {
        None
    };
    let bright_palette = if BRIGHT_PALETTE_ENABLED {
        Some(BRIGHT_PALETTE.to_string())
    } else {
        None
    };
    let mut info = [
        (0, TITLE_KEY, title),
        (1, TITLE_KEY, sep),
        (OS_PRIORITY, OS_KEY, pretty),
        (HOST_PRIORITY, HOST_KEY, host),
        (UPTIME_PRIORITY, UPTIME_KEY, uptime),
        (SHELL_PRIORITY, SHELL_KEY, shell),
        (KERNEL_PRIORITY, KERNEL_KEY, kernel),
        (DE_PRIORITY, DE_KEY, de),
        (THEME_PRIORITY, THEME_KEY, theme),
        (CURSOR_PRIORITY, CURSOR_KEY, cursor),
        (CPU_PRIORITY, CPU_KEY, cpu),
        (GPU_PRIORITY, GPU_KEY, gpu),
        (LOCALE_PRIORITY, LOCALE_KEY, locale),
        (RAM_PRIORITY, RAM_KEY, ram),
        (SWAP_PRIORITY, SWAP_KEY, swap),
        (253, "", palette_sep),
        (254, STANDARD_PALETTE_KEY, standard_palette),
        (255, BRIGHT_PALETTE_KEY, bright_palette),
    ];
    info.sort_unstable();
    let mut out = String::with_capacity(256);
    let mut logo = "";
    if LOGO_ENABLED {
        logo = config::get_logo(
            &id.unwrap_or("".to_string()),
            &id_like.unwrap_or("".to_string()),
        );
    }
    let logo_lines: Vec<&str> = logo.lines().collect();
    let terminal_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(usize::MAX);
    let mut line = 0;
    for (_, name, value) in info {
        if let Some(val) = value {
            let out_line = format!(
                "{}{}{}{}\x1b[0m",
                logo_lines.get(line).unwrap_or(&""),
                name,
                VALUES_COLOR,
                val
            );
            let display_line = if measure_text_width(&out_line) > terminal_width {
                let content = truncate_str(&out_line, terminal_width - 1, "");
                format!("{}\x1b[49;2m…\x1b[0m", content)
            } else {
                out_line
            };
            let _ = writeln!(out, "{}", display_line).expect("Failed to print output");
            line += 1;
        }
    }
    if line <= logo_lines.len() {
        for _ in line..logo_lines.len() {
            let display_line = if measure_text_width(logo_lines[line]) > terminal_width {
                let content = truncate_str(logo_lines[line], terminal_width - 1, "");
                format!("{}\x1b[49;2m…\x1b[0m", content)
            } else {
                logo_lines[line].to_string()
            };
            let _ = writeln!(out, "{}", display_line).expect("Failed to print output");
            line += 1;
        }
    }
    print!("{}", out);
}
