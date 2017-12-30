use std::process::Command;
use std::io::Result;
use std::path::*;
use std::process::Stdio;
use std::process::Child;
use std::process::ChildStdout;
//use std::io::Read;
//use std::io::BufReader;
use std::io::{BufRead, BufReader};
use std::process::ExitStatus;

#[macro_use]
extern crate serde_json;

//use serde_json;//::{Value,to_string};

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
fn read(result:std::io::Result<Child>) -> Result<(Result<ExitStatus>, Option<String> )> {
    match result {
       std::result::Result::Ok(mut child) => 
           std::result::Result::Ok(  //return Ok (T,T)
            ( 
                child.wait(),
                match child.stdout {
                    std::option::Option::Some(stdout) => Some(readout(stdout)),
                    std::option::Option::None => None
                }
           )),
       std::result::Result::Err(e) => Err(e) 
    }
}
fn result(maybe_stdout:Option<String>) -> serde_json::Value {
    match maybe_stdout {
    Some(stdout) => json!({
	"result": "patch",
	  "diff": {
		"args": "diff test.txt /home/sean/test.txt",
		"returncode": 1,
		"stderr": "",
		"stdout": stdout
	  }}),
    None => json!({
	"result": "patch",
	  "diff": {
		"args": "diff test.txt /home/sean/test.txt",
		"returncode": 1,
		"stderr": "",
	  }})
    }
}
fn main() {
    let src = Path::new("test.txt");
    let dst = Path::new("/home/sean/test.txt");
    match read(diff(src, dst)) {
            Ok(t) => { 
                let (maybe_status, maybe_stdout) = t;
                let json = result(maybe_stdout);
                println!("{}", json.to_string());
            },
            Err(e) =>  {println!("error {}", e)}
    }
}
