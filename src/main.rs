/*
 *     qfetch v0.1.7
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
    let (pretty, id, id_like) = sysinfo::distro();
    let (title, sep) = sysinfo::title();
    let (ram, swap) = sysinfo::memory();
    let mut info = [
        (0, TITLE_ENABLED, TITLE_KEY, title),
        (1, TITLE_ENABLED, TITLE_KEY, sep),
        (OS_PRIORITY, OS_ENABLED, OS_KEY, pretty),
        (HOST_PRIORITY, HOST_ENABLED, HOST_KEY, sysinfo::host()),
        (
            UPTIME_PRIORITY,
            UPTIME_ENABLED,
            UPTIME_KEY,
            sysinfo::uptime(),
        ),
        (SHELL_PRIORITY, SHELL_ENABLED, SHELL_KEY, sysinfo::shell()),
        (
            KERNEL_PRIORITY,
            KERNEL_ENABLED,
            KERNEL_KEY,
            sysinfo::kernel(),
        ),
        (DE_PRIORITY, DE_ENABLED, DE_KEY, sysinfo::de()),
        (THEME_PRIORITY, THEME_ENABLED, THEME_KEY, sysinfo::theme()),
        (
            CURSOR_PRIORITY,
            CURSOR_ENABLED,
            CURSOR_KEY,
            sysinfo::cursor(),
        ),
        (CPU_PRIORITY, CPU_ENABLED, CPU_KEY, sysinfo::cpu()),
        (GPU_PRIORITY, GPU_ENABLED, GPU_KEY, sysinfo::gpu()),
        (
            LOCALE_PRIORITY,
            LOCALE_ENABLED,
            LOCALE_KEY,
            sysinfo::locale(),
        ),
        (RAM_PRIORITY, RAM_ENABLED, RAM_KEY, ram),
        (SWAP_PRIORITY, SWAP_ENABLED, SWAP_KEY, swap),
        (
            997,
            STANDARD_PALETTE_ENABLED || BRIGHT_PALETTE_ENABLED,
            "",
            Some("".to_string()),
        ),
        (
            998,
            STANDARD_PALETTE_ENABLED,
            STANDARD_PALETTE_KEY,
            Some(STANDARD_PALETTE.to_string()),
        ),
        (
            999,
            BRIGHT_PALETTE_ENABLED,
            BRIGHT_PALETTE_KEY,
            Some(BRIGHT_PALETTE.to_string()),
        ),
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
    for (_, enabled, name, value) in info {
        if let Some(val) = value
            && enabled
        {
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
