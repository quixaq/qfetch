use console::measure_text_width;
use serde::Deserialize;
use std::{env, fs, path::Path};

#[derive(Deserialize)]
struct Module {
    id: String,
    enabled: bool,
    key: String,
}

#[derive(Deserialize)]
struct Modules {
    general: Vec<Module>,
}

#[derive(Deserialize)]
struct TitleSettings {
    separator_char: String,
}

#[derive(Deserialize)]
struct ThresholdSettings {
    medium: usize,
    high: usize,
}

#[derive(Deserialize)]
struct RamSettings {
    thresholds: ThresholdSettings,
}

#[derive(Deserialize)]
struct SwapSettings {
    thresholds: ThresholdSettings,
}

#[derive(Deserialize)]
struct MountsSettings {
    thresholds: ThresholdSettings,
}

#[derive(Deserialize)]
struct ModuleSettings {
    title: TitleSettings,
    ram: RamSettings,
    swap: SwapSettings,
    mounts: MountsSettings,
}

#[derive(Deserialize)]
struct Colors {
    title: String,
    title_sep: String,
    keys: String,
    separator: String,
    values: String,
    mountpoints: String,
    low: String,
    medium: String,
    high: String,
}

#[derive(Deserialize)]
struct LogoElement {
    id: String,
    colors: Vec<String>,
}

#[derive(Deserialize)]
struct Padding {
    left: usize,
    right: usize,
    char: String,
    replace_spaces_with_char: bool,
}

#[derive(Deserialize)]
struct Logo {
    enabled: bool,
    padding: Padding,
    include: Vec<LogoElement>,
}

#[derive(Deserialize)]
struct Config {
    modules: Modules,
    module_settings: ModuleSettings,
    colors: Colors,
    logo: Logo,
}

fn parse_x1b(s: String) -> String {
    if s.starts_with("#") {
        if s.len() == 9 {
            if &s[7..9].to_lowercase() != "ff" {
                {
                    cargo_build::error(format!("transparency is not supported: {}", s).as_str());
                    std::process::exit(1)
                }
            }
        }
        let r = match s.len() {
            4 => {
                usize::from_str_radix(&s[1..2].repeat(2), 16).expect(&format!("invalid hex: {}", s))
            }
            7 | 9 => usize::from_str_radix(&s[1..3], 16).expect(&format!("invalid hex: {}", s)),
            _ => {
                cargo_build::error(format!("invalid hex: {}", s).as_str());
                std::process::exit(1)
            }
        };
        let g = match s.len() {
            4 => {
                usize::from_str_radix(&s[2..3].repeat(2), 16).expect(&format!("invalid hex: {}", s))
            }
            7 | 9 => usize::from_str_radix(&s[3..5], 16).expect(&format!("invalid hex: {}", s)),
            _ => {
                cargo_build::error(format!("invalid hex: {}", s).as_str());
                std::process::exit(1)
            }
        };
        let b = match s.len() {
            4 => {
                usize::from_str_radix(&s[3..4].repeat(2), 16).expect(&format!("invalid hex: {}", s))
            }
            7 | 9 => usize::from_str_radix(&s[5..7], 16).expect(&format!("invalid hex: {}", s)),
            _ => {
                cargo_build::error(format!("invalid hex: {}", s).as_str());
                std::process::exit(1)
            }
        };
        return format!("\x1b[38;2;{r};{g};{b};m");
    } else if s.starts_with("a") {
        let color =
            usize::from_str_radix(&s[1..3], 10).expect(&format!("invalid ansi color: {}", s));
        if color < 30 || color > 97 || (color > 37 && color < 90) {
            cargo_build::error(format!("invalid ansi color: {}", s).as_str());
            std::process::exit(1)
        };
        return format!("\x1b[{}m", color);
    } else {
        cargo_build::error(format!("invalid color: {}", s).as_str());
        std::process::exit(1)
    }
}

fn main() {
    println!("cargo:rerun-if-changed=config.yaml");
    println!("cargo:rerun-if-env-changed=CONFIG_FILE_PATH");

    color_backtrace::install();

    let config_path = env::var("CONFIG_FILE_PATH").unwrap_or_else(|_| "config.yaml".to_string());
    let yaml_str = fs::read_to_string(&config_path).expect("Failed to read config.yaml");
    let config: Config = serde_yaml::from_str(&yaml_str).expect("Failed to parse YAML config.");

    let mut constants: Vec<String> = Vec::new();

    let keys = parse_x1b(config.colors.keys);
    constants.push(format!(
        "pub const KEYS_COLOR: &'static str = \"{}\";",
        keys
    ));

    let separator = parse_x1b(config.colors.separator);
    constants.push(format!(
        "pub const SEPARATOR_COLOR: &'static str = \"{}\";",
        separator
    ));

    let mut info: Vec<(usize, String, String)> = Vec::new();

    for (index, module) in config.modules.general.iter().enumerate() {
        // the following block is here to block the module id as a vector for arbitrary code execution
        if !matches!(
            module.id.as_str(),
            "title"
                | "os"
                | "host"
                | "kernel"
                | "uptime"
                | "shell"
                | "de"
                | "theme"
                | "cursor"
                | "cpu"
                | "gpu"
                | "ram"
                | "swap"
                | "mounts"
                | "locale"
                | "standard_palette"
                | "bright_palette"
        ) {
            panic!("Inexistent module id! Found: \"{}\"", module.id)
        }

        let mut key = module.key.clone();
        if !key.is_empty() && module.id != "mounts" {
            key = format!("{}{key}{}:\\x1b[0m ", keys, separator)
        }

        match module.id.as_str() {
            "standard_palette" => info.push((254, key, "standard_palette".to_owned())),
            "bright_palette" => info.push((255, key, "bright_palette".to_owned())),
            "title" => {
                info.push((0, key.clone(), "title".to_owned()));
                info.push((1, key.clone(), "sep".to_owned()));
            }
            "mounts" => {
                constants.push(format!("pub const MOUNTS_KEY: &'static str = \"{}\";", key));
                info.push((index + 1, "".to_owned(), module.id.clone()));
            }
            _ => {
                info.push((index + 1, key.clone(), module.id.clone()));
            }
        };

        constants.push(format!(
            "pub const {}_ENABLED: bool = {};",
            module.id.to_uppercase(),
            module.enabled
        ));
    }

    info.push((253, "".to_owned(), "palette_sep".to_owned()));

    let values = parse_x1b(config.colors.values);
    constants.push(format!(
        "pub const VALUES_COLOR: &'static str = \"{}\";",
        values
    ));

    let title = parse_x1b(config.colors.title);
    constants.push(format!(
        "pub const TITLE_COLOR: &'static str = \"{}\";",
        title
    ));

    let title_sep = parse_x1b(config.colors.title_sep);
    constants.push(format!(
        "pub const TITLE_SEP_COLOR: &'static str = \"{}\";",
        title_sep
    ));

    let mountpoint_color = parse_x1b(config.colors.mountpoints);
    constants.push(format!(
        "pub const MOUNTSPOINT_COLOR: &'static str = \"{}\";",
        mountpoint_color
    ));

    let low = parse_x1b(config.colors.low);
    constants.push(format!("pub const LOW_COLOR: &'static str = \"{}\";", low));

    let medium = parse_x1b(config.colors.medium);
    constants.push(format!(
        "pub const MEDIUM_COLOR: &'static str = \"{}\";",
        medium
    ));

    let high = parse_x1b(config.colors.high);
    constants.push(format!(
        "pub const HIGH_COLOR: &'static str = \"{}\";",
        high
    ));

    constants.push(format!(
        "pub const TITLE_SEP_CHAR: &'static str = \"{}\";",
        config.module_settings.title.separator_char
    ));

    let ram_medium_threshold = config.module_settings.ram.thresholds.medium;
    let ram_high_threshold = config.module_settings.ram.thresholds.high;
    if ram_medium_threshold > 100 {
        panic!("ram medium threshold is higher than 100%")
    }
    if ram_high_threshold > 100 {
        panic!("ram high threshold is higher than 100%")
    }
    if ram_medium_threshold > ram_high_threshold {
        panic!(
            "ram medium threshold is higher than mounts high threshold, consider changing the colors instead"
        )
    }
    constants.push(format!(
        "pub const RAM_MEDIUM: usize = {};",
        ram_medium_threshold
    ));
    constants.push(format!(
        "pub const RAM_HIGH: usize = {};",
        ram_high_threshold
    ));

    let swap_medium_threshold = config.module_settings.swap.thresholds.medium;
    let swap_high_threshold = config.module_settings.swap.thresholds.high;
    if swap_medium_threshold > 100 {
        panic!("swap medium threshold is higher than 100%")
    }
    if swap_high_threshold > 100 {
        panic!("swap high threshold is higher than 100%")
    }
    if swap_medium_threshold > swap_high_threshold {
        panic!(
            "swap medium threshold is higher than mounts high threshold, consider changing the colors instead"
        )
    }
    constants.push(format!(
        "pub const SWAP_MEDIUM: usize = {};",
        swap_medium_threshold
    ));
    constants.push(format!(
        "pub const SWAP_HIGH: usize = {};",
        swap_high_threshold
    ));

    let mounts_medium_threshold = config.module_settings.mounts.thresholds.medium;
    let mounts_high_threshold = config.module_settings.mounts.thresholds.high;
    if mounts_medium_threshold > 100 {
        panic!("mounts medium threshold is higher than 100%")
    }
    if mounts_high_threshold > 100 {
        panic!("mounts high threshold is higher than 100%")
    }
    if mounts_medium_threshold > mounts_high_threshold {
        panic!(
            "mounts medium threshold is higher than mounts high threshold, consider changing the colors instead"
        )
    }
    constants.push(format!(
        "pub const MOUNTS_MEDIUM: usize = {};",
        mounts_medium_threshold
    ));
    constants.push(format!(
        "pub const MOUNTS_HIGH: usize = {};",
        mounts_high_threshold
    ));

    let logo_enabled = config.logo.enabled;
    constants.push(format!("pub const LOGO_ENABLED: bool = {};", logo_enabled));

    let included_logos = config.logo.include;
    let mut logos: Vec<String> = Vec::new();
    for (index, logo) in included_logos.iter().enumerate() {
        if logo.id == "*" {}
        let text = fs::read_to_string(format!("logo/{}.txt", logo.id))
            .expect(&format!("Logo {} doesn't exist.", logo.id));
        let colors = &logo.colors;
        let mut iter = text.chars().peekable();
        let mut out = String::new();
        while let Some(char) = iter.next() {
            if char == '$' {
                if let Some(next_char) = iter.peek() {
                    if next_char.is_ascii_digit() {
                        let digit_char = iter.next().unwrap();
                        let digit = digit_char.to_digit(10).unwrap() as usize;
                        if let Some(color) = colors.get(digit.saturating_sub(1)) {
                            out.push_str(&parse_x1b(color.to_owned()));
                            continue;
                        }
                    }
                }
            }
            out.push(char);
        }
        let lines: Vec<&str> = out.lines().collect();
        let target = lines
            .iter()
            .map(|line| measure_text_width(line))
            .max()
            .unwrap_or(0)
            + config.logo.padding.right;
        let mut padded_out = String::new();
        for line in lines {
            let width = measure_text_width(line);
            let padding = config.logo.padding.char.repeat(target - width);
            padded_out.push_str(&format!(
                "{}{}\x1b[0m{}\\n",
                config.logo.padding.char.repeat(config.logo.padding.left),
                match config.logo.padding.replace_spaces_with_char {
                    true => line.replace(" ", config.logo.padding.char.as_str()),
                    false => line.to_string(),
                },
                padding
            ));
        }
        logos.push(logo.id.to_string());
        if index == 0 {
            constants.push(format!(
                "const LOGO_FALLBACK: &'static str = \"{}\";",
                padded_out.to_string()
            ));
        }
        constants.push(format!(
            "const {}_LOGO: &'static str = \"{}\";",
            logo.id.to_uppercase(),
            padded_out.to_string()
        ));
    }

    let mut match_arms: Vec<String> = Vec::new();
    for logo in logos {
        match_arms.push(format!("\"{}\" => {}_LOGO,", logo, logo.to_uppercase()));
    }

    constants.push(format!(
        "\npub fn get_logo(id: &str, fallback_id: &str) -> &'static str {{\n\tmatch id {{\n\t\t{}\n\t\t_ => match fallback_id {{\n\t\t\t{}\n\t\t\t_ => LOGO_FALLBACK,\n\t\t}},\n\t}}\n}}",
        match_arms.join("\n\t\t"),
        match_arms.join("\n\t\t\t")
    ));

    let mut modules: Vec<String> = Vec::new();

    modules.push("    {\n        [".to_string());
    info.sort_unstable();
    for item in info {
        modules.push(format!("            (\"{}\", {}),", item.1, item.2));
    }
    modules.push("        ]\n    }".to_string());

    let code = format!(
        "// The configuration can be edited in config.yaml. This file is generated on build based on the YAML config.\n\n{}\n",
        constants.join("\n")
    );

    let code2 = format!(
        "// This block has been generated by the build script\n\n{}\n",
        modules.join("\n")
    );

    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("config.rs");
    let path2 = Path::new(&out_dir).join("modules.rs");
    fs::write(path, code).unwrap();
    fs::write(path2, code2).unwrap()
}
