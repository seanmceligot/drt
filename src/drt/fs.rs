use std::convert::AsRef;
use std::path::Path;
use drt::userinput::ask;

pub fn create_dir(maybe_path: Option<&Path>) {
    match maybe_path {
        None => {}
        Some(dir) => {
            if !dir.exists() {
                let ans = ask(format!("create directory {} (y/n)", dir.display()));
                match ans.as_ref() {
                    "n" => {
                        println!("skipping mkdir {}", dir.display());
                    }
                    "y" => {
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
