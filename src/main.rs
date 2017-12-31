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

extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use serde_json::ser::to_string_pretty;

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
    line
}
fn diff(_a:&Path, _b:&Path) -> Result<Child>  {
    Command::new("diff")
            .arg("test.txt")
            .arg("/home/sean/test.txt")
            .stdout(Stdio::piped())
			.spawn()
}

#[derive(Serialize, Deserialize, Debug)]
struct ExecResult {
    stdout: Option<String>,
    stderr: Option<String>,
    exit_code: Option<i32>
}

fn read_command_child(result_child:std::io::Result<Child>) -> Result<(Result<ExitStatus>, Option<String> )> {
    match result_child {
       std::result::Result::Ok(mut child) => 
           std::result::Result::Ok(  //return Ok (T,T)
            ( 
                child.wait(),
                //match child.stdout { std::option::Option::Some(stdout) => Some(readout(stdout)), std::option::Option::None => None }
                child.stdout.map(readout)
           )),
       std::result::Result::Err(e) => Err(e) 
    }
}
fn diff_result_json(optional_stdout:Option<String>) -> serde_json::Value {
    let mut result = json!({ });

    match optional_stdout {
        Some(stdout) => {
            result["stdout"] = json!(stdout);
            result
        },
        None => result
    }
}

//fn json_or_err<'a>(tag:&'a str, option_val:Option<&'a str>)  -> (&'a str,Option<&'a str>) {
//    match option_val {
//       std::option::Option::Some(val) => (tag,Some(val)),
//       std::option::Option::None => (tag,None)
//    }
//}
fn input_json(mut parent:serde_json::Value, src:&Path, dst:&Path) -> serde_json::Value {
    //let mut input = &parent["input"];
    if let Some(src_str) = src.to_str() {
        parent["input"]["src"] = json!(src_str);
    } else {
        parent["input"]["src"] = json!({"!error": "conversion"});
    }

    if let Some(dst_str) = dst.to_str() {
        parent["input"]["dst"] = json!(dst_str);
    } else {
        parent["input"]["dst"] = json!({"!error": "conversion"});
    }
    parent
}
fn main() {
    let src = Path::new("test.txt");
    let dst = Path::new("/home/sean/test.txt");
    let r1 = json!({});
    let mut r2 = input_json(r1, src, dst);
    println!("input {}", r2.to_string());
    r2["result"] = match read_command_child(diff(src, dst)) {
            Ok( t ) => diff_result_json(t.1) ,
            Err(e) =>  json!({"error!": format!("{}", e)})
    };
    println!("result {}", to_string_pretty(&r2).unwrap());
}
