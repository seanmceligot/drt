

//pub mod eval {
use std::process::Command;
use std::process::Stdio;
use std::path::Path;
use serde_json::Value;
//use std::io::{Result, Error, ErrorKind};
//use std::io::{self};
use exec::output_to_diffresult;
use exec::input_json;
//use exec::ExecReturn;
use exec::DiffState;
use exec::DiffResult;
use serde_json::ser::to_string_pretty;

fn diff(_a:&Path, _b:&Path) -> Result<DiffResult, String>  {
    if !_a.exists() {
        Err( format!("file not found: {}", _a.display()))
    } else if !_b.exists() {
        Ok( DiffResult { state: DiffState::Create, diffstr:None} ) 
    } else {
        match Command::new("diff")
            .arg(_a)
            .arg(_b)
            .stdout(Stdio::piped())
            .output() {
                Ok( t ) => Ok(output_to_diffresult(t)) ,
                Err(e) =>  Err(format!("{:?}", e.kind()))
            }
       }
    }

// global
pub fn difftask(src:&Path,dst:&Path) -> Value {
    let r1 = json!({});
    let mut result = input_json(r1, src, dst);
    println!("input {}", to_string_pretty( &result) .unwrap());
    result["result"] = match diff(src, dst) {
            Err(e) =>  json!({"error!": format!("{}", e)}),
            Ok( t ) => json!(t) 
    };
    result
}

//}
