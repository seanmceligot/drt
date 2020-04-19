use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

pub enum DiffStatus {
    NoChanges,
    NewFile,
    Changed,
    Failed,
}
pub fn diff(path: &Path, path2: &Path) -> DiffStatus {
    println!("diff {} {}", path.display(), path2.display());
    if !path2.exists() {
        DiffStatus::NewFile
    } else {
        let output = Command::new("diff")
            .arg(path)
            .arg(path2)
            .output()
            .expect("diff failed");
        io::stdout().write_all(&output.stdout).unwrap();
        match output.status.code().unwrap() {
            1 => DiffStatus::Changed,
            2 => DiffStatus::Failed,
            0 => DiffStatus::NoChanges,
            _ => DiffStatus::Failed,
        }
    }
}

