use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use copypasta::{ClipboardContext, ClipboardProvider};

pub fn collect_files(dir: &Path, base: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, base, files)?;
        } else {
            files.push(path.strip_prefix(base).unwrap().to_path_buf());
        }
    }
    Ok(())
}

fn copy_to_clipboard(content: &str) -> io::Result<()> {
    // Try wl-copy first (Wayland)
    if let Ok(mut child) = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            if let Err(e) = stdin.write_all(content.as_bytes()) {
                eprintln!("Failed to write to wl-copy: {}", e);
            }
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

pub fn print_workspace_snapshot() -> io::Result<()> {
    let base = env::current_dir()?;
    let mut workspace_content = String::new();
    
    workspace_content.push_str(&format!("\u{f07c} Workspace: {}\n\n", base.display()));

    let mut files = Vec::new();
    collect_files(&base, &base, &mut files)?;

    for relative_path in files {
        let full_path = base.join(&relative_path);
        let content =
            fs::read_to_string(&full_path).unwrap_or_else(|_| "<binary or unreadable>".into());

        workspace_content.push_str(&format!("./{}\n", relative_path.display()));
        workspace_content.push_str("---------------------\n");
        workspace_content.push_str(&format!("{}\n\n", content));
    }

    copy_to_clipboard(&workspace_content)
}