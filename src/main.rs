// SPDX-FileCopyrightText: 2026 Quixaq
// SPDX-License-Identifier: GPL-3.0-or-later

/*
 *     qfetch v0.4.1
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

include!(concat!(env!("OUT_DIR"), "/config.rs"));

mod sysinfo;
use console::{measure_text_width, truncate_str};
use std::fmt::Write;
use terminal_size::{Width, terminal_size};

#[cfg(standard_palette)]
const STANDARD_PALETTE: &str = "\x1b[40m   \x1b[41m   \x1b[42m   \x1b[43m   \x1b[44m   \x1b[45m   \x1b[46m   \x1b[47m   \x1b[0m";
#[cfg(bright_palette)]
const BRIGHT_PALETTE: &str = "\x1b[100m   \x1b[101m   \x1b[102m   \x1b[103m   \x1b[104m   \x1b[105m   \x1b[106m   \x1b[107m   \x1b[0m";

fn main() {
    let (os, id, id_like) = sysinfo::distro();

    #[cfg(title)]
    let (title, sep) = sysinfo::title();
    #[cfg(host)]
    let host = sysinfo::host();
    #[cfg(shell)]
    let shell = sysinfo::shell();
    #[cfg(kernel)]
    let kernel = sysinfo::kernel();
    #[cfg(de)]
    let de = sysinfo::de();
    #[cfg(theme)]
    let theme = sysinfo::theme();
    #[cfg(cursor)]
    let cursor = sysinfo::cursor();
    #[cfg(cpu)]
    let cpu = sysinfo::cpu();
    #[cfg(gpu)]
    let gpu = sysinfo::gpu();
    #[cfg(any(uptime, ram, swap))]
    let (uptime, ram, swap) = sysinfo::sysinfo();
    #[cfg(mounts)]
    let mounts = sysinfo::mounts();
    #[cfg(ip)]
    let ip = sysinfo::local_ip();
    #[cfg(locale)]
    let locale = sysinfo::locale();
    #[cfg(any(standard_palette, bright_palette))]
    let palette_sep = Some("\n".to_string());
    #[cfg(standard_palette)]
    let standard_palette = Some(STANDARD_PALETTE.to_string());
    #[cfg(bright_palette)]
    let bright_palette = Some(BRIGHT_PALETTE.to_string());

    let info = include!(concat!(env!("OUT_DIR"), "/modules.rs"));

    let mut out = String::with_capacity(256);

    #[cfg(logo)]
    let logo = get_logo(
        &id.unwrap_or("".to_string()),
        &id_like.unwrap_or("".to_string()),
    );

    #[cfg(not(logo))]
    let logo = "";

    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_line_fallback = logo
        .lines()
        .next()
        .map_or("".to_string(), |line| " ".repeat(measure_text_width(line)));
    let terminal_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(usize::MAX);
    let mut line_i = 0;
    for (name, value) in info {
        if let Some(val) = value {
            for line in val.lines() {
                let out_line = format!(
                    "{}{}{}{}\x1b[0m",
                    logo_lines
                        .get(line_i)
                        .unwrap_or(&logo_line_fallback.as_str()),
                    name,
                    VALUES_COLOR,
                    line
                );
                let display_line = if measure_text_width(&out_line) > terminal_width {
                    let content = truncate_str(&out_line, terminal_width - 1, "");
                    format!("{}\x1b[49;2m…\x1b[0m", content)
                } else {
                    out_line
                };
                let _ = writeln!(out, "{}", display_line).expect("Failed to print output");
                line_i += 1;
            }
        }
    }
    if line_i <= logo_lines.len() {
        for _ in line_i..logo_lines.len() {
            let display_line = if measure_text_width(logo_lines[line_i]) > terminal_width {
                let content = truncate_str(logo_lines[line_i], terminal_width - 1, "");
                format!("{}\x1b[49;2m…\x1b[0m", content)
            } else {
                logo_lines[line_i].to_string()
            };
            let _ = writeln!(out, "{}", display_line).expect("Failed to print output");
            line_i += 1;
        }
    }
    print!("{}", out);
}
