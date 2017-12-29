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

fn readout(out:ChildStdout) {
    let mut buf_reader = BufReader::new(out);
	let mut line = String::new();
	//let len = reader.read_line(&mut line)?;
	match buf_reader.read_line(&mut line) {
		Ok(n) => {
			print!("line: {}", line);
			print!("n: {}", n);
		},
		Err(e) => { println!("ERROR: read_line '{}'", e) }
	}
//	for line in buf_reader.lines() {
//		match line {
//			Ok(l) => {
//    			println!("stdout:\n{}", l);
//			},
//			Err(_) => return,
//		};
//	}
}
fn diff(_a:&Path, _b:&Path) -> Result<Child>  {
    return Command::new("diff")
            .arg("test.txt")
            .arg("/home/sean/test.txt")
            .stdout(Stdio::piped())
			.spawn();
}
fn read(result:std::io::Result<Child>) -> Option<&'static str> {
	match result {
       Ok(v) => { 
        let maybe_stdout:std::option::Option<std::process::ChildStdout> = v.stdout;
        match maybe_stdout {
            std::option::Option::Some(stdout) => {
			readout(stdout);
            Some("")
            },
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
    read(diff(src, dst));

//			match process.stdout {
}
