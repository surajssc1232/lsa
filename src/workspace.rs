use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
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

    // Copy to clipboard
    match ClipboardContext::new() {
        Ok(mut ctx) => {
            match ctx.set_contents(workspace_content) {
                Ok(_) => println!("\u{f00c} Workspace content copied to clipboard!"),
                Err(e) => {
                    eprintln!("\u{f00d} Failed to copy to clipboard: {}", e);
                    return Err(io::Error::new(io::ErrorKind::Other, e));
                }
            }
        }
        Err(e) => {
            eprintln!("\u{f00d} Failed to access clipboard: {}", e);
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }
    }

    Ok(())
}