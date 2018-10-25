
//pub mod exec {

use std::process::Output;
use serde_json::Value;
use std::path::Path;
//use serde::ser::{Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecReturn {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DiffState {
    Equal,
    Create,
    Patch,
    FileNotFound,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DiffResult {
    pub state: DiffState,
    pub diffstr: Option<String>,
}
/*
impl Serialize for DiffResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("DiffResult", 2)?;
        s.serialize_field("state", &self.state)?;
        s.serialize_field("diff", &self.diff)?;
        s.end()
    }
}*/
//impl DiffResult {
//    fn greet(&self) {
//        println!("Hello, my name is {}", self.name);
//    }
//}
pub fn output_to_diffresult(output:Output) -> DiffResult {
    //let o:Vec<u8> = output.stdout;
    //let e:Vec<u8> = output.stderr;
    //let stdout: String = String::from_utf8(o).unwrap();
    //let stderr: String = String::from_utf8(e).unwrap();
    //let exit_code = output.status.code();
    DiffResult { state:DiffState::Patch, diffstr: Some(String::from_utf8(output.stdout).unwrap())} 
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
//}
