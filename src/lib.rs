
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::path::Path;

mod exec {
use std::process::Output;
use serde_json::Value;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecReturn {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>
}


pub fn output_to_execresult(output:Output) -> ExecReturn {
    let o:Vec<u8> = output.stdout;
    let e:Vec<u8> = output.stderr;
    ExecReturn { 
        stdout: String::from_utf8(o).unwrap(),
        stderr: String::from_utf8(e).unwrap(),
        exit_code: output.status.code() 
   } 
}

pub fn input_json(mut parent:Value, src:&Path, dst:&Path) -> Value {
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
}
pub mod eval {
use std::process::Command;
use std::process::Stdio;
use std::path::Path;
use serde_json::ser::to_string_pretty;
use std::io::Result;
use exec::output_to_execresult;
use exec::input_json;
use exec::ExecReturn;

fn diff(_a:&Path, _b:&Path) -> Result<ExecReturn>  {
    let result_output= Command::new("diff")
            .arg("test.txt")
            .arg("/home/sean/test.txt")
            .stdout(Stdio::piped())
			.output();
    result_output.map( output_to_execresult )
}
pub fn difftask(src:&Path,dst:&Path) -> String {
    let r1 = json!({});
    let mut r2 = input_json(r1, src, dst);
    println!("input {}", r2.to_string());
    r2["result"] = match diff(src, dst) {
            Ok( t ) => json!(t) ,
            Err(e) =>  json!({"error!": format!("{}", e)})
    };
    to_string_pretty(&r2).unwrap()
}
}
pub fn difftest() {
   println!("Hello drt 0.1");
   let src = Path::new("test.txt");
   let dst = Path::new("/home/sean/test.txt");
   print!("{}", eval::difftask(src, dst));
}
