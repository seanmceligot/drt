use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::vec::IntoIter;
use log::debug;

#[derive(Debug)]
pub struct DiffText<'f> {
    pub text: &'f IntoIter<u8>,
}

pub enum DiffStatus {
    NoChanges,
    NewFile,
    Changed(IntoIter<u8>),
    Failed,
}

pub fn diff<'f,'g>(path: &'f Path, path2: &'f Path) -> DiffStatus {
    debug!("diff {} {}", path.display(), path2.display());
    if !path2.exists() {
        DiffStatus::NewFile
    } else {
        let output = Command::new("diff")
            .arg(path)
            .arg(path2)
            .output()
            .expect("diff failed");
        //io::stdout().write_all(&output.stdout).unwrap();
        match output.status.code().unwrap() {
            1 => DiffStatus::Changed(output.stdout.into_iter()),
            2 => DiffStatus::Failed,
            0 => DiffStatus::NoChanges,
            _ => DiffStatus::Failed,
        }
    }
}
