use std::process::Command;
use std::io::Result;
use std::path::*;
use std::process::Stdio;
//use std::process::Child;
use std::process::ChildStdout;
//use std::io::Read;
//use std::io::BufReader;
use std::io::{BufRead, BufReader};
//use std::process::ExitStatus;
use std::process::Output;

extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use serde_json::ser::to_string_pretty;

//use serde_json;//::{Value,to_string};

fn _childstdout_to_string(out:ChildStdout) -> String {
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
#[derive(Serialize, Deserialize, Debug)]
struct ExecReturn {
    stdout: String,
    stderr: String,
    exit_code: Option<i32>
}
fn output_to_execresult(output:Output) -> ExecReturn {
    let o:Vec<u8> = output.stdout;
    let e:Vec<u8> = output.stderr;
    ExecReturn { 
        stdout: String::from_utf8(o).unwrap(),
        stderr: String::from_utf8(e).unwrap(),
        exit_code: output.status.code() 
   } 
}
fn diff(_a:&Path, _b:&Path) -> Result<ExecReturn>  {
    let result_output= Command::new("diff")
            .arg("test.txt")
            .arg("/home/sean/test.txt")
            .stdout(Stdio::piped())
			.output();
    result_output.map( output_to_execresult )
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
fn difftask(src:String,dst:String) {
}

fn main() {
    let src = Path::new("test.txt");
    let dst = Path::new("/home/sean/test.txt");
    let r1 = json!({});
    let mut r2 = input_json(r1, src, dst);
    println!("input {}", r2.to_string());
    r2["result"] = match diff(src, dst) {
            Ok( t ) => json!(t) ,
            Err(e) =>  json!({"error!": format!("{}", e)})
    };
    println!("result {}", to_string_pretty(&r2).unwrap());
}
