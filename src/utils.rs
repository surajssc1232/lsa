use chrono::Duration;
use std::time::SystemTime;

pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else {
        format!("{size} B")
    }
}

pub fn format_time(time: SystemTime) -> String {
    let now = SystemTime::now();
    match now.duration_since(time) {
        Ok(duration) => {
            if duration < Duration::minutes(1).to_std().unwrap() {
                format!("{} seconds ago", duration.as_secs())
            } else if duration < Duration::hours(1).to_std().unwrap() {
                format!("{} minutes ago", duration.as_secs() / 60)
            } else if duration < Duration::days(1).to_std().unwrap() {
                format!("{} hours ago", duration.as_secs() / 3600)
            } else if duration < Duration::weeks(1).to_std().unwrap() {
                format!("{} days ago", duration.as_secs() / (3600 * 24))
            } else if duration < Duration::weeks(4).to_std().unwrap() {
                format!("{} weeks ago", duration.as_secs() / (3600 * 24 * 7))
            } else if duration < Duration::days(365).to_std().unwrap() {
                format!("{} months ago", duration.as_secs() / (3600 * 24 * 30))
            } else {
                format!("{} years ago", duration.as_secs() / (3600 * 24 * 365))
            }
        }
        Err(_) => "Unknown".to_string(),
    }
}

#[cfg(unix)]
pub fn format_permissions(mode: u32) -> String {
    let user = format!(
        "{}{}{}",
        if mode & 0o400 != 0 { "r" } else { "-" },
        if mode & 0o200 != 0 { "w" } else { "-" },
        if mode & 0o100 != 0 { "x" } else { "-" }
    );
    let group = format!(
        "{}{}{}",
        if mode & 0o040 != 0 { "r" } else { "-" },
        if mode & 0o020 != 0 { "w" } else { "-" },
        if mode & 0o010 != 0 { "x" } else { "-" }
    );
    let other = format!(
        "{}{}{}",
        if mode & 0o004 != 0 { "r" } else { "-" },
        if mode & 0o002 != 0 { "w" } else { "-" },
        if mode & 0o001 != 0 { "x" } else { "-" }
    );
    format!("{user}{group}{other}")
}

#[cfg(windows)]
pub fn format_permissions(_metadata: &std::fs::Metadata) -> String {
    "N/A".to_string()
}

pub fn colorize_borders(table_str: &str, theme: &crate::themes::Theme) -> String {
    let border_color = format!(
        "\x1b[38;2;{};{};{}m",
        theme.border.0, theme.border.1, theme.border.2
    );
    let reset_color = "\x1b[0m";

    table_str
        .lines()
        .map(|line| {
            let mut colored_line = String::new();
            for ch in line.chars() {
                match ch {
                    '╭' | '╮' | '╰' | '╯' | '│' | '─' | '┼' | '├' | '┤' | '┬' | '┴' | '═' | '╞'
                    | '╡' => {
                        colored_line.push_str(&format!("{border_color}{ch}{reset_color}"));
                    }
                    _ => {
                        colored_line.push(ch);
                    }
                }
            }
            colored_line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

