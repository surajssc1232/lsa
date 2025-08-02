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
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

#[derive(Parser)]
#[command(name = "lsr")]
#[command(about = "A colorful directory listing tool with multiple themes")]
#[command(disable_help_flag = true)]
struct Cli {
    /// Show help information
    #[arg(short, long)]
    help: bool,
    /// Show CPU information
    #[arg(long)]
    cpu: bool,
    /// Show workspace snapshot
    #[arg(long)]
    workspace: bool,
    /// Only include source files in workspace (src/, *.rs, *.js, *.py, etc.)
    #[arg(long, requires = "workspace")]
    source_only: bool,
    /// Maximum size limit for workspace output in KB (default: no limit)
    #[arg(long, requires = "workspace", value_name = "KB")]
    max_size: Option<usize>,
    /// Show directory tree view
    #[arg(short = 't', long)]
    tree: bool,
    /// Maximum depth for tree view
    #[arg(short, long, requires = "tree")]
    depth: Option<usize>,
    /// Show all files including hidden ones (for tree view)
    #[arg(short = 'a', long, requires = "tree")]
    all: bool,
    /// Set theme as default (shows interactive menu if no theme name provided)
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
        Theme {
            name: "onedark".to_string(),
            description: "One Dark - Atom's iconic One Dark theme".to_string(),
            border: (97, 175, 239),      // Blue
            header: (198, 120, 221),     // Purple
            file_name: (97, 175, 239),   // Blue
            file_type: (97, 175, 239),   // Blue
            dir_name: (198, 120, 221),   // Purple
            dir_type: (198, 120, 221),   // Purple
            file_size: (152, 195, 121),  // Green
            dir_size: (92, 99, 112),     // Comment Gray
            modified: (229, 192, 123),   // Yellow
            permissions: (86, 182, 194), // Cyan
            row_number: (171, 178, 191), // Light Gray
        },
        Theme {
            name: "material".to_string(),
            description: "Material - Google's Material Design color palette".to_string(),
            border: (100, 181, 246),    // Blue 300
            header: (156, 39, 176),     // Purple 600
            file_name: (33, 150, 243),  // Blue 500
            file_type: (33, 150, 243),  // Blue 500
            dir_name: (103, 58, 183),   // Deep Purple 500
            dir_type: (103, 58, 183),   // Deep Purple 500
            file_size: (76, 175, 80),   // Green 500
            dir_size: (117, 117, 117),  // Grey 600
            modified: (255, 152, 0),    // Orange 500
            permissions: (0, 188, 212), // Cyan 500
            row_number: (97, 97, 97),   // Grey 700
        },
        Theme {
            name: "oceanic-next".to_string(),
            description: "Oceanic Next - Sophisticated oceanic color scheme".to_string(),
            border: (101, 115, 126),     // Base Gray
            header: (192, 197, 206),     // Light Gray
            file_name: (102, 153, 204),  // Blue
            file_type: (102, 153, 204),  // Blue
            dir_name: (193, 132, 1),     // Orange
            dir_type: (193, 132, 1),     // Orange
            file_size: (153, 173, 106),  // Green
            dir_size: (79, 91, 102),     // Dark Gray
            modified: (250, 208, 122),   // Yellow
            permissions: (95, 179, 151), // Teal
            row_number: (160, 168, 180), // Medium Gray
        },
        Theme {
            name: "ayu-dark".to_string(),
            description: "Ayu Dark - Modern minimalist dark theme".to_string(),
            border: (83, 89, 97),         // Border
            header: (255, 173, 0),        // Orange
            file_name: (89, 181, 230),    // Blue
            file_type: (89, 181, 230),    // Blue
            dir_name: (209, 154, 102),    // Yellow
            dir_type: (209, 154, 102),    // Yellow
            file_size: (172, 203, 115),   // Green
            dir_size: (92, 103, 115),     // Comment
            modified: (242, 151, 24),     // Orange Light
            permissions: (128, 203, 196), // Cyan
            row_number: (151, 165, 180),  // Foreground
        },
        Theme {
            name: "synthwave".to_string(),
            description: "Synthwave - Retro 80s cyberpunk aesthetics".to_string(),
            border: (241, 250, 140),     // Neon Yellow
            header: (255, 20, 147),      // Deep Pink
            file_name: (0, 255, 255),    // Cyan
            file_type: (0, 255, 255),    // Cyan
            dir_name: (255, 20, 147),    // Deep Pink
            dir_type: (255, 20, 147),    // Deep Pink
            file_size: (50, 255, 50),    // Bright Green
            dir_size: (139, 69, 19),     // Saddle Brown
            modified: (255, 165, 0),     // Orange
            permissions: (255, 0, 255),  // Magenta
            row_number: (255, 255, 255), // White
        },
        Theme {
            name: "github-dark".to_string(),
            description: "GitHub Dark - GitHub's official dark theme".to_string(),
            border: (48, 54, 61),         // Border
            header: (255, 255, 255),      // White
            file_name: (121, 192, 255),   // Blue
            file_type: (121, 192, 255),   // Blue
            dir_name: (255, 184, 108),    // Orange
            dir_type: (255, 184, 108),    // Orange
            file_size: (63, 185, 80),     // Green
            dir_size: (139, 148, 158),    // Gray
            modified: (255, 235, 59),     // Yellow
            permissions: (164, 196, 255), // Light Blue
            row_number: (201, 209, 217),  // Light Gray
        },
        Theme {
            name: "cobalt2".to_string(),
            description: "Cobalt2 - Electric blue theme for night owls".to_string(),
            border: (0, 122, 204),       // Deep Blue
            header: (255, 204, 102),     // Orange
            file_name: (158, 206, 106),  // Green
            file_type: (158, 206, 106),  // Green
            dir_name: (255, 204, 102),   // Orange
            dir_type: (255, 204, 102),   // Orange
            file_size: (102, 217, 239),  // Cyan
            dir_size: (128, 128, 128),   // Gray
            modified: (255, 198, 109),   // Yellow
            permissions: (255, 157, 77), // Light Orange
            row_number: (193, 193, 193), // Light Gray
        },
        Theme {
            name: "palenight".to_string(),
            description: "Palenight - A darker Material Theme variant".to_string(),
            border: (130, 170, 255),      // Blue
            header: (199, 146, 234),      // Purple
            file_name: (130, 170, 255),   // Blue
            file_type: (130, 170, 255),   // Blue
            dir_name: (255, 203, 107),    // Yellow
            dir_type: (255, 203, 107),    // Yellow
            file_size: (195, 232, 141),   // Green
            dir_size: (103, 110, 149),    // Comment
            modified: (255, 158, 100),    // Orange
            permissions: (137, 221, 255), // Cyan
            row_number: (171, 178, 191),  // Foreground
        },
        Theme {
            name: "night-owl".to_string(),
            description: "Night Owl - A theme for night owls by Sarah Drasner".to_string(),
            border: (130, 170, 255),      // Blue
            header: (199, 146, 234),      // Purple
            file_name: (130, 170, 255),   // Blue
            file_type: (130, 170, 255),   // Blue
            dir_name: (195, 232, 141),    // Green
            dir_type: (195, 232, 141),    // Green
            file_size: (255, 203, 107),   // Yellow
            dir_size: (103, 110, 149),    // Comment
            modified: (255, 158, 100),    // Orange
            permissions: (137, 221, 255), // Cyan
            row_number: (214, 222, 235),  // Foreground
        },
        Theme {
            name: "horizon".to_string(),
            description: "Horizon - A beautifully warm dual theme".to_string(),
            border: (250, 183, 149),      // Coral
            header: (236, 196, 141),      // Apricot
            file_name: (156, 207, 216),   // Turquoise
            file_type: (156, 207, 216),   // Turquoise
            dir_name: (250, 183, 149),    // Coral
            dir_type: (250, 183, 149),    // Coral
            file_size: (158, 180, 158),   // Sage
            dir_size: (87, 82, 74),       // Comment
            modified: (236, 196, 141),    // Apricot
            permissions: (229, 152, 155), // Rose
            row_number: (203, 204, 198),  // Foreground
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
        "\u{f07b}" // 
    } else if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
        match extension.to_lowercase().as_str() {
            // Programming languages
            "rs" => "\u{e7a8}",                           //
            "py" => "\u{e73c}",                           //
            "js" | "mjs" => "\u{e74e}",                   //
            "ts" => "\u{e628}",                           //
            "jsx" => "\u{e7ba}",                          //
            "tsx" => "\u{e7ba}",                          //
            "go" => "\u{e626}",                           //
            "java" => "\u{e204}",                         //
            "c" => "\u{e61e}",                            //
            "h" => "\u{e61e}",                            //
            "cpp" | "cc" | "cxx" => "\u{e61d}",           //
            "hpp" => "\u{e61d}",                          //
            "cs" => "\u{f81a}",                           // 󰠚
            "php" => "\u{e73d}",                          //
            "rb" => "\u{e21e}",                           //
            "swift" => "\u{e755}",                        //
            "kt" => "\u{e634}",                           //
            "dart" => "\u{e798}",                         //
            "lua" => "\u{e620}",                          //
            "r" => "\u{f25d}",                            //
            "scala" => "\u{e737}",                        //
            "clj" | "cljs" => "\u{e768}",                 //
            "hs" => "\u{e777}",                           //
            "ml" | "mli" => "\u{e7a7}",                   //
            "elm" => "\u{e62c}",                          //
            "ex" | "exs" => "\u{e62d}",                   //
            "erl" => "\u{e7b1}",                          //
            "vim" => "\u{e62b}",                          //
            "sh" | "bash" | "zsh" | "fish" => "\u{f489}", //
            "ps1" => "\u{f489}",                          //
            // Web technologies
            "html" | "htm" => "\u{e60e}",  //
            "css" => "\u{e614}",           //
            "scss" | "sass" => "\u{e603}", //
            "less" => "\u{e758}",          //
            "vue" => "\u{fd42}",           // ﵂
            "svelte" => "\u{e697}",        //
            "angular" => "\u{e753}",       //
            // Configuration and data
            "json" => "\u{e60b}",            //
            "toml" => "\u{e615}",            //
            "yaml" | "yml" => "\u{f481}",    //
            "xml" => "\u{e619}",             //
            "ini" => "\u{f17a}",             //
            "conf" | "config" => "\u{e615}", //
            "env" => "\u{f462}",             //
            "dockerfile" => "\u{f308}",      //
            "makefile" => "\u{f728}",        //
            // Documentation
            "md" | "markdown" => "\u{e609}", //
            "txt" => "\u{f15c}",             //
            "rst" => "\u{f15c}",             //
            "tex" => "\u{e600}",             //
            "rtf" => "\u{f15c}",             //
            "pdf" => "\u{f1c1}",             //
            // Images
            "png" | "jpg" | "jpeg" => "\u{f1c5}", //
            "gif" => "\u{f1c5}",                  //
            "svg" => "\u{fc1f}",                  // ﰟ
            "bmp" | "tiff" | "tif" => "\u{f1c5}", //
            "webp" => "\u{f1c5}",                 //
            "ico" => "\u{f1c5}",                  //
            "psd" => "\u{e7b8}",                  //
            "ai" => "\u{e7b4}",                   //
            // Archives
            "zip" | "7z" | "rar" => "\u{f410}",                 //
            "tar" | "gz" | "gzip" | "bz2" | "xz" => "\u{f410}", //
            // Executables and binaries
            "exe" | "msi" => "\u{f17a}",         //
            "app" | "dmg" => "\u{f179}",         //
            "deb" | "rpm" | "pkg" => "\u{f187}", //
            "appimage" => "\u{f179}",            //
            // Documents
            "doc" | "docx" => "\u{f1c2}",        //
            "xls" | "xlsx" => "\u{f1c3}",        //
            "ppt" | "pptx" => "\u{f1c4}",        //
            "odt" | "ods" | "odp" => "\u{f1c2}", //
            // Audio
            "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" | "wma" => "\u{f001}", //
            // Video
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "webm" | "flv" | "m4v" => "\u{f03d}", //
            // Fonts
            "ttf" | "otf" | "woff" | "woff2" | "eot" => "\u{f031}", //
            // Database
            "db" | "sqlite" | "sqlite3" => "\u{f1c0}", //
            "sql" => "\u{f1c0}",                       //
            // Lock and temp files
            "lock" => "\u{f023}",           //
            "tmp" | "temp" => "\u{f2ed}",   //
            "bak" | "backup" => "\u{f56e}", //
            "log" => "\u{f18e}",            //
            // Other
            _ => "\u{f15b}",
        }
    } else {
        // Handle special filenames without extensions
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match filename.as_str() {
            "dockerfile" => "\u{f308}",   //
            "makefile" => "\u{f728}",     //
            "readme" => "\u{f7fb}",       //
            "license" => "\u{f718}",      //
            "changelog" => "\u{f7d9}",    //
            "cargo.toml" => "\u{e7a8}",   //
            "package.json" => "\u{e718}", //
            ".gitignore" => "\u{f1d3}",   //
            ".gitmodules" => "\u{f1d3}",  //
            ".env" => "\u{f462}",         //
            _ => "\u{f15b}",              //
        }
    }
}

fn get_theme_by_name(name: &str) -> Option<Theme> {
    get_themes().into_iter().find(|t| t.name == name)
}

fn show_cpu_info(theme: &Theme) {
    // Run lscpu command and parse the output
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

    // Parse lscpu output
    let mut cpu_info = Vec::new();
    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            cpu_info.push((key.trim().to_string(), value.trim().to_string()));
        }
    }

    // Create table with CPU information
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

    // Add CPU information to table with alternating colors
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

    // Show the table with colored borders
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

    // Print title with theme colors
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

    // Show the table with colored borders
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

    // Handle --help flag with custom display
    if cli.help {
        let config = load_config();
        let theme =
            get_theme_by_name(&config.default_theme).unwrap_or_else(|| get_themes()[0].clone());
        show_help(&theme);
        return;
    }

    // Handle --theme flag
    if let Some(theme_option) = &cli.theme {
        let themes = get_themes();

        let selected_theme_name = match theme_option {
            Some(theme_name) => {
                // Theme name provided directly
                theme_name.clone()
            }
            None => {
                // No theme name provided, show interactive menu
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

        // Find and save the selected theme
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

    // Handle --cpu flag
    if cli.cpu {
        let config = load_config();
        let theme =
            get_theme_by_name(&config.default_theme).unwrap_or_else(|| get_themes()[0].clone());
        show_cpu_info(&theme);
        return;
    }

    // Handle --workspace flag
    if cli.workspace {
        if let Err(e) = workspace::print_workspace_snapshot(cli.source_only, cli.max_size) {
            eprintln!("Error printing workspace: {e}");
        }
        return;
    }

    // Handle --tree flag
    if cli.tree {
        let config = load_config();
        let theme =
            get_theme_by_name(&config.default_theme).unwrap_or_else(|| get_themes()[0].clone());
        show_tree(&theme, cli.depth, cli.all);
        return;
    }

    // Default behavior: show directory listing with saved theme
    let config = load_config();
    let theme = get_theme_by_name(&config.default_theme).unwrap_or_else(|| get_themes()[0].clone());
    show_directory_table(&theme);
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

        // Format size with theme colors
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

        // Format modified time with theme colors
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

        // Format permissions with theme color
        let permissions_cell =
            Cell::new(format_permissions(metadata.permissions().mode())).fg(Color::Rgb {
                r: theme.permissions.0,
                g: theme.permissions.1,
                b: theme.permissions.2,
            });

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

    // Step 5: Show the table with colored borders
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

fn show_tree(theme: &Theme, max_depth: Option<usize>, show_all: bool) {
    let current_dir = env::current_dir().expect("Could not get current directory");

    // Print the root directory
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

    // Build and display the tree
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
    // Check depth limit
    if let Some(max) = max_depth {
        if current_depth >= max {
            return;
        }
    }

    // Read directory entries
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    // Collect and sort entries
    let mut items: Vec<_> = entries.filter_map(|entry| entry.ok()).collect();
    items.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();

        // Directories first, then files
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    // Filter hidden files if not showing all
    if !show_all {
        items.retain(|item| !item.file_name().to_string_lossy().starts_with('.'));
    }

    let total_items = items.len();

    for (index, entry) in items.iter().enumerate() {
        let path = entry.path();
        let is_last_item = index == total_items - 1;

        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        // Choose tree characters
        let (current_prefix, next_prefix) = if is_last_item {
            ("└── ", "    ")
        } else {
            ("├── ", "│   ")
        };

        // Get file icon and colors
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

        // Color the tree structure
        let tree_color = format!(
            "\x1b[38;2;{};{};{}m",
            theme.border.0, theme.border.1, theme.border.2
        );
        let reset_color = "\x1b[0m";

        // Print the current item
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

        // Recursively display subdirectories
        if path.is_dir() {
            let new_prefix = format!("{}{}", prefix, next_prefix);
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
