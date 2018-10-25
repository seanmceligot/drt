pub mod exec;
pub mod eval;

extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::path::Path;

use serde_json::ser::to_string_pretty;
pub fn difftest() {
   let src = Path::new("test.txt");
   let dst = Path::new("/home/sean/test.txt");
   let result = eval::difftask(src, dst);
   println!("result {}", to_string_pretty( &result) .unwrap());
}
