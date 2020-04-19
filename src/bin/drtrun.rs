extern crate getopts;
use getopts::Options;
use std::env;
use std::collections::HashMap;
extern crate drt;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

enum Type {
    Template,
    InputFile,
    OutputFile,
    Variable,
    Unknown
}
fn drain_to(mut input: String, ch: char) -> String {
    let offset = input.find(':').unwrap_or(input.len());
    let st: String = input.drain(..offset+1).collect();
    return st
}
//|| input i:n=1 s:name=sean t:myconfig:project/my.config
//|| input of:f:mkdir,out1/my.config
fn parse_type(input: String) -> (Type, String) {
    println!("input {}", input);
    let ss = input.splitn(2, ":").collect::<Vec<_>>();
    let (typename, remain) = (ss[0], ss[1]);
    println!("typename {}", typename);
    println!("remain {}", remain);
    let next = String::from(remain);
    match typename {
        "t" => (Type::Template, next),
        "of" => (Type::OutputFile, next),
        "if" => (Type::InputFile, next),
        "v" => (Type::Variable, next),
        _ => (Type::Unknown, next)
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    //opts.optopt("d", "dir", "project dir", "DIR");
    opts.optopt("l", "live", "system properties", "NAME");
    //opts.optflag("p", "prop", "system properties");
    opts.optflag("h", "help", "print this help menu");
    let matches = opts.parse(&args[1..]).unwrap();
    if matches.opt_present("h") {
        println!("h");
        print_usage(&program, opts);
        return;
    }
    let is_live = matches.opt_present("live");
    let mut vars: HashMap<String, String> = HashMap::new();
    for f in matches.free {
        let tok_type = parse_type(f);
        match tok_type {
            (Type::Template, remain) =>  {},
            (Type::OutputFile, remain) => {},
            (Type::InputFile, remain) => {},
            (Type::Variable, remain) => {
                let ss = remain.splitn(2, "=").collect::<Vec<_>>();
                let (key,val) = (ss[0], ss[1]);
                vars.insert(String::from(key), String::from(val));
            },
            (Type::Unknown, remain) => {}
            
        }
    }
    println!("is_live {}", is_live);
}
