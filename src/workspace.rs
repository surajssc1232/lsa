use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use copypasta::{ClipboardContext, ClipboardProvider};
use ignore::WalkBuilder;

pub fn collect_files(base: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    let walker = WalkBuilder::new(base)
        .hidden(false)  // Don't automatically ignore hidden files
        .git_ignore(true)  // Respect .gitignore
        .git_exclude(true)  // Respect .git/info/exclude
        .git_global(true)   // Respect global gitignore
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();
                
                // Skip .git directory entirely
                if path.components().any(|c| c.as_os_str() == ".git") {
                    continue;
                }
                
                // Only include files, not directories
                if path.is_file() {
                    if let Ok(relative_path) = path.strip_prefix(base) {
                        files.push(relative_path.to_path_buf());
                    }
                }
            }
            Err(err) => {
                eprintln!("Warning: Error walking directory: {}", err);
            }
        }
    }
    
    Ok(())
}

fn is_source_file(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
        match extension.to_lowercase().as_str() {
            // Programming languages
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "c" | "h" | "cpp" | "cc" | "cxx" | "hpp" |
            "cs" | "php" | "rb" | "swift" | "kt" | "dart" | "lua" | "r" | "scala" | "clj" | "cljs" | "hs" |
            "ml" | "mli" | "elm" | "ex" | "exs" | "erl" | "vim" | "sh" | "bash" | "zsh" | "fish" | "ps1" |
            // Web technologies
            "html" | "htm" | "css" | "scss" | "sass" | "less" | "vue" | "svelte" |
            // Configuration and data
            "json" | "toml" | "yaml" | "yml" | "xml" | "ini" | "conf" | "config" | "env" |
            // Documentation
            "md" | "markdown" | "txt" | "rst" | "tex" | "rtf" => true,
            _ => false,
        }
    } else {
        // Handle special filenames without extensions
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        matches!(filename.as_str(), 
            "dockerfile" | "makefile" | "readme" | "license" | "changelog" | 
            "cargo.toml" | "package.json" | ".gitignore" | ".gitmodules" | ".env"
        )
    }
}

fn copy_to_clipboard(content: &str) -> io::Result<()> {
    // Try wl-copy first (Wayland)
    if let Ok(mut child) = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            match stdin.write_all(content.as_bytes()) {
                Ok(_) => (),
                Err(e) => eprintln!("Failed to write to wl-copy: {}", e),
            }
            drop(stdin); // Close stdin to signal EOF
        }
        
        match child.wait() {
            Ok(status) if status.success() => {
                println!("\u{f00c} Workspace content copied to clipboard!");
                return Ok(());
            }
            Ok(_) => eprintln!("wl-copy failed"),
            Err(e) => eprintln!("Failed to wait for wl-copy: {}", e),
        }
    }

    // Try xclip as fallback (X11)
    if let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            if let Err(e) = stdin.write_all(content.as_bytes()) {
                eprintln!("Failed to write to xclip: {}", e);
            }
        }
        
        match child.wait() {
            Ok(status) if status.success() => {
                println!("\u{f00c} Workspace content copied to clipboard!");
                return Ok(());
            }
            Ok(_) => eprintln!("xclip failed"),
            Err(e) => eprintln!("Failed to wait for xclip: {}", e),
        }
    }

    // Try copypasta as final fallback
    match ClipboardContext::new() {
        Ok(mut ctx) => {
            match ctx.set_contents(content.to_string()) {
                Ok(_) => {
                    println!("\u{f00c} Workspace content copied to clipboard!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("\u{f00d} Failed to copy to clipboard: {}", e);
                    Err(io::Error::new(io::ErrorKind::Other, e))
                }
            }
        }
        Err(e) => {
            eprintln!("\u{f00d} Failed to access clipboard: {}", e);
            Err(io::Error::new(io::ErrorKind::Other, e))
        }
    }
}

pub fn print_workspace_snapshot(source_only: bool, max_size_kb: Option<usize>) -> io::Result<()> {
    let base = env::current_dir()?;
    let mut workspace_content = String::new();
    
    workspace_content.push_str(&format!("\u{f07c} Workspace: {}\n\n", base.display()));

    let mut files = Vec::new();
    collect_files(&base, &mut files)?;

    // Filter for source files only if requested
    if source_only {
        files.retain(|path| is_source_file(path));
    }

    let max_size_bytes = max_size_kb.map(|kb| kb * 1024);
    let mut total_size = workspace_content.len();

    for relative_path in files {
        let full_path = base.join(&relative_path);
        let content =
            fs::read_to_string(&full_path).unwrap_or_else(|_| "<binary or unreadable>".into());

        let file_section = format!("./{}\n---------------------\n{}\n\n", relative_path.display(), content);
        
        // Check size limit if specified
        if let Some(max_bytes) = max_size_bytes {
            if total_size + file_section.len() > max_bytes {
                workspace_content.push_str(&format!("... (truncated due to {}KB size limit)\n", max_size_kb.unwrap()));
                break;
            }
        }

        workspace_content.push_str(&file_section);
        total_size += file_section.len();
    }

    // Add size information
    let size_kb = total_size as f64 / 1024.0;
    println!("Workspace size: {:.1}KB ({} characters)", size_kb, total_size);
    
    if size_kb > 32.0 {
        println!("⚠️  Content is large (>32KB). Some LLMs may truncate input.");
        println!("💡 Try: lsr --workspace --source-only or --max-size 32");
    }

    copy_to_clipboard(&workspace_content)
}