extern crate getopts;
extern crate glob;
extern crate regex;

use self::glob::glob;
use getopts::Options;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::str;
mod drt;
use drt::diff::diff;
use drt::diff::DiffStatus;
use drt::template::{create_from, generate_recommended_file, Generated, TemplateFiles};
use std::io::Error;
use std::slice::Iter;

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
fn create_or_diff(sink: Sink) -> Result<DiffStatus, Error> {
    match sink {
        Sink::SyncNewFile(files) => create_from(&files.template, &files.gen, &files.dest),
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
fn generated_to_sink<'t>(generated: Generated<'t>) -> Result<Sink<'t>, Error> {
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
#[derive(Debug)]
enum Action {
    Template,
    None,
}
#[derive(Debug)]
enum Type {
    Template,
    InputFile,
    OutputFile,
    Variable,
    Unknown,
    End,
}
fn _drain_to(mut input: String, ch: char) -> String {
    let offset = input.find(ch).unwrap_or(input.len());
    let st: String = input.drain(..offset + 1).collect();
    return st;
}

//|| input i:n=1 s:name=sean t:myconfig:project/my.config
//|| input of:f:mkdir,out1/my.config
fn parse_type(inputs: &mut Iter<String>) -> Type {
    let maybe_input = inputs.next();
    println!("input {:?}", maybe_input);
    match maybe_input {
        None => Type::End,
        Some(input) => match input.as_str() {
            "t" => Type::Template,
            "of" => Type::OutputFile,
            "if" => Type::InputFile,
            "v" => Type::Variable,
            _ => Type::Unknown,
        },
    }
}
// v n=1 v y=hello of out1/my.config t project/my.config
fn process_template_file<'t>(
    vars: &'t HashMap<&'_ str, &'_ str>,
    template_file: TemplateFiles,
) -> Result<DiffStatus, Error> {
    let gen = generate_recommended_file(vars, &template_file)?;
    let sink = generated_to_sink(gen)?;
    create_or_diff(sink)
}

fn use_mut<'t>(
	vars: &'t mut HashMap<&'_ str, &'_ str>
) {
  vars.insert("a", "one");
}

///
/// # Example
///
/// ```
/// let remain = v!["of", "out.file", "t", "in.template"]
/// let (current_action, next_action) = process_type(vars, task_vars, remain)?;
/// ```
fn process_type<'a>(
    vars:      &'a mut HashMap<&'a str, &'a str>,
    task_vars: &'a mut HashMap<&'a str, &'a str>,
    remain:    &'a mut Iter   <'a, String>,
) -> Result<(Action, Action), Error> {
    let mut current_action: Action = Action::None;
    loop {
        let t = parse_type(remain); 
    	println!("parse_type retuned {:#?}", t);
        match t {
            Type::Template => match current_action {
                Action::None => {
                    let template_file_name = remain
                        .next()
                        .expect("t template_file_name: template_file_name required argument");
                    task_vars.insert("template_file_name", template_file_name);
                    current_action = Action::Template;
                }
                _ => {
                    // we reached the second action. return the current_action and process this later
                    return Ok((current_action, Action::Template));
                }
            },
            Type::OutputFile => {
                let out_file_name = remain
                    .next()
                    .expect("of out_file_name: out_file_name required argument");
                task_vars.insert("out_file_name", out_file_name);
            }
            Type::InputFile => {
                let in_file_name = remain
                    .next()
                    .expect("of in_file_name: in_file_name required argument");
                task_vars.insert("in_file_name", in_file_name);
            }
            Type::Variable => {
                let ss = remain
                    .next()
                    .expect("expected key=val")
                    .splitn(2, "=")
                    .collect::<Vec<_>>();
                println!("split{:#?}", ss);
                assert!(ss.len() == 2, "expected key=val after v");
                let (key, val) = (ss[0], ss[1]);
                vars.insert(key, val);
            }
            Type::Unknown => {
                return Ok((current_action, Action::None));
            }
            Type::End => {
                return Ok((current_action, Action::None));
            }
        }
    }
}
fn do_action<'g>(
    vars: &'g HashMap<&'g str, &'g str>,
    task_vars: &'g HashMap<&'g str, &'g str>,
    action: Action,
) -> Result<(), Error> {
    match action {
        Action::Template => {
            let template_file_name = task_vars
                .get("template_file_name")
                .expect("template_file_name required");
            let template_file = PathBuf::from(template_file_name);
            let output_file_name = task_vars
                .get("output_file_name")
                .expect("output_file_name required");
            let output_file = PathBuf::from(output_file_name);

            let files = TemplateFiles::new(template_file, output_file).unwrap();
            process_template_file(&vars, files)?;
            Ok(())
        }
        Action::None => {
            Ok(())
        }
    }
}
fn use_immut<'t>(vars: &'t HashMap<&str, &str>) {
    println!("vars {:#?}", vars.get("a"));
}


#[test]
fn mut_test() {
    let mut vars: HashMap<&str, &str> = HashMap::new();
    use_mut(&mut vars);
    use_immut(&vars);

}
#[test]
fn test_process_type() {
    let mut vars: HashMap<&str, &str> = HashMap::new();
    let remain = vec![
	String::from("v"), 
	String::from("n=1"), 
	String::from("v"), 
	String::from("y=hello"), 
	String::from("of"),
	String::from("out1/my.config"),
	String::from("t"), 
	String::from("project/my.config")];
    let mut task_vars: HashMap<&str, &str> = HashMap::new();
    let x = process_type(&mut vars, &mut task_vars, &mut remain.iter());
    let (action, next_type) = x.expect("process_type failed");
    println!("action {:#?}", action);
    match action {
    	Action::Template => {},
    	_ => panic!("expected Template"),
    }
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
        return Ok(());
    }
    let is_live = matches.opt_present("live");
    // begin 'g
    let mut vars: HashMap<&str, &str> = HashMap::new();

    let remain = &mut matches.free.iter();

    // begin 't
    {
        let mut task_vars: HashMap<&str, &str> = HashMap::new();
        let (action, next_type) = process_type(&mut vars, &mut task_vars, remain)?;
        //println!("task_vars {:#?}", &task_vars);
        //println!("vars {:#?}", &vars);
        println!("action {:#?}", action);
        println!("next_type {:#?}", next_type);
        //do_action(&vars, &task_vars, action);
    }
    //println!("task_vars {:#?}", task_vars);
    println!("is_live {}", is_live);
    Ok(())
}
