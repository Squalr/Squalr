use crate::commands::command_line::clap;
pub fn format_prompt_command_error(error: &clap::Error) -> String {
    let normalized_message = normalize_prompt_command_message(&error.message);

    match error.kind {
        clap::ErrorKind::HelpDisplayed => format_prompt_command_help(&normalized_message),
        clap::ErrorKind::VersionDisplayed => format_prompt_command_version(&normalized_message),
        _ => summarize_prompt_command_error(&normalized_message),
    }
}

fn normalize_prompt_command_message(message: &str) -> String {
    message
        .lines()
        .filter(|line| !line.trim_start().starts_with("For more information try"))
        .map(strip_prompt_command_usage_padding)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn strip_prompt_command_usage_padding(line: &str) -> &str {
    if line.trim().is_empty() {
        return "";
    }

    line.strip_prefix("    ").unwrap_or(line)
}

fn summarize_prompt_command_error(message: &str) -> String {
    let mut summary_lines = Vec::new();

    if let Some(first_error_line) = message.lines().find(|line| !line.trim().is_empty()) {
        summary_lines.push(first_error_line.trim().to_string());
    }

    if let Some(usage_line) = prompt_command_usage_line(message) {
        summary_lines.push(format!("Usage: {}", usage_line.trim()));
    }

    summary_lines.join("\n")
}

fn prompt_command_usage_line(message: &str) -> Option<&str> {
    let mut lines = message.lines();

    while let Some(line) = lines.next() {
        if line.trim() == "USAGE:" {
            return lines.find(|usage_line| !usage_line.trim().is_empty());
        }
    }

    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PromptCommandHelpSection {
    Usage,
    Args,
    Options,
    Flags,
    Subcommands,
}

fn format_prompt_command_help(message: &str) -> String {
    let mut usage_line = None;
    let mut arg_items = Vec::new();
    let mut option_items = Vec::new();
    let mut flag_items = Vec::new();
    let mut subcommand_items = Vec::new();
    let mut current_section = None;

    for line in message.lines() {
        let trimmed_line = line.trim();

        if trimmed_line.is_empty() {
            continue;
        }

        if let Some(section) = prompt_command_help_section(trimmed_line) {
            current_section = Some(section);
            continue;
        }

        match current_section {
            Some(PromptCommandHelpSection::Usage) if usage_line.is_none() => usage_line = Some(trimmed_line.to_string()),
            Some(PromptCommandHelpSection::Args) => {
                if let Some(arg_item) = compact_prompt_help_item(trimmed_line) {
                    arg_items.push(arg_item);
                }
            }
            Some(PromptCommandHelpSection::Options) => {
                if let Some(option_item) = compact_prompt_help_item(trimmed_line) {
                    option_items.push(option_item);
                }
            }
            Some(PromptCommandHelpSection::Flags) => {
                if let Some(flag_item) = compact_prompt_help_item(trimmed_line) {
                    flag_items.push(flag_item);
                }
            }
            Some(PromptCommandHelpSection::Subcommands) => {
                if let Some(subcommand_item) = compact_prompt_help_item(trimmed_line) {
                    subcommand_items.push(subcommand_item);
                }
            }
            _ => {}
        }
    }

    let mut output_lines = Vec::new();

    if let Some(usage_line) = usage_line {
        output_lines.push(format!("Usage: {}", usage_line));
    }

    push_prompt_help_section(&mut output_lines, "Commands", &subcommand_items);
    push_prompt_help_section(&mut output_lines, "Args", &arg_items);
    push_prompt_help_section(&mut output_lines, "Options", &option_items);
    push_prompt_help_section(&mut output_lines, "Flags", &flag_items);

    if output_lines.is_empty() {
        return message.to_string();
    }

    output_lines.join("\n")
}

fn format_prompt_command_version(message: &str) -> String {
    message
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .unwrap_or("Version unavailable")
        .to_string()
}

fn prompt_command_help_section(line: &str) -> Option<PromptCommandHelpSection> {
    match line {
        "USAGE:" => Some(PromptCommandHelpSection::Usage),
        "ARGS:" => Some(PromptCommandHelpSection::Args),
        "OPTIONS:" => Some(PromptCommandHelpSection::Options),
        "FLAGS:" => Some(PromptCommandHelpSection::Flags),
        "SUBCOMMANDS:" => Some(PromptCommandHelpSection::Subcommands),
        _ => None,
    }
}

fn compact_prompt_help_item(line: &str) -> Option<String> {
    let trimmed_line = line.trim();

    if trimmed_line.is_empty() || is_prompt_help_noise_item(trimmed_line) {
        return None;
    }

    let Some(description_split_offset) = find_help_item_description_split(trimmed_line) else {
        return Some(trimmed_line.to_string());
    };

    let item_name = trimmed_line[..description_split_offset].trim();
    let item_description = trimmed_line[description_split_offset..].trim();

    if item_description.is_empty() {
        Some(item_name.to_string())
    } else {
        Some(format!("{} - {}", item_name, item_description))
    }
}

fn is_prompt_help_noise_item(line: &str) -> bool {
    line.starts_with('-') && (line.contains("--help") || line.contains("--version"))
}

fn find_help_item_description_split(line: &str) -> Option<usize> {
    let mut whitespace_start_offset = None;
    let mut whitespace_count = 0;

    for (byte_offset, character) in line.char_indices() {
        if character.is_whitespace() {
            if whitespace_count == 0 {
                whitespace_start_offset = Some(byte_offset);
            }

            whitespace_count += 1;

            if whitespace_count >= 2 {
                return whitespace_start_offset;
            }
        } else {
            whitespace_start_offset = None;
            whitespace_count = 0;
        }
    }

    None
}

fn push_prompt_help_section(
    output_lines: &mut Vec<String>,
    section_label: &str,
    section_items: &[String],
) {
    if !section_items.is_empty() {
        output_lines.push(format!("{}: {}", section_label, section_items.join("; ")));
    }
}
