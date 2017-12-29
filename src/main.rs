use std::process::Command;
use std::io::Result;
use std::path::*;
//use std::io::Error;
use std::process::Stdio;
use std::process::Child;

fn diff(a:&Path, b:&Path) -> Result<Child>  {
    return Command::new("diff")
            .arg("test.txt")
            .arg("/home/sean/test.txt")
            .stdout(Stdio::piped())
			.spawn();
}
fn read(result:std::io::Result<Child>) -> Option<&'static str> {
	match result {
       Ok(v) => { 
        let stdout:std::option::Option<std::process::ChildStdout> = v.stdout;
        match stdout {
            std::option::Option::Some(w) => Some(""),
            std::option::Option::None => None
        }
       }
       Err(e) => None 
			//	0 => print!("called `Result::unwrap()` on an `Err` value: {:?}", ""),
			//	1 => print!("OK")
			}

}
fn main() {
    let src = Path::new("test.txt");
    let dst = Path::new("/home/sean/test.txt");
    read(diff(src, dst));

//			match process.stdout {
}
