use std::path::Path;

pub fn get_file_icon(path: &Path) -> &'static str {
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