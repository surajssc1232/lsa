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
use crate::parser::{parse_file, DataValue};
use crate::themes::Theme;
use crate::utils::{colorize_borders, format_permissions, format_size, format_time, calculate_directory_size};

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
        (
            "--open <FILE>",
            "",
            "Parse and display structured data (JSON, YAML, TOML)",
        ),
        (
            "--sort <SORT_BY>",
            "",
            "Sort by: name, size, modified, type",
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
        "{}lsa - A colorful directory listing tool{}",
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
        "  {}lsa{}                              # Show current directory listing",
        example_color, reset_color
    );
    println!(
        "  {}lsa src{}                          # Show specific directory listing",
        example_color, reset_color
    );
    println!(
        "  {}lsa --tree{}                       # Show tree view",
        example_color, reset_color
    );
    println!(
        "  {}lsa --path{}                       # Show PATH environment directories",
        example_color, reset_color
    );
    println!(
        "  {}lsa --workspace{}                  # Copy workspace to clipboard",
        example_color, reset_color
    );
    println!(
        "  {}lsa --workspace-file src/main.rs{} # Copy specific file to clipboard",
        example_color, reset_color
    );
    #[cfg(unix)]
    println!(
        "  {}lsa --workspace-folder src/{} # Copy specific folder to clipboard",
        example_color, reset_color
    );
    #[cfg(windows)]
    println!(
        "  {}lsa --workspace-folder src\\\\{} # Copy specific folder to clipboard",
        example_color, reset_color
    );
    println!(
        "  {}lsa --workspace --source-only{}    # Copy only source files",
        example_color, reset_color
    );
    println!(
        "  {}lsa --workspace --max-size 32{}    # Limit output to 32KB",
        example_color, reset_color
    );
    println!(
        "  {}lsa --theme{}                      # Set default theme interactively",
        example_color, reset_color
    );
    println!(
        "  {}lsa --cpu{}                        # Show CPU information",
        example_color, reset_color
    );
    println!(
        "  {}lsa --open config.json{}           # Display JSON file in tabular format",
        example_color, reset_color
    );
    println!(
        "  {}lsa --open settings.yaml{}         # Display YAML file in tabular format",
        example_color, reset_color
    );
    println!(
        "  {}lsa --open Cargo.toml{}            # Display TOML file in tabular format",
        example_color, reset_color
    );
}

pub fn show_directory_table(theme: &Theme, directory_path: Option<&str>, sort_by: Option<&crate::SortBy>) {
    let target_dir = if let Some(path) = directory_path {
        std::path::PathBuf::from(path)
    } else {
        env::current_dir().expect("Could not get current directory")
    };

    let entries = fs::read_dir(&target_dir).expect("Could not read directory");

    // Collect all entries with their metadata
    let mut entries_with_meta: Vec<_> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let metadata = path.metadata().ok()?;
            let name = path.file_name()?.to_string_lossy().to_string();
            let file_type = if path.is_dir() { "Directory" } else { "File" };
            let size = if path.is_dir() {
                calculate_directory_size(&path)
            } else {
                metadata.len()
            };
            let modified = metadata.modified().ok()?;
            Some((path, name, file_type.to_string(), size, modified, metadata))
        })
        .collect();

    // Sort entries based on sort_by parameter
    if let Some(sort_by) = sort_by {
        entries_with_meta.sort_by(|a, b| {
            match sort_by {
                crate::SortBy::Name => a.1.cmp(&b.1),
                crate::SortBy::Size => a.3.cmp(&b.3),
                crate::SortBy::Modified => a.4.cmp(&b.4),
                crate::SortBy::Type => {
                    // Directories first, then files
                    let a_is_dir = a.0.is_dir();
                    let b_is_dir = b.0.is_dir();
                    match (a_is_dir, b_is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.1.cmp(&b.1), // Same type, sort by name
                    }
                }
            }
        });
    }

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
    for (path, name, file_type, size, modified, metadata) in entries_with_meta {
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
            Cell::new(&file_type).fg(Color::Rgb {
                r: theme.dir_type.0,
                g: theme.dir_type.1,
                b: theme.dir_type.2,
            })
        } else {
            Cell::new(&file_type).fg(Color::Rgb {
                r: theme.file_type.0,
                g: theme.file_type.1,
                b: theme.file_type.2,
            })
        };

        let size_cell = Cell::new(format_size(size)).fg(if path.is_dir() {
            Color::Rgb {
                r: theme.dir_size.0,
                g: theme.dir_size.1,
                b: theme.dir_size.2,
            }
        } else {
            Color::Rgb {
                r: theme.file_size.0,
                g: theme.file_size.1,
                b: theme.file_size.2,
            }
        });

        let modified_cell = Cell::new(format_time(modified)).fg(Color::Rgb {
            r: theme.modified.0,
            g: theme.modified.1,
            b: theme.modified.2,
        });

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

pub fn show_structured_data(theme: &Theme, file_path: &str) {
    match parse_file(file_path) {
        Ok(parsed_data) => {
            let title_color = format!(
                "\x1b[38;2;{};{};{}m",
                theme.header.0, theme.header.1, theme.header.2
            );
            let reset_color = "\x1b[0m";

            println!(
                "{}Structured Data ({}) - {}{}", 
                title_color, parsed_data.format, file_path, reset_color
            );
            println!();

            render_flattened_data(&parsed_data.data, theme);
        }
        Err(e) => {
            eprintln!("Error parsing file '{}': {}", file_path, e);
        }
    }
}

fn render_flattened_data(data: &DataValue, theme: &Theme) {
    render_main_table_with_nested(data, theme);
}

fn render_main_table_with_nested(data: &DataValue, theme: &Theme) {
    match data {
        DataValue::Object(obj) => {
            let mut table = Table::new();
            table
                .load_preset(UTF8_BORDERS_ONLY)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Key")
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

            for (i, (key, value)) in obj.iter().enumerate() {
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

                let value_cell_content = if value.is_simple_value() {
                    value.to_display_string()
                } else {
                    // Create a nested table as a string and embed it in the cell
                    create_nested_table_string(value, theme)
                };

                let value_color = if value.is_simple_value() {
                    if i % 2 == 0 {
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
                    }
                } else {
                    // For nested tables, use a neutral color
                    Color::Rgb {
                        r: theme.permissions.0,
                        g: theme.permissions.1,
                        b: theme.permissions.2,
                    }
                };

                table.add_row(vec![
                    Cell::new(key).fg(key_color),
                    Cell::new(value_cell_content).fg(value_color),
                ]);
            }

            let table_output = table.to_string();
            let colored_output = colorize_borders(&table_output, theme);
            println!("{}", colored_output);
        }
        DataValue::Array(arr) => {
            let mut table = Table::new();
            table
                .load_preset(UTF8_BORDERS_ONLY)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Index")
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

            for (i, value) in arr.iter().enumerate() {
                let index_color = if i % 2 == 0 {
                    Color::Rgb {
                        r: theme.row_number.0,
                        g: theme.row_number.1,
                        b: theme.row_number.2,
                    }
                } else {
                    Color::Rgb {
                        r: theme.file_type.0,
                        g: theme.file_type.1,
                        b: theme.file_type.2,
                    }
                };

                let value_cell_content = if value.is_simple_value() {
                    value.to_display_string()
                } else {
                    create_nested_table_string(value, theme)
                };

                let value_color = if value.is_simple_value() {
                    if i % 2 == 0 {
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
                    }
                } else {
                    Color::Rgb {
                        r: theme.permissions.0,
                        g: theme.permissions.1,
                        b: theme.permissions.2,
                    }
                };

                table.add_row(vec![
                    Cell::new(i.to_string()).fg(index_color),
                    Cell::new(value_cell_content).fg(value_color),
                ]);
            }

            let table_output = table.to_string();
            let colored_output = colorize_borders(&table_output, theme);
            println!("{}", colored_output);
        }
        _ => {
            // For simple values, just display them in a single-column table
            let mut table = Table::new();
            table
                .load_preset(UTF8_BORDERS_ONLY)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Value")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Rgb {
                            r: theme.header.0,
                            g: theme.header.1,
                            b: theme.header.2,
                        }),
                ]);

            table.add_row(vec![
                Cell::new(data.to_display_string()).fg(Color::Rgb {
                    r: theme.file_name.0,
                    g: theme.file_name.1,
                    b: theme.file_name.2,
                }),
            ]);

            let table_output = table.to_string();
            let colored_output = colorize_borders(&table_output, theme);
            println!("{}", colored_output);
        }
    }
}

fn create_nested_table_string(data: &DataValue, _theme: &Theme) -> String {
    match data {
        DataValue::Object(obj) => {
            let mut nested_table = Table::new();
            nested_table
                .load_preset(UTF8_BORDERS_ONLY)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic);

            // Create a compact nested table with full recursion
            for (key, value) in obj.iter() {
                let display_value = if value.is_simple_value() {
                    value.to_display_string()
                } else {
                    // Recursively create nested tables instead of summaries
                    create_nested_table_string(value, _theme)
                };
                nested_table.add_row(vec![
                    Cell::new(key),
                    Cell::new(display_value),
                ]);
            }
            
            // Remove ANSI color codes from the table string for embedding
            strip_ansi_codes(&nested_table.to_string())
        }
        DataValue::Array(arr) => {
            let mut nested_table = Table::new();
            nested_table
                .load_preset(UTF8_BORDERS_ONLY)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic);

            for (i, value) in arr.iter().enumerate() {
                let display_value = if value.is_simple_value() {
                    value.to_display_string()
                } else {
                    // Recursively create nested tables instead of summaries
                    create_nested_table_string(value, _theme)
                };
                nested_table.add_row(vec![
                    Cell::new(i.to_string()),
                    Cell::new(display_value),
                ]);
            }
            
            strip_ansi_codes(&nested_table.to_string())
        }
        _ => data.to_display_string(),
    }
}

fn strip_ansi_codes(input: &str) -> String {
    // Simple ANSI escape sequence removal for table embedding
    let mut result = String::new();
    let mut chars = input.chars();
    
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Skip ANSI escape sequences
            while let Some(next_ch) = chars.next() {
                if next_ch.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}




