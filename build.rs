use console::measure_text_width;
use serde::Deserialize;
use std::{env, fs};

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
struct Colors {
    title: String,
    keys: String,
    separator: String,
    values: String,
}

#[derive(Deserialize)]
struct LogoElement {
    id: String,
    colors: Vec<String>,
}

#[derive(Deserialize)]
struct Logo {
    enabled: bool,
    include: Vec<LogoElement>,
}

#[derive(Deserialize)]
struct Config {
    modules: Modules,
    colors: Colors,
    logo: Logo,
}

fn main() {
    println!("cargo:rerun-if-changed=config.yaml");
    println!("cargo:rerun-if-env-changed=CONFIG_FILE_PATH");

    let config_path = env::var("CONFIG_FILE_PATH").unwrap_or_else(|_| "config.yaml".to_string());
    let yaml_str = fs::read_to_string(&config_path).expect("Failed to read config.yaml");
    let config: Config = serde_yaml::from_str(&yaml_str).expect("Failed to parse YAML config.");

    let mut constants: Vec<String> = Vec::new();

    let keys = config.colors.keys;
    let keys_r = u32::from_str_radix(&keys[1..3], 16).expect("Invalid keys hex color.");
    let keys_g = u32::from_str_radix(&keys[3..5], 16).expect("Invalid keys hex color.");
    let keys_b = u32::from_str_radix(&keys[5..7], 16).expect("Invalid keys hex color.");

    let separator = config.colors.separator;
    let separator_r =
        u32::from_str_radix(&separator[1..3], 16).expect("Invalid separator hex color.");
    let separator_g =
        u32::from_str_radix(&separator[3..5], 16).expect("Invalid separator hex color.");
    let separator_b =
        u32::from_str_radix(&separator[5..7], 16).expect("Invalid separator hex color.");

    for (index, module) in config.modules.general.iter().enumerate() {
        if module.id != "standard_palette" && module.id != "bright_palette" && module.id != "title"
        {
            constants.push(format!(
                "pub const {}_PRIORITY: usize = {};",
                module.id.to_uppercase(),
                index + 1
            ));
        }
        constants.push(format!(
            "pub const {}_ENABLED: bool = {};",
            module.id.to_uppercase(),
            module.enabled
        ));
        let mut key = module.key.clone();
        if !key.is_empty() {
            key = format!(
                "\\x1b[38;2;{keys_r};{keys_g};{keys_b};m{key}\\x1b[38;2;{separator_r};{separator_g};{separator_b};m:\\x1b[0m "
            )
        }
        constants.push(format!(
            "pub const {}_KEY: &'static str = \"{}\";",
            module.id.to_uppercase(),
            key
        ));
    }

    let values = config.colors.values;
    let values_r = u32::from_str_radix(&values[1..3], 16).expect("Invalid values hex color.");
    let values_g = u32::from_str_radix(&values[3..5], 16).expect("Invalid values hex color.");
    let values_b = u32::from_str_radix(&values[5..7], 16).expect("Invalid values hex color.");
    constants.push(format!(
        "pub const VALUES_COLOR: &'static str = \"{}\";",
        format!("\\x1b[38;2;{values_r};{values_g};{values_b};m")
    ));

    let title = config.colors.title;
    let title_r = u32::from_str_radix(&title[1..3], 16).expect("Invalid values hex color.");
    let title_g = u32::from_str_radix(&title[3..5], 16).expect("Invalid values hex color.");
    let title_b = u32::from_str_radix(&title[5..7], 16).expect("Invalid values hex color.");
    constants.push(format!(
        "pub const TITLE_COLOR: &'static str = \"{}\";",
        format!("\\x1b[38;2;{title_r};{title_g};{title_b};m")
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
                            let color_r = u32::from_str_radix(&color[1..3], 16)
                                .expect("Invalid values hex color.");
                            let color_g = u32::from_str_radix(&color[3..5], 16)
                                .expect("Invalid values hex color.");
                            let color_b = u32::from_str_radix(&color[5..7], 16)
                                .expect("Invalid values hex color.");
                            let color_ansi = format!("\x1b[38;2;{color_r};{color_g};{color_b};m");
                            out.push_str(&color_ansi);
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
            + 4;
        let mut padded_out = String::new();
        for line in lines {
            let width = measure_text_width(line);
            let padding = " ".repeat(target - width);
            padded_out.push_str(&format!(
                "{}\x1b[0m{}\\n",
                line.replace("\x1b", "\\x1b"),
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

    let code = format!(
        "// The configuration can be edited in config.yaml. This file is generated on build based on the YAML config.\n\n{}\n",
        constants.join("\n")
    );

    fs::write("src/config.rs", code).unwrap()
}
