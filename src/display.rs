use comfy_table::{
    Attribute, Cell, Color, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_BORDERS_ONLY,
};
use std::env;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use crate::icons::get_file_icon;
use crate::themes::Theme;
use crate::utils::{colorize_borders, format_permissions, format_size, format_time};

pub fn show_cpu_info(theme: &Theme) {
    let output = match Command::new("lscpu").output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                eprintln!("Error running lscpu command");
                return;
            }
        }
        Err(e) => {
            eprintln!("Failed to execute lscpu: {}", e);
            return;
        }
    };

    let mut cpu_info = Vec::new();
    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            cpu_info.push((key.trim().to_string(), value.trim().to_string()));
        }
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("CPU Information")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: theme.header.0,
                    g: theme.header.1,
                    b: theme.header.2,
                }),
            Cell::new("Value")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: theme.header.0,
                    g: theme.header.1,
                    b: theme.header.2,
                }),
        ]);

    for (i, (key, value)) in cpu_info.iter().enumerate() {
        let key_color = if i % 2 == 0 {
            Color::Rgb {
                r: theme.file_name.0,
                g: theme.file_name.1,
                b: theme.file_name.2,
            }
        } else {
            Color::Rgb {
                r: theme.dir_name.0,
                g: theme.dir_name.1,
                b: theme.dir_name.2,
            }
        };

        let value_color = if i % 2 == 0 {
            Color::Rgb {
                r: theme.file_size.0,
                g: theme.file_size.1,
                b: theme.file_size.2,
            }
        } else {
            Color::Rgb {
                r: theme.modified.0,
                g: theme.modified.1,
                b: theme.modified.2,
            }
        };

        table.add_row(vec![
            Cell::new(key).fg(key_color),
            Cell::new(value).fg(value_color),
        ]);
    }

    let table_output = table.to_string();
    let colored_output = colorize_borders(&table_output, theme);
    println!("{}", colored_output);
}

pub fn show_help(theme: &Theme) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Option")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: theme.header.0,
                    g: theme.header.1,
                    b: theme.header.2,
                }),
            Cell::new("Short")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: theme.header.0,
                    g: theme.header.1,
                    b: theme.header.2,
                }),
            Cell::new("Description")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: theme.header.0,
                    g: theme.header.1,
                    b: theme.header.2,
                }),
        ]);

    let options = vec![
        ("--help", "-h", "Show this help information"),
        ("--cpu", "", "Show CPU information"),
        (
            "--workspace",
            "",
            "Show workspace snapshot (copy to clipboard)",
        ),
        (
            "--workspace-file <PATH>",
            "",
            "Copy specific file to clipboard",
        ),
        (
            "--workspace-folder <PATH>",
            "",
            "Copy specific folder to clipboard",
        ),
        (
            "--source-only",
            "",
            "Only include source files in workspace",
        ),
        (
            "--max-size <KB>",
            "",
            "Maximum size limit for workspace output",
        ),
        ("--tree", "-t", "Show directory tree view"),
        ("--depth <NUM>", "-d", "Maximum depth for tree view"),
        (
            "--all",
            "-a",
            "Show all files including hidden ones (for tree)",
        ),
        ("--path", "", "Show PATH environment variable directories"),
        (
            "--theme [NAME]",
            "",
            "Set theme as default (interactive if no name)",
        ),
    ];

    for (i, (long, short, desc)) in options.iter().enumerate() {
        let option_color = if i % 2 == 0 {
            Color::Rgb {
                r: theme.file_name.0,
                g: theme.file_name.1,
                b: theme.file_name.2,
            }
        } else {
            Color::Rgb {
                r: theme.dir_name.0,
                g: theme.dir_name.1,
                b: theme.dir_name.2,
            }
        };

        let short_color = if i % 2 == 0 {
            Color::Rgb {
                r: theme.file_type.0,
                g: theme.file_type.1,
                b: theme.file_type.2,
            }
        } else {
            Color::Rgb {
                r: theme.dir_type.0,
                g: theme.dir_type.1,
                b: theme.dir_type.2,
            }
        };

        let desc_color = if i % 2 == 0 {
            Color::Rgb {
                r: theme.file_size.0,
                g: theme.file_size.1,
                b: theme.file_size.2,
            }
        } else {
            Color::Rgb {
                r: theme.modified.0,
                g: theme.modified.1,
                b: theme.modified.2,
            }
        };

        table.add_row(vec![
            Cell::new(*long).fg(option_color),
            Cell::new(*short).fg(short_color),
            Cell::new(*desc).fg(desc_color),
        ]);
    }

    let title_color = format!(
        "\x1b[38;2;{};{};{}m",
        theme.header.0, theme.header.1, theme.header.2
    );
    let reset_color = "\x1b[0m";

    println!(
        "{}lsr - A colorful directory listing tool{}",
        title_color, reset_color
    );
    println!();

    let table_output = table.to_string();
    let colored_output = colorize_borders(&table_output, theme);
    println!("{}", colored_output);

    println!();
    println!("{}Examples:{}", title_color, reset_color);
    let example_color = format!(
        "\x1b[38;2;{};{};{}m",
        theme.file_name.0, theme.file_name.1, theme.file_name.2
    );
    println!(
        "  {}lsr{}                              # Show current directory listing",
        example_color, reset_color
    );
    println!(
        "  {}lsr src{}                          # Show specific directory listing",
        example_color, reset_color
    );
    println!(
        "  {}lsr --tree{}                       # Show tree view",
        example_color, reset_color
    );
    println!(
        "  {}lsr --path{}                       # Show PATH environment directories",
        example_color, reset_color
    );
    println!(
        "  {}lsr --workspace{}                  # Copy workspace to clipboard",
        example_color, reset_color
    );
    println!(
        "  {}lsr --workspace-file src/main.rs{} # Copy specific file to clipboard",
        example_color, reset_color
    );
    #[cfg(unix)]
    println!(
        "  {}lsr --workspace-folder src/{} # Copy specific folder to clipboard",
        example_color, reset_color
    );
    #[cfg(windows)]
    println!(
        "  {}lsr --workspace-folder src\\\\{} # Copy specific folder to clipboard",
        example_color, reset_color
    );
    println!(
        "  {}lsr --workspace --source-only{}    # Copy only source files",
        example_color, reset_color
    );
    println!(
        "  {}lsr --workspace --max-size 32{}    # Limit output to 32KB",
        example_color, reset_color
    );
    println!(
        "  {}lsr --theme{}                      # Set default theme interactively",
        example_color, reset_color
    );
    println!(
        "  {}lsr --cpu{}                        # Show CPU information",
        example_color, reset_color
    );
}

pub fn show_directory_table(theme: &Theme, directory_path: Option<&str>) {
    let target_dir = if let Some(path) = directory_path {
        std::path::PathBuf::from(path)
    } else {
        env::current_dir().expect("Could not get current directory")
    };

    let entries = fs::read_dir(&target_dir).expect("Could not read directory");

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

        let name_with_icon = format!("{} {}", get_file_icon(&path), name);
        let name_cell = if path.is_dir() {
            Cell::new(&name_with_icon).fg(Color::Rgb {
                r: theme.dir_name.0,
                g: theme.dir_name.1,
                b: theme.dir_name.2,
            })
        } else {
            Cell::new(&name_with_icon).fg(Color::Rgb {
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

        let size_cell = if path.is_dir() {
            Cell::new("-".to_string()).fg(Color::Rgb {
                r: theme.dir_size.0,
                g: theme.dir_size.1,
                b: theme.dir_size.2,
            })
        } else {
            Cell::new(format_size(metadata.len())).fg(Color::Rgb {
                r: theme.file_size.0,
                g: theme.file_size.1,
                b: theme.file_size.2,
            })
        };

        let modified_cell = match metadata.modified() {
            Ok(time) => Cell::new(format_time(time)).fg(Color::Rgb {
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

        let permissions_cell = {
            #[cfg(unix)]
            let perm_text = format_permissions(metadata.permissions().mode());
            #[cfg(windows)]
            let perm_text = format_permissions(&metadata);

            Cell::new(perm_text).fg(Color::Rgb {
                r: theme.permissions.0,
                g: theme.permissions.1,
                b: theme.permissions.2,
            })
        };

        table.add_row(vec![
            Cell::new(row_number.to_string()).fg(Color::Rgb {
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

    let table_output = table.to_string();
    let colored_output = colorize_borders(&table_output, theme);
    println!("{colored_output}");
}

pub fn show_tree(theme: &Theme, max_depth: Option<usize>, show_all: bool) {
    let current_dir = env::current_dir().expect("Could not get current directory");

    let root_name = current_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    let colored_root = format!(
        "\x1b[38;2;{};{};{}m{} {}\x1b[0m",
        theme.dir_name.0,
        theme.dir_name.1,
        theme.dir_name.2,
        get_file_icon(&current_dir),
        root_name
    );
    println!("{}", colored_root);

    display_tree_recursive(&current_dir, "", true, 0, max_depth, show_all, theme);
}

pub fn show_path_table(theme: &Theme) {
    let path_env = match env::var("PATH") {
        Ok(path) => path,
        Err(_) => {
            println!("Error: Could not read PATH environment variable");
            return;
        }
    };

    let path_dirs: Vec<&str> = path_env.split(':').collect();

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
            Cell::new("Directory")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: theme.header.0,
                    g: theme.header.1,
                    b: theme.header.2,
                }),
            Cell::new("Status")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: theme.header.0,
                    g: theme.header.1,
                    b: theme.header.2,
                }),
        ]);

    for (index, path_dir) in path_dirs.iter().enumerate() {
        let path = std::path::Path::new(path_dir);
        let exists = path.exists();
        let is_dir = path.is_dir();
        let is_symlink = path.is_symlink();

        let status = if !exists {
            if is_symlink {
                "Broken symlink"
            } else {
                "Missing"
            }
        } else if is_symlink && is_dir {
            "Symlink to directory"
        } else if is_symlink {
            "Symlink to file"
        } else if !is_dir {
            "Not a directory"
        } else {
            "OK"
        };

        let dir_cell = if exists && is_dir {
            let icon = if is_symlink {
                "↪" 
            } else {
                get_file_icon(path)
            };
            Cell::new(format!("{} {}", icon, path_dir)).fg(Color::Rgb {
                r: theme.dir_name.0,
                g: theme.dir_name.1,
                b: theme.dir_name.2,
            })
        } else if is_symlink {
            Cell::new(format!("↪ {}", path_dir)).fg(Color::Rgb {
                r: theme.permissions.0,
                g: theme.permissions.1,
                b: theme.permissions.2,
            })
        } else {
            Cell::new(format!("✗ {}", path_dir)).fg(Color::Rgb {
                r: theme.file_name.0,
                g: theme.file_name.1,
                b: theme.file_name.2,
            })
        };

        let status_cell = match status {
            "OK" => Cell::new(status).fg(Color::Rgb {
                r: theme.file_size.0,
                g: theme.file_size.1,
                b: theme.file_size.2,
            }),
            _ => Cell::new(status).fg(Color::Rgb {
                r: theme.modified.0,
                g: theme.modified.1,
                b: theme.modified.2,
            }),
        };

        table.add_row(vec![
            Cell::new((index + 1).to_string()).fg(Color::Rgb {
                r: theme.row_number.0,
                g: theme.row_number.1,
                b: theme.row_number.2,
            }),
            dir_cell,
            status_cell,
        ]);
    }

    let table_output = table.to_string();
    let colored_output = colorize_borders(&table_output, theme);
    println!("{colored_output}");
}

fn display_tree_recursive(
    dir: &std::path::Path,
    prefix: &str,
    _is_last: bool,
    current_depth: usize,
    max_depth: Option<usize>,
    show_all: bool,
    theme: &Theme,
) {
    if let Some(max) = max_depth {
        if current_depth >= max {
            return;
        }
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    let mut items: Vec<_> = entries.filter_map(|entry| entry.ok()).collect();
    items.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();

        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    if !show_all {
        items.retain(|item| !item.file_name().to_string_lossy().starts_with('.'));
    }

    let total_items = items.len();

    for (index, entry) in items.iter().enumerate() {
        let path = entry.path();
        let is_last_item = index == total_items - 1;

        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        let current_prefix = if is_last_item {
            "└── "
        } else {
            "├── "
        };

        let icon = get_file_icon(&path);
        let (name_color, type_indicator) = if path.is_dir() {
            (
                format!(
                    "\x1b[38;2;{};{};{}m",
                    theme.dir_name.0, theme.dir_name.1, theme.dir_name.2
                ),
                "/",
            )
        } else {
            (
                format!(
                    "\x1b[38;2;{};{};{}m",
                    theme.file_name.0, theme.file_name.1, theme.file_name.2
                ),
                "",
            )
        };

        let tree_color = format!(
            "\x1b[38;2;{};{};{}m",
            theme.border.0, theme.border.1, theme.border.2
        );
        let reset_color = "\x1b[0m";

        println!(
            "{}{}{}{}{}{}{}{}{}",
            prefix,
            tree_color,
            current_prefix,
            reset_color,
            name_color,
            icon,
            " ",
            file_name,
            if path.is_dir() {
                format!(
                    "{}{}{}",
                    format!(
                        "\x1b[38;2;{};{};{}m",
                        theme.dir_type.0, theme.dir_type.1, theme.dir_type.2
                    ),
                    type_indicator,
                    reset_color
                )
            } else {
                reset_color.to_string()
            }
        );

        if path.is_dir() {
            let colored_next_prefix = if is_last_item {
                "    "
            } else {
                &format!("{}│{}{:3}", tree_color, reset_color, "")
            };
            let new_prefix = format!("{}{}", prefix, colored_next_prefix);
            display_tree_recursive(
                &path,
                &new_prefix,
                is_last_item,
                current_depth + 1,
                max_depth,
                show_all,
                theme,
            );
        }
    }
}

