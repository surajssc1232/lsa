mod workspace;
use clap::{Parser, Subcommand};
use comfy_table::{
    Attribute, Cell, Color, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_BORDERS_ONLY,
};
use inquire::Select;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::SystemTime;
use chrono::{Duration, Local};

#[derive(Parser)]
#[command(name = "lsr")]
#[command(about = "A colorful directory listing tool with multiple themes")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show workspace snapshot
    Workspace,
    /// Select or change theme
    Theme {
        /// Theme name (optional, shows interactive menu if not provided)
        name: Option<String>,
        /// Set theme as default for future use
        #[arg(long)]
        set: bool,
    },
}

#[derive(Clone)]
struct Theme {
    name: String,
    description: String,
    border: (u8, u8, u8),
    header: (u8, u8, u8),
    file_name: (u8, u8, u8),
    file_type: (u8, u8, u8),
    dir_name: (u8, u8, u8),
    dir_type: (u8, u8, u8),
    file_size: (u8, u8, u8),
    dir_size: (u8, u8, u8),
    modified: (u8, u8, u8),
    permissions: (u8, u8, u8),
    row_number: (u8, u8, u8),
}

#[derive(Serialize, Deserialize)]
struct Config {
    default_theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_theme: "catppuccin".to_string(),
        }
    }
}

fn get_themes() -> Vec<Theme> {
    vec![
        Theme {
            name: "catppuccin".to_string(),
            description: "Catppuccin - Soothing pastel theme".to_string(),
            border: (183, 189, 248),      // Lavender
            header: (203, 166, 247),      // Mauve
            file_name: (116, 199, 236),   // Sky
            file_type: (116, 199, 236),   // Sky
            dir_name: (137, 180, 250),    // Blue
            dir_type: (137, 180, 250),    // Blue
            file_size: (166, 227, 161),   // Green
            dir_size: (108, 112, 134),    // Overlay0
            modified: (249, 226, 175),    // Peach
            permissions: (148, 226, 213), // Teal
            row_number: (205, 214, 244),  // Text
        },
        Theme {
            name: "nord".to_string(),
            description: "Nord - Arctic, north-bluish color palette".to_string(),
            border: (136, 192, 208),      // Nord8
            header: (143, 188, 187),      // Nord7
            file_name: (129, 161, 193),   // Nord9
            file_type: (129, 161, 193),   // Nord9
            dir_name: (94, 129, 172),     // Nord10
            dir_type: (94, 129, 172),     // Nord10
            file_size: (163, 190, 140),   // Nord14
            dir_size: (76, 86, 106),      // Nord1
            modified: (235, 203, 139),    // Nord13
            permissions: (180, 142, 173), // Nord15
            row_number: (216, 222, 233),  // Nord4
        },
        Theme {
            name: "dracula".to_string(),
            description: "Dracula - Dark theme with vibrant colors".to_string(),
            border: (189, 147, 249),      // Purple
            header: (255, 121, 198),      // Pink
            file_name: (139, 233, 253),   // Cyan
            file_type: (139, 233, 253),   // Cyan
            dir_name: (189, 147, 249),    // Purple
            dir_type: (189, 147, 249),    // Purple
            file_size: (80, 250, 123),    // Green
            dir_size: (98, 114, 164),     // Comment
            modified: (255, 184, 108),    // Orange
            permissions: (241, 250, 140), // Yellow
            row_number: (248, 248, 242),  // Foreground
        },
        Theme {
            name: "monokai".to_string(),
            description: "Monokai - Classic vibrant dark theme".to_string(),
            border: (174, 129, 255),     // Purple
            header: (255, 97, 136),      // Pink
            file_name: (120, 220, 232),  // Cyan
            file_type: (120, 220, 232),  // Cyan
            dir_name: (174, 129, 255),   // Purple
            dir_type: (174, 129, 255),   // Purple
            file_size: (158, 206, 106),  // Green
            dir_size: (117, 113, 94),    // Comment
            modified: (255, 216, 102),   // Yellow
            permissions: (255, 97, 136), // Pink
            row_number: (248, 248, 242), // White
        },
        Theme {
            name: "gruvbox".to_string(),
            description: "Gruvbox - Retro groove warm color scheme".to_string(),
            border: (214, 153, 108),     // Orange
            header: (251, 73, 52),       // Red
            file_name: (131, 165, 152),  // Aqua
            file_type: (131, 165, 152),  // Aqua
            dir_name: (69, 133, 136),    // Blue
            dir_type: (69, 133, 136),    // Blue
            file_size: (152, 151, 26),   // Green
            dir_size: (146, 131, 116),   // Gray
            modified: (215, 153, 33),    // Yellow
            permissions: (177, 98, 134), // Purple
            row_number: (235, 219, 178), // Foreground
        },
        Theme {
            name: "solarized".to_string(),
            description: "Solarized - Precision colors for machines and people".to_string(),
            border: (42, 161, 152),       // Cyan
            header: (211, 54, 130),       // Magenta
            file_name: (38, 139, 210),    // Blue
            file_type: (38, 139, 210),    // Blue
            dir_name: (42, 161, 152),     // Cyan
            dir_type: (42, 161, 152),     // Cyan
            file_size: (133, 153, 0),     // Green
            dir_size: (101, 123, 131),    // Base00
            modified: (181, 137, 0),      // Yellow
            permissions: (108, 113, 196), // Violet
            row_number: (131, 148, 150),  // Base0
        },
        Theme {
            name: "tokyo-night".to_string(),
            description: "Tokyo Night - A clean dark theme inspired by Tokyo's skyline".to_string(),
            border: (125, 207, 255),      // Light Blue
            header: (187, 154, 247),      // Purple
            file_name: (125, 207, 255),   // Light Blue
            file_type: (125, 207, 255),   // Light Blue
            dir_name: (187, 154, 247),    // Purple
            dir_type: (187, 154, 247),    // Purple
            file_size: (158, 206, 106),   // Green
            dir_size: (86, 95, 137),      // Comment
            modified: (255, 158, 100),    // Orange
            permissions: (255, 203, 107), // Yellow
            row_number: (169, 177, 214),  // Foreground
        },
    ]
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lsr")
        .join("config.toml")
}

fn load_config() -> Config {
    let config_path = get_config_path();

    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => config,
                Err(_) => Config::default(),
            },
            Err(_) => Config::default(),
        }
    } else {
        Config::default()
    }
}

fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}

fn get_theme_by_name(name: &str) -> Option<Theme> {
    get_themes().into_iter().find(|t| t.name == name)
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Workspace) => {
            if let Err(e) = workspace::print_workspace_snapshot() {
                eprintln!("Error printing workspace: {}", e);
            }
        }
        Some(Commands::Theme { name, set }) => {
            handle_theme_command(name, set);
        }
        None => {
            // Default behavior: show directory listing with saved theme
            let config = load_config();
            let theme =
                get_theme_by_name(&config.default_theme).unwrap_or_else(|| get_themes()[0].clone()); // Fallback to Catppuccin
            show_directory_table(&theme);
        }
    }
}

fn handle_theme_command(theme_name: Option<String>, set_as_default: bool) {
    let themes = get_themes();

    let selected_theme = if let Some(name) = theme_name {
        // Direct theme selection
        themes.iter().find(|t| t.name == name).cloned()
    } else {
        // Interactive theme selection
        let theme_options: Vec<String> = themes
            .iter()
            .map(|t| format!("{} - {}", t.name, t.description))
            .collect();

        match Select::new("Select a theme:", theme_options).prompt() {
            Ok(selection) => {
                let theme_name = selection.split(" - ").next().unwrap();
                themes.iter().find(|t| t.name == theme_name).cloned()
            }
            Err(_) => {
                println!("Theme selection cancelled.");
                return;
            }
        }
    };

    match selected_theme {
        Some(theme) => {
            if set_as_default {
                // Save theme as default
                let config = Config {
                    default_theme: theme.name.clone(),
                };

                match save_config(&config) {
                    Ok(_) => {
                        println!("✓ Set '{}' as default theme", theme.name);
                        println!("Using theme: {} - {}", theme.name, theme.description);
                    }
                    Err(e) => {
                        eprintln!("Error saving config: {}", e);
                        println!(
                            "Using theme: {} - {} (not saved)",
                            theme.name, theme.description
                        );
                    }
                }
            } else {
                println!("Using theme: {} - {}", theme.name, theme.description);
                println!("💡 Use --set to make this your default theme");
            }
            show_directory_table(&theme);
        }
        None => {
            println!("Theme not found. Available themes:");
            for theme in themes {
                println!("  {} - {}", theme.name, theme.description);
            }
        }
    }
}

fn show_directory_table(theme: &Theme) {
    // Step 1: Get current directory
    let current_dir = env::current_dir().expect("Could not get current directory");

    // Step 2: Read contents of current directory
    let entries = fs::read_dir(&current_dir).expect("Could not read current directory");

    // Step 3: Setup pretty table with selected theme
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("#")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: theme.header.0,
                    g: theme.header.1,
                    b: theme.header.2,
                }),
            Cell::new("Name")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: theme.header.0,
                    g: theme.header.1,
                    b: theme.header.2,
                }),
            Cell::new("Type")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: theme.header.0,
                    g: theme.header.1,
                    b: theme.header.2,
                }),
            Cell::new("Size")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: theme.header.0,
                    g: theme.header.1,
                    b: theme.header.2,
                }),
            Cell::new("Modified")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: theme.header.0,
                    g: theme.header.1,
                    b: theme.header.2,
                }),
            Cell::new("Permissions")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: theme.header.0,
                    g: theme.header.1,
                    b: theme.header.2,
                }),
        ]);

    // Step 4: Add each file/folder to the table
    let mut row_number = 1;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let metadata = match path.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };

        let file_type = if path.is_dir() { "Directory" } else { "File" };

        // Create colored cells based on file type using theme colors
        let name_cell = if path.is_dir() {
            Cell::new(&name).fg(Color::Rgb {
                r: theme.dir_name.0,
                g: theme.dir_name.1,
                b: theme.dir_name.2,
            })
        } else {
            Cell::new(&name).fg(Color::Rgb {
                r: theme.file_name.0,
                g: theme.file_name.1,
                b: theme.file_name.2,
            })
        };

        let type_cell = if path.is_dir() {
            Cell::new(file_type).fg(Color::Rgb {
                r: theme.dir_type.0,
                g: theme.dir_type.1,
                b: theme.dir_type.2,
            })
        } else {
            Cell::new(file_type).fg(Color::Rgb {
                r: theme.file_type.0,
                g: theme.file_type.1,
                b: theme.file_type.2,
            })
        };

        // Format size with theme colors
        let size_cell = if path.is_dir() {
            Cell::new(&"-".to_string()).fg(Color::Rgb {
                r: theme.dir_size.0,
                g: theme.dir_size.1,
                b: theme.dir_size.2,
            })
        } else {
            Cell::new(&format_size(metadata.len())).fg(Color::Rgb {
                r: theme.file_size.0,
                g: theme.file_size.1,
                b: theme.file_size.2,
            })
        };

        // Format modified time with theme colors
        let modified_cell = match metadata.modified() {
            Ok(time) => Cell::new(&format_time(time)).fg(Color::Rgb {
                r: theme.modified.0,
                g: theme.modified.1,
                b: theme.modified.2,
            }),
            Err(_) => Cell::new("Unknown").fg(Color::Rgb {
                r: theme.modified.0,
                g: theme.modified.1,
                b: theme.modified.2,
            }),
        };

        // Format permissions with theme color
        let permissions_cell =
            Cell::new(&format_permissions(metadata.permissions().mode())).fg(Color::Rgb {
                r: theme.permissions.0,
                g: theme.permissions.1,
                b: theme.permissions.2,
            });

        table.add_row(vec![
            Cell::new(&row_number.to_string()).fg(Color::Rgb {
                r: theme.row_number.0,
                g: theme.row_number.1,
                b: theme.row_number.2,
            }),
            name_cell,
            type_cell,
            size_cell,
            modified_cell,
            permissions_cell,
        ]);
        row_number += 1;
    }

    // Step 5: Show the table with colored borders
    let table_output = table.to_string();
    let colored_output = colorize_borders(&table_output, theme);
    println!("{}", colored_output);
}

fn format_size(size: u64) -> String {
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
        format!("{} B", size)
    }
}

fn format_time(time: SystemTime) -> String {
    let now = SystemTime::now();
    match now.duration_since(time) {
        Ok(duration) => {
            if duration < Duration::minutes(1).to_std().unwrap() {
                "just now".to_string()
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

fn format_permissions(mode: u32) -> String {
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
    format!("{}{}{}", user, group, other)
}

fn colorize_borders(table_str: &str, theme: &Theme) -> String {
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
                        colored_line.push_str(&format!("{}{}{}", border_color, ch, reset_color));
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
