extern crate serde;

use std::path::Path;
extern crate drt;

fn main() {
    let src = Path::new("test.txt");
    let dst = Path::new("/home/sean/test.txt");
    print!("{}", drt::eval::difftask(src, dst));
}
