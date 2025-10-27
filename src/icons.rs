use std::path::Path;

use crate::theme::{IconTheme, Theme};

use lazy_static::lazy_static;

fn load_icon_theme() -> IconTheme {
    // Try to load from icon-theme.yaml in current directory
    if let Ok(theme) = Theme::from_path("icon-theme.yaml") {
        return theme;
    }

    // Try to load from config directory
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join("lsa").join("icon-theme.yaml");
        if let Some(path_str) = config_path.to_str() {
            if let Ok(theme) = Theme::from_path(path_str) {
                return theme;
            }
        }
    }

    // Fall back to default
    IconTheme::default()
}

lazy_static! {
    static ref ICON_THEME: IconTheme = load_icon_theme();
}

pub fn get_file_icon(path: &Path) -> &'static str {
    if path.is_dir() {
        &ICON_THEME.filetype.dir
    } else if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
        ICON_THEME.extension.get(&extension.to_lowercase()).map(|s| s.as_str()).unwrap_or(&ICON_THEME.filetype.file)
    } else {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        ICON_THEME.name.get(&filename).map(|s| s.as_str()).unwrap_or(&ICON_THEME.filetype.file)
    }
}

