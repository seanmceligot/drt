extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::path::Path;
extern crate drt;

fn main() {
    let src = Path::new("test.txt");
    let dst = Path::new("/home/sean/test.txt");
    print!("{}", drt::eval::difftask(src, dst));
}
