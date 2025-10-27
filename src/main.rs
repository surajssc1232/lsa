mod config;
mod display;
mod icons;
mod parser;
mod theme;
mod themes;
mod utils;
mod workspace;

use clap::Parser;
use inquire::Select;

use config::{Config, load_config, save_config};
use display::{show_cpu_info, show_directory_table, show_help, show_path_table, show_structured_data, show_tree};
use themes::{get_theme_by_name, get_themes};

#[derive(Parser)]
#[command(name = "lsa")]
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

    #[arg(long)]
    path: bool,

    #[arg(long, value_name = "FILE_PATH")]
    open: Option<String>,

    #[arg(value_name = "DIRECTORY")]
    directory: Option<String>,
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
            Some(theme_name) => theme_name.clone(),
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

    if cli.path {
        let config = load_config();
        let theme =
            get_theme_by_name(&config.default_theme).unwrap_or_else(|| get_themes()[0].clone());
        show_path_table(&theme);
        return;
    }

    if let Some(file_path) = &cli.open {
        let config = load_config();
        let theme =
            get_theme_by_name(&config.default_theme).unwrap_or_else(|| get_themes()[0].clone());
        show_structured_data(&theme, file_path);
        return;
    }

    let config = load_config();
    let theme = get_theme_by_name(&config.default_theme).unwrap_or_else(|| get_themes()[0].clone());
    show_directory_table(&theme, cli.directory.as_deref());
}

