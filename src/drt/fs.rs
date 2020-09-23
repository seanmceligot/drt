extern crate libc;
use drt::userinput::ask;
use std::path::Path;
use drt::Mode;
use drt::err::{DrtError, log_path_action, Verb::SKIPPED};
use log::trace;
use std::ffi::CString;


#[test]
fn test_parent() {
    //assert_eq!(can_create_dir_maybe(Path::new("/root/test").parent()).is_err(), true);
    //assert_eq!(can_create_dir_maybe(Path::new("/tmp/test").parent()).is_ok(), true);
}
fn access_x(path: &Path) -> bool {
 	let cstr = CString::new(path.display().to_string()).unwrap();
	unsafe {
		match libc::faccessat(libc::AT_FDCWD, cstr.as_ptr(), libc::X_OK, libc::AT_EACCESS) as isize {
			0 => true,
			_ => false
		}
	}
}
pub fn can_create_dir_maybe(maybe_dir: Option<&Path>) ->  Result<&Path, DrtError> {
    trace!("can_create_dir_maybe {:?}", maybe_dir);
    match maybe_dir {
        Some(dir) => can_create_dir(dir),
        None => Err(DrtError::PathNotFound0)
    }
}
pub fn can_create_dir(dir: &Path) ->  Result<&Path, DrtError> {
    trace!("can_create_dir{:?}", dir);
    if dir.exists() {
		if access_x(dir) {
			Ok(dir)
		} else {
            Err(DrtError::InsufficientPrivileges(dir.display().to_string()))
		}
    } else {
        can_create_dir_maybe(dir.parent())
    }
}
pub fn create_dir_maybe(mode: Mode, maybe_dir: Option<&Path>) -> Result<&Path, DrtError> {
    trace!("create_dir_maybe {:?}", maybe_dir);
    match maybe_dir {
        Some(dir) => create_dir(mode, dir),
        None => Err(DrtError::PathNotFound0)
    }
}
pub fn create_dir(mode: Mode, dir: &Path) -> Result<&Path, DrtError> {
    trace!("create_dir{:?}", dir);
    if dir.exists() {
        Ok(dir)
    } else {
        let ans = match mode {
            Mode::Passive => 'n',
            Mode::Active => 'y',
            Mode::Interactive => ask(format!("create directory {} (y/n)", dir.display()).as_str())
        };
        match ans {
            'n' => {
                match can_create_dir_maybe(dir.parent()) {
                    Err(e) => Err(e),
                    Ok(dir) => {
                        log_path_action("create dir", SKIPPED, dir);
                        Ok(dir)
                    }
                } 
            },
            'y' => {
                println!("mkdir {}", dir.display());
                match std::fs::create_dir_all(dir) {
                    Err(e) => Err(DrtError::IoError(e)),
                    Ok(_) => Ok(dir)
                }
            },
            _ => create_dir(mode, dir ) //repeat the question
        }
    }
}
