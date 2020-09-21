extern crate which;

use drt::err::DrtError;
use std::path::PathBuf;
use drt::err::Verb;
use drt::err::log_cmd_action;

pub fn exectable_full_path(prg: &str) -> Result<PathBuf, DrtError> {
	let maybe_prg: which::Result<PathBuf> = which::which(prg);
	exectable_full_path_which(prg, maybe_prg)
}
fn exectable_full_path_which(prg: &str, maybe_prg: which::Result<PathBuf>) -> Result<PathBuf, DrtError> {
    match maybe_prg {
        Ok(prg_path) => {
            log_cmd_action("run", Verb::WOULD, prg_path.to_string_lossy());
            Ok(prg_path)
        }
        Err(_e) => {
            Err(DrtError::CommandNotFound(String::from(prg)))
        }
    }
}
