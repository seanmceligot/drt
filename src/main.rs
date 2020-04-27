#[allow(dead_code)]
extern crate regex;
extern crate getopts;
extern crate glob;

use getopts::Options;
use self::glob::glob;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::str;
mod drt;
use drt::template::{Generated, TemplateFiles, create_from, generate_recommended_file};
use drt::diff::{diff};
use std::io::Error;
use drt::diff::DiffStatus;

enum Sink<'r> {
    SyncBinaryFile(&'r TemplateFiles),
    SyncTextFile(&'r TemplateFiles),
    SyncNewFile(&'r TemplateFiles),
}

fn _report(sink: Sink) {
    match sink {
        Sink::SyncNewFile(files) => {
            println!("new file: {:?}", files.dest);
        }
        Sink::SyncTextFile(files) => {
            diff(&files.gen, &files.dest);
        }
        Sink::SyncBinaryFile(files) => {
            println!("binary {:?}", files.dest);
            if !files.dest.exists() {
                println!("new binary: {}", files.dest.display());
            } else {
                diff(&files.gen, &files.dest);
            }
        }
    }
}
fn create_or_diff(sink: Sink) -> Result< DiffStatus, Error>  {
    match sink {
        Sink::SyncNewFile(files) => {
            create_from(&files.template, &files.gen, &files.dest)
        }
        Sink::SyncTextFile(files) => {
            diff(&files.gen, &files.dest);
            create_from(&files.template, &files.gen, &files.dest)
        }
        Sink::SyncBinaryFile(files) => {
            if !files.dest.exists() {
                create_from(&files.template, &files.gen, &files.dest)
            } else {
                // TODO ask
                create_from(&files.template, &files.gen, &files.dest)
            }
        }
    }
}
//fn get_x<'r>(p: &'r Point) -> &'r f64 { &p.x }
fn generated_to_sink<'f>(generated: Generated<'f>) -> Result<Sink<'f>, Error> {
    match generated {
        Generated::RText(files) => {
            let exists = files.dest.exists();
            if exists {
                return Ok(Sink::SyncTextFile(files));
            } else {
                return Ok(Sink::SyncNewFile(files));
            }
        }
        Generated::_RBinary(files) => {
            let exists = files.dest.exists();
            if exists {
                return Ok(Sink::SyncBinaryFile(files));
            } else {
                return Ok(Sink::SyncNewFile(files));
            }
        }
    }
}
fn _process_dir(pattern: &str) -> Vec<PathBuf> {
    let mut vec = Vec::new();
    println!("glob {}", pattern);

    match glob(pattern) {
        Err(e1) => println!("{:?}", e1),
        Ok(paths) => {
            for globresult in paths {
                match globresult {
                    Err(glob_error) => println!("{:?}", glob_error),
                    Ok(pathbuf) => vec.push(pathbuf),
                }
            }
        }
    }
    vec
}
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
fn _drain_to(mut input: String, ch: char) -> String {
    let offset = input.find(ch).unwrap_or(input.len());
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
// v n=1 v y=hello of out1/my.config t project/my.config
fn process_template_file<'f>(map: &'f HashMap<String,String>, template_file: TemplateFiles) -> Result<DiffStatus, Error> {
    let gen = generate_recommended_file(&map, &template_file)?;
    let sink = generated_to_sink(gen)?;
    create_or_diff(sink)
}
fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    //opts.optopt("d", "dir", "project dir", "DIR");
    opts.optopt("l", "live", "system properties", "NAME");
    //opts.optflag("p", "prop", "system properties");
    opts.optflag("h", "help", "print this help menu");
    let matches = opts.parse(&args[1..]).unwrap();
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(())
    }
    let is_live = matches.opt_present("live");
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut task_vars: HashMap<String, String> = HashMap::new();
    for f in matches.free {
        match parse_type(f) {
            (Type::Template, remain) =>  {
                let template_file = PathBuf::from(remain);
                let dest_file = PathBuf::from(task_vars.get("out_file").unwrap());
                let files = TemplateFiles::new(template_file, dest_file).unwrap();
                process_template_file(&vars, files)?;
            },
            (Type::OutputFile, remain) => {
                task_vars.insert(String::from("out_file"), remain);
            },
            (Type::InputFile, remain) => {
                task_vars.insert(String::from("in_file"), remain);
            },
            (Type::Variable, remain) => {
                let ss = remain.splitn(2, "=").collect::<Vec<_>>();
                let (key,val) = (ss[0], ss[1]);
                vars.insert(String::from(key), String::from(val));
            },
            (Type::Unknown, _remain) => {
            }
            
        }
    }
    println!("vars {:#?}", vars);
    println!("is_live {}", is_live);
    Ok(())
}
