use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
    println!("📦 Workspace: {}\n", base.display());

    let mut files = Vec::new();
    collect_files(&base, &base, &mut files)?;

    for relative_path in files {
        let full_path = base.join(&relative_path);
        let content =
            fs::read_to_string(&full_path).unwrap_or_else(|_| "<binary or unreadable>".into());

        println!("./{}", relative_path.display());
        println!("---------------------");
        println!("{content}\n");
    }

    Ok(())
}