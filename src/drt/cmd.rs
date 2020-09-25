extern crate which;

use drt::err::DrtError;
use std::path::PathBuf;

pub fn cmdline(cmd: String, args: Vec<&str>) -> String {
        let mut full = vec![cmd.as_str()];
        full.append(&mut args.to_vec());
        full.join(" ")
}

pub fn exectable_full_path(prg: &str) -> Result<PathBuf, DrtError> {
	let maybe_prg: which::Result<PathBuf> = which::which(prg);
	exectable_full_path_which(prg, maybe_prg)
}
fn exectable_full_path_which(prg: &str, maybe_prg: which::Result<PathBuf>) -> Result<PathBuf, DrtError> {
    match maybe_prg {
        Ok(prg_path) => {
            Ok(prg_path)
        }
        Err(_e) => {
            Err(DrtError::CommandNotFound(String::from(prg)))
        }
    }
}