extern crate getopts;
use getopts::Options;
use std::env;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    //opts.optopt("d", "dir", "project dir", "DIR");
    opts.optopt("", "live", "system properties", "NAME");
    //opts.optflag("p", "prop", "system properties");
    opts.optflag("", "help", "print this help menu");
    let matches = opts.parse(&args[1..]).unwrap();
    if matches.opt_present("h") {
        println!("h");
        print_usage(&program, opts);
        return;
    }
    let is_live = matches.opt_present("live");
    println!("is_live {}", is_live);

}
