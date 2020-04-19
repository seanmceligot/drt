extern crate regex;
extern crate getopts;
extern crate glob;

use getopts::Options;
use self::glob::glob;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::str;
mod drt;
use drt::properties;
use drt::template::{Generated, TemplateFiles, create_from, generate_recommended};
use drt::diff::{diff};

enum Sink<'r> {
    SyncBinaryFile(&'r TemplateFiles),
    SyncTextFile(&'r TemplateFiles),
    SyncNewFile(&'r TemplateFiles),
}

fn report(sink: Sink) {
    match sink {
        Sink::SyncNewFile(files) => {
            println!("new file: {}", files.dest);
        }
        Sink::SyncTextFile(files) => {
            let genp = Path::new(files.gen.as_str());
            let destp = Path::new(files.dest.as_str());
            diff(genp, destp);
        }
        Sink::SyncBinaryFile(files) => {
            println!("binary {}", files.dest);
            let genp = Path::new(files.gen.as_str());
            let destp = Path::new(files.dest.as_str());
            if !destp.exists() {
                println!("new binary: {}", destp.display());
            } else {
                diff(genp, destp);
            }
        }
    }
}
fn create_or_diff(sink: Sink) {
    match sink {
        Sink::SyncNewFile(files) => {
            let genp = Path::new(files.gen.as_str());
            let destp = Path::new(files.dest.as_str());
            let template = Path::new(files.template.as_str());
            create_from(&template, &genp, destp);
        }
        Sink::SyncTextFile(files) => {
            let genp = Path::new(files.gen.as_str());
            let destp = Path::new(files.dest.as_str());
            let template = Path::new(files.template.as_str());
            diff(genp, destp);
            create_from(&template, genp, destp);
        }
        Sink::SyncBinaryFile(files) => {
            let genp = Path::new(files.gen.as_str());
            let destp = Path::new(files.dest.as_str());
            let template = Path::new(files.template.as_str());
            println!("binary {}", destp.display());
            if !destp.exists() {
                create_from(&template, genp, destp);
            }
        }
    }
}
//fn get_x<'r>(p: &'r Point) -> &'r f64 { &p.x }
fn generated_to_sink<'r>(generated: Result<Generated<'r>, String>) -> Result<Sink<'r>, String> {
    match generated {
        Err(why) => return Err(why),
        Ok(g) => match g {
            Generated::RText(files) => {
                let exists = Path::new(files.dest.as_str()).exists();
                if exists {
                    return Ok(Sink::SyncTextFile(files));
                } else {
                    return Ok(Sink::SyncNewFile(files));
                }
            }
            Generated::RBinary(files) => {
                let exists = Path::new(files.dest.as_str()).exists();
                if exists {
                    return Ok(Sink::SyncBinaryFile(files));
                } else {
                    return Ok(Sink::SyncNewFile(files));
                }
            }
        },
    }
}
fn process_dir(pattern: &str) -> Vec<PathBuf> {
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
fn sync(map: HashMap<String, String>, dirs: Vec<String>) {
    for dir in dirs.iter() {
        println!("dir {}", dir);
        let paths = process_dir(dir);

        //let files_it = paths.iter().map(|path:&PathBuf| TemplateFiles::new(path, dest_dir));
        for path in paths {
            let dest_dir = "out/".to_string();
            //let files = TemplateFiles::new(path.as_path(), dest_dir);
            let files = TemplateFiles::new(path.as_path(), dest_dir);
            //for files in files_it {
            //files: core::iter::Map<core::slice::Iter<'_, std::path::PathBuf>
            let gen = generate_recommended(&map, &files);
            let result_sink = generated_to_sink(gen);
            //  let gens = files_it.map(|&files| generate_recommended(&map, files));
            //let sinks = gens.map(|gen| generated_to_sink(gen));
            //for result_sink in sinks {
            match result_sink {
                Err(msg) => println!("err: {}", msg),
                Ok(sink) => {
                    //report(sink);
                    create_or_diff(sink);
                }
            }
        }
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    //opts.optopt("d", "dir", "project dir", "DIR");
    opts.optopt("p", "", "system properties", "NAME");
    //opts.optflag("p", "prop", "system properties");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        println!("h");
        print_usage(&program, opts);
        return;
    }
    let prop: String = match matches.opt_str("p") {
        Some(p) => p,
        None => "".to_string(), //println!("p");}
                                //print_usage(&program, opts);
                                //return; }
    };
    println!("prop {}", prop);
    let dirs: Vec<String> = if !matches.free.is_empty() {
        matches.free
    } else {
        println!("matches.free {:?}", matches.free.as_slice());
        print_usage(&program, opts);
        return;
    };

    let mut map: HashMap<String, String> = HashMap::new();
    properties::properties(&mut map, prop).expect("error loading properties");
    sync(map, dirs);
}
