use std::process::Command;
use std::io::Result;
use std::path::*;
//use std::io::Error;
use std::process::Stdio;
use std::process::Child;
use std::process::ChildStdout;
//use std::io::Read;
//use std::io::BufReader;
use std::io::{BufRead, BufReader};

fn readout(out:ChildStdout) -> String {
    let mut buf_reader = BufReader::new(out);
	//let len = reader.read_line(&mut line)?;
	let mut line = String::new();
	loop {
        match buf_reader.read_line(&mut line) {
            Ok(n) => {
                if n < 1 {
                    break;
                }
            },
            Err(e) => { println!("ERROR: read_line '{}'", e); }
        }
    }
    return line;
}
fn diff(_a:&Path, _b:&Path) -> Result<Child>  {
    return Command::new("diff")
            .arg("test.txt")
            .arg("/home/sean/test.txt")
            .stdout(Stdio::piped())
			.spawn();
}
fn read(result:std::io::Result<Child>) -> Option<String> {
	match result {
       Ok(v) => { 
        let maybe_stdout:std::option::Option<std::process::ChildStdout> = v.stdout;
        match maybe_stdout {
            std::option::Option::Some(stdout) => std::option::Option::Some(readout(stdout)),
            std::option::Option::None => None
        }
       }
       Err(_e) => None 
			//	0 => print!("called `Result::unwrap()` on an `Err` value: {:?}", ""),
			//	1 => print!("OK")
			}

}
fn main() {
    let src = Path::new("test.txt");
    let dst = Path::new("/home/sean/test.txt");
    match read(diff(src, dst)) {
            std::option::Option::Some(stdout) => { println!("{}", stdout)},
            std::option::Option::None =>  {println!("failed")},
    }

//			match process.stdout {
}
