use drt::userinput::ask;
use std::path::Path;
use drt::Mode;

#[test]
fn test_parent() {
    assert_eq!(is_empty(Path::new("test").parent()), true);
    assert_eq!(is_empty(Path::new("/tmp/test").parent()), false);
}
pub fn is_empty(maybe_path: Option<&Path>) -> bool {
    match maybe_path {
        None => false,
        Some(p) => match p.to_str() {
            None => false,
            Some(s) => s.len() == 0,
        },
    }
}
pub fn create_dir(mode: Mode, maybe_path: Option<&Path>) -> Option<&Path> {
    if !is_empty(maybe_path) {
        match maybe_path {
            None => None,
            Some(dir) => {
                if !dir.exists() {
                    let ans = match mode {
                        Mode::Passive => 'n',
                        Mode::Active => 'y',
                        Mode::Interactive => ask(format!("create directory {} (y/n)", dir.display()).as_str())
                    };
                    match ans {
                        'n' => {
                            println!("skipping mkdir {}", dir.display());
                            // TODO: verify that we could create dir
                            None
                        },
                        'y' => {
                            println!("mkdir {}", dir.display());
                            std::fs::create_dir_all(dir)
                                .unwrap_or_else(|_| panic!("create dir failed: {}", dir.display()));
                            if !dir.exists() {
                                println!("dir not found {}", dir.display());
                                None
                            } else {
                                Some(dir)
                            }
                        },
                        _ => create_dir(mode, maybe_path), //repeat the question
                    }
                } else {
                    None
                }
            }
        }
    } else {
        None
    }
}
