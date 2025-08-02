mod workspace;
use chrono::Duration;
use clap::Parser;
use comfy_table::{
    Attribute, Cell, Color, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_BORDERS_ONLY,
};
use inquire::Select;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

#[derive(Parser)]
#[command(name = "lsr")]
#[command(about = "A colorful directory listing tool with multiple themes")]
#[command(disable_help_flag = true)]
struct Cli {

    #[arg(short, long)]
    help: bool,

    #[arg(long)]
    cpu: bool,

    #[arg(long)]
    workspace: bool,

    #[arg(long, value_name = "FILE_PATH")]
    workspace_file: Option<String>,

    #[arg(long, value_name = "FOLDER_PATH")]
    workspace_folder: Option<String>,

    #[arg(long, requires = "workspace")]
    source_only: bool,

    #[arg(long, requires = "workspace", value_name = "KB")]
    max_size: Option<usize>,

    #[arg(short = 't', long)]
    tree: bool,

    #[arg(short, long, requires = "tree")]
    depth: Option<usize>,

    #[arg(short = 'a', long, requires = "tree")]
    all: bool,

    #[arg(long, value_name = "THEME")]
    theme: Option<Option<String>>,
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
            border: (183, 189, 248),
            header: (203, 166, 247),
            file_name: (116, 199, 236),
            file_type: (116, 199, 236),
            dir_name: (137, 180, 250),
            dir_type: (137, 180, 250),
            file_size: (166, 227, 161),
            dir_size: (108, 112, 134),
            modified: (249, 226, 175),
            permissions: (148, 226, 213),
            row_number: (205, 214, 244),
        },
        Theme {
            name: "nord".to_string(),
            description: "Nord - Arctic, north-bluish color palette".to_string(),
            border: (136, 192, 208),
            header: (143, 188, 187),
            file_name: (129, 161, 193),
            file_type: (129, 161, 193),
            dir_name: (94, 129, 172),
            dir_type: (94, 129, 172),
            file_size: (163, 190, 140),
            dir_size: (76, 86, 106),
            modified: (235, 203, 139),
            permissions: (180, 142, 173),
            row_number: (216, 222, 233),
        },
        Theme {
            name: "dracula".to_string(),
            description: "Dracula - Dark theme with vibrant colors".to_string(),
            border: (189, 147, 249),
            header: (255, 121, 198),
            file_name: (139, 233, 253),
            file_type: (139, 233, 253),
            dir_name: (189, 147, 249),
            dir_type: (189, 147, 249),
            file_size: (80, 250, 123),
            dir_size: (98, 114, 164),
            modified: (255, 184, 108),
            permissions: (241, 250, 140),
            row_number: (248, 248, 242),
        },
        Theme {
            name: "monokai".to_string(),
            description: "Monokai - Classic vibrant dark theme".to_string(),
            border: (174, 129, 255),
            header: (255, 97, 136),
            file_name: (120, 220, 232),
            file_type: (120, 220, 232),
            dir_name: (174, 129, 255),
            dir_type: (174, 129, 255),
            file_size: (158, 206, 106),
            dir_size: (117, 113, 94),
            modified: (255, 216, 102),
            permissions: (255, 97, 136),
            row_number: (248, 248, 242),
        },
        Theme {
            name: "gruvbox".to_string(),
            description: "Gruvbox - Retro groove warm color scheme".to_string(),
            border: (214, 153, 108),
            header: (251, 73, 52),
            file_name: (131, 165, 152),
            file_type: (131, 165, 152),
            dir_name: (69, 133, 136),
            dir_type: (69, 133, 136),
            file_size: (152, 151, 26),
            dir_size: (146, 131, 116),
            modified: (215, 153, 33),
            permissions: (177, 98, 134),
            row_number: (235, 219, 178),
        },
        Theme {
            name: "solarized".to_string(),
            description: "Solarized - Precision colors for machines and people".to_string(),
            border: (42, 161, 152),
            header: (211, 54, 130),
            file_name: (38, 139, 210),
            file_type: (38, 139, 210),
            dir_name: (42, 161, 152),
            dir_type: (42, 161, 152),
            file_size: (133, 153, 0),
            dir_size: (101, 123, 131),
            modified: (181, 137, 0),
            permissions: (108, 113, 196),
            row_number: (131, 148, 150),
        },
        Theme {
            name: "tokyo-night".to_string(),
            description: "Tokyo Night - A clean dark theme inspired by Tokyo's skyline".to_string(),
            border: (125, 207, 255),
            header: (187, 154, 247),
            file_name: (125, 207, 255),
            file_type: (125, 207, 255),
            dir_name: (187, 154, 247),
            dir_type: (187, 154, 247),
            file_size: (158, 206, 106),
            dir_size: (86, 95, 137),
            modified: (255, 158, 100),
            permissions: (255, 203, 107),
            row_number: (169, 177, 214),
        },
        Theme {
            name: "onedark".to_string(),
            description: "One Dark - Atom's iconic One Dark theme".to_string(),
            border: (97, 175, 239),
            header: (198, 120, 221),
            file_name: (97, 175, 239),
            file_type: (97, 175, 239),
            dir_name: (198, 120, 221),
            dir_type: (198, 120, 221),
            file_size: (152, 195, 121),
            dir_size: (92, 99, 112),
            modified: (229, 192, 123),
            permissions: (86, 182, 194),
            row_number: (171, 178, 191),
        },
        Theme {
            name: "material".to_string(),
            description: "Material - Google's Material Design color palette".to_string(),
            border: (100, 181, 246),
            header: (156, 39, 176),
            file_name: (33, 150, 243),
            file_type: (33, 150, 243),
            dir_name: (103, 58, 183),
            dir_type: (103, 58, 183),
            file_size: (76, 175, 80),
            dir_size: (117, 117, 117),
            modified: (255, 152, 0),
            permissions: (0, 188, 212),
            row_number: (97, 97, 97),
        },
        Theme {
            name: "oceanic-next".to_string(),
            description: "Oceanic Next - Sophisticated oceanic color scheme".to_string(),
            border: (101, 115, 126),
            header: (192, 197, 206),
            file_name: (102, 153, 204),
            file_type: (102, 153, 204),
            dir_name: (193, 132, 1),
            dir_type: (193, 132, 1),
            file_size: (153, 173, 106),
            dir_size: (79, 91, 102),
            modified: (250, 208, 122),
            permissions: (95, 179, 151),
            row_number: (160, 168, 180),
        },
        Theme {
            name: "ayu-dark".to_string(),
            description: "Ayu Dark - Modern minimalist dark theme".to_string(),
            border: (83, 89, 97),
            header: (255, 173, 0),
            file_name: (89, 181, 230),
            file_type: (89, 181, 230),
            dir_name: (209, 154, 102),
            dir_type: (209, 154, 102),
            file_size: (172, 203, 115),
            dir_size: (92, 103, 115),
            modified: (242, 151, 24),
            permissions: (128, 203, 196),
            row_number: (151, 165, 180),
        },
        Theme {
            name: "synthwave".to_string(),
            description: "Synthwave - Retro 80s cyberpunk aesthetics".to_string(),
            border: (241, 250, 140),
            header: (255, 20, 147),
            file_name: (0, 255, 255),
            file_type: (0, 255, 255),
            dir_name: (255, 20, 147),
            dir_type: (255, 20, 147),
            file_size: (50, 255, 50),
            dir_size: (139, 69, 19),
            modified: (255, 165, 0),
            permissions: (255, 0, 255),
            row_number: (255, 255, 255),
        },
        Theme {
            name: "github-dark".to_string(),
            description: "GitHub Dark - GitHub's official dark theme".to_string(),
            border: (48, 54, 61),
            header: (255, 255, 255),
            file_name: (121, 192, 255),
            file_type: (121, 192, 255),
            dir_name: (255, 184, 108),
            dir_type: (255, 184, 108),
            file_size: (63, 185, 80),
            dir_size: (139, 148, 158),
            modified: (255, 235, 59),
            permissions: (164, 196, 255),
            row_number: (201, 209, 217),
        },
        Theme {
            name: "cobalt2".to_string(),
            description: "Cobalt2 - Electric blue theme for night owls".to_string(),
            border: (0, 122, 204),
            header: (255, 204, 102),
            file_name: (158, 206, 106),
            file_type: (158, 206, 106),
            dir_name: (255, 204, 102),
            dir_type: (255, 204, 102),
            file_size: (102, 217, 239),
            dir_size: (128, 128, 128),
            modified: (255, 198, 109),
            permissions: (255, 157, 77),
            row_number: (193, 193, 193),
        },
        Theme {
            name: "palenight".to_string(),
            description: "Palenight - A darker Material Theme variant".to_string(),
            border: (130, 170, 255),
            header: (199, 146, 234),
            file_name: (130, 170, 255),
            file_type: (130, 170, 255),
            dir_name: (255, 203, 107),
            dir_type: (255, 203, 107),
            file_size: (195, 232, 141),
            dir_size: (103, 110, 149),
            modified: (255, 158, 100),
            permissions: (137, 221, 255),
            row_number: (171, 178, 191),
        },
        Theme {
            name: "night-owl".to_string(),
            description: "Night Owl - A theme for night owls by Sarah Drasner".to_string(),
            border: (130, 170, 255),
            header: (199, 146, 234),
            file_name: (130, 170, 255),
            file_type: (130, 170, 255),
            dir_name: (195, 232, 141),
            dir_type: (195, 232, 141),
            file_size: (255, 203, 107),
            dir_size: (103, 110, 149),
            modified: (255, 158, 100),
            permissions: (137, 221, 255),
            row_number: (214, 222, 235),
        },
        Theme {
            name: "horizon".to_string(),
            description: "Horizon - A beautifully warm dual theme".to_string(),
            border: (250, 183, 149),
            header: (236, 196, 141),
            file_name: (156, 207, 216),
            file_type: (156, 207, 216),
            dir_name: (250, 183, 149),
            dir_type: (250, 183, 149),
            file_size: (158, 180, 158),
            dir_size: (87, 82, 74),
            modified: (236, 196, 141),
            permissions: (229, 152, 155),
            row_number: (203, 204, 198),
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

fn get_file_icon(path: &std::path::Path) -> &'static str {
    if path.is_dir() {
        "\u{f07b}"
    } else if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
        match extension.to_lowercase().as_str() {

            "rs" => "\u{e7a8}",
            "py" => "\u{e73c}",
            "js" | "mjs" => "\u{e74e}",
            "ts" => "\u{e628}",
            "jsx" => "\u{e7ba}",
            "tsx" => "\u{e7ba}",
            "go" => "\u{e626}",
            "java" => "\u{e204}",
            "c" => "\u{e61e}",
            "h" => "\u{e61e}",
            "cpp" | "cc" | "cxx" => "\u{e61d}",
            "hpp" => "\u{e61d}",
            "cs" => "\u{f81a}",
            "php" => "\u{e73d}",
            "rb" => "\u{e21e}",
            "swift" => "\u{e755}",
            "kt" => "\u{e634}",
            "dart" => "\u{e798}",
            "lua" => "\u{e620}",
            "r" => "\u{f25d}",
            "scala" => "\u{e737}",
            "clj" | "cljs" => "\u{e768}",
            "hs" => "\u{e777}",
            "ml" | "mli" => "\u{e7a7}",
            "elm" => "\u{e62c}",
            "ex" | "exs" => "\u{e62d}",
            "erl" => "\u{e7b1}",
            "vim" => "\u{e62b}",
            "sh" | "bash" | "zsh" | "fish" => "\u{f489}",
            "ps1" => "\u{f489}",

            "html" | "htm" => "\u{e60e}",
            "css" => "\u{e614}",
            "scss" | "sass" => "\u{e603}",
            "less" => "\u{e758}",
            "vue" => "\u{fd42}",
            "svelte" => "\u{e697}",
            "angular" => "\u{e753}",

            "json" => "\u{e60b}",
            "toml" => "\u{e615}",
            "yaml" | "yml" => "\u{f481}",
            "xml" => "\u{e619}",
            "ini" => "\u{f17a}",
            "conf" | "config" => "\u{e615}",
            "env" => "\u{f462}",
            "dockerfile" => "\u{f308}",
            "makefile" => "\u{f728}",

            "md" | "markdown" => "\u{e609}",
            "txt" => "\u{f15c}",
            "rst" => "\u{f15c}",
            "tex" => "\u{e600}",
            "rtf" => "\u{f15c}",
            "pdf" => "\u{f1c1}",

            "png" | "jpg" | "jpeg" => "\u{f1c5}",
            "gif" => "\u{f1c5}",
            "svg" => "\u{fc1f}",
            "bmp" | "tiff" | "tif" => "\u{f1c5}",
            "webp" => "\u{f1c5}",
            "ico" => "\u{f1c5}",
            "psd" => "\u{e7b8}",
            "ai" => "\u{e7b4}",

            "zip" | "7z" | "rar" => "\u{f410}",
            "tar" | "gz" | "gzip" | "bz2" | "xz" => "\u{f410}",

            "exe" | "msi" => "\u{f17a}",
            "app" | "dmg" => "\u{f179}",
            "deb" | "rpm" | "pkg" => "\u{f187}",
            "appimage" => "\u{f179}",

            "doc" | "docx" => "\u{f1c2}",
            "xls" | "xlsx" => "\u{f1c3}",
            "ppt" | "pptx" => "\u{f1c4}",
            "odt" | "ods" | "odp" => "\u{f1c2}",

            "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" | "wma" => "\u{f001}",

            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "webm" | "flv" | "m4v" => "\u{f03d}",

            "ttf" | "otf" | "woff" | "woff2" | "eot" => "\u{f031}",

            "db" | "sqlite" | "sqlite3" => "\u{f1c0}",
            "sql" => "\u{f1c0}",

            "lock" => "\u{f023}",
            "tmp" | "temp" => "\u{f2ed}",
            "bak" | "backup" => "\u{f56e}",
            "log" => "\u{f18e}",

            _ => "\u{f15b}",
        }
    } else {

        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match filename.as_str() {
            "dockerfile" => "\u{f308}",
            "makefile" => "\u{f728}",
            "readme" => "\u{f7fb}",
            "license" => "\u{f718}",
            "changelog" => "\u{f7d9}",
            "cargo.toml" => "\u{e7a8}",
            "package.json" => "\u{e718}",
            ".gitignore" => "\u{f1d3}",
            ".gitmodules" => "\u{f1d3}",
            ".env" => "\u{f462}",
            _ => "\u{f15b}",
        }
    }
}

fn get_theme_by_name(name: &str) -> Option<Theme> {
    get_themes().into_iter().find(|t| t.name == name)
}

fn show_cpu_info(theme: &Theme) {

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

fn show_help(theme: &Theme) {
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
        "  {}lsr{}                              # Show directory listing",
        example_color, reset_color
    );
    println!(
        "  {}lsr --tree{}                       # Show tree view",
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
        "  {}lsr --workspace-folder src/{}      # Copy specific folder to clipboard",
        example_color, reset_color
    );
    #[cfg(windows)]
    println!(
        "  {}lsr --workspace-folder src\\{}     # Copy specific folder to clipboard",
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

fn main() {
    let cli = Cli::parse();


    if cli.help {
        let config = load_config();
        let theme =
            get_theme_by_name(&config.default_theme).unwrap_or_else(|| get_themes()[0].clone());
        show_help(&theme);
        return;
    }


    if let Some(theme_option) = &cli.theme {
        let themes = get_themes();

        let selected_theme_name = match theme_option {
            Some(theme_name) => {

                theme_name.clone()
            }
            None => {

                let theme_options: Vec<String> = themes
                    .iter()
                    .map(|t| format!("{} - {}", t.name, t.description))
                    .collect();

                match Select::new("Select a theme to set as default:", theme_options).prompt() {
                    Ok(selection) => {
                        let theme_name = selection.split(" - ").next().unwrap();
                        theme_name.to_string()
                    }
                    Err(_) => {
                        println!("Theme selection cancelled.");
                        return;
                    }
                }
            }
        };


        if let Some(theme) = themes.iter().find(|t| t.name == selected_theme_name) {
            let config = Config {
                default_theme: theme.name.clone(),
            };
            match save_config(&config) {
                Ok(_) => {
                    println!("✓ Set '{}' as default theme", theme.name);
                    return;
                }
                Err(e) => {
                    eprintln!("Error saving config: {e}");
                    return;
                }
            }
        } else {
            println!(
                "Theme '{}' not found. Available themes:",
                selected_theme_name
            );
            for theme in themes {
                println!("  {} - {}", theme.name, theme.description);
            }
            return;
        }
    }


    if cli.cpu {
        let config = load_config();
        let theme =
            get_theme_by_name(&config.default_theme).unwrap_or_else(|| get_themes()[0].clone());
        show_cpu_info(&theme);
        return;
    }


    if let Some(file_path) = &cli.workspace_file {
        if let Err(e) = workspace::copy_file_to_clipboard(file_path) {
            eprintln!("Error copying file: {e}");
        }
        return;
    }


    if let Some(folder_path) = &cli.workspace_folder {
        if let Err(e) = workspace::copy_folder_to_clipboard(folder_path) {
            eprintln!("Error copying folder: {e}");
        }
        return;
    }


    if cli.workspace {
        if let Err(e) = workspace::print_workspace_snapshot(cli.source_only, cli.max_size) {
            eprintln!("Error printing workspace: {e}");
        }
        return;
    }


    if cli.tree {
        let config = load_config();
        let theme =
            get_theme_by_name(&config.default_theme).unwrap_or_else(|| get_themes()[0].clone());
        show_tree(&theme, cli.depth, cli.all);
        return;
    }


    let config = load_config();
    let theme = get_theme_by_name(&config.default_theme).unwrap_or_else(|| get_themes()[0].clone());
    show_directory_table(&theme);
}

fn show_directory_table(theme: &Theme) {

    let current_dir = env::current_dir().expect("Could not get current directory");


    let entries = fs::read_dir(&current_dir).expect("Could not read current directory");


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
        format!("{size} B")
    }
}

fn format_time(time: SystemTime) -> String {
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
    format!("{user}{group}{other}")
}

#[cfg(windows)]
fn format_permissions(_metadata: &std::fs::Metadata) -> String {


    "N/A".to_string()
}

fn show_tree(theme: &Theme, max_depth: Option<usize>, show_all: bool) {
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
