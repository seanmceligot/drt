extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod drt;
use drt::ExecReturn;
use std::path::Path;


fn main() {
    let src = Path::new("test.txt");
    let dst = Path::new("/home/sean/test.txt");
    print!("{}", drt::difftask(src, dst));
}
