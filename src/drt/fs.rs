use drt::userinput::ask;
use std::path::Path;

#[allow(dead_code)]
pub fn create_dir(maybe_path: Option<&Path>) {
    match maybe_path {
        None => {}
        Some(dir) => {
            if !dir.exists() {
                let ans = ask(format!("create directory {} (y/n)", dir.display()).as_str());
                match ans {
                    'n' => {
                        println!("skipping mkdir {}", dir.display());
                    }
                    'y' => {
                        println!("mkdir {}", dir.display());
                        std::fs::create_dir_all(dir)
                            .expect(&format!("create dir failed: {}", dir.display()));
                        if !dir.exists() {
                            println!("dir not found {}", dir.display());
                        }
                    }
                    _ => create_dir(maybe_path), //repeat the question
                }
            }
        }
    }
}
