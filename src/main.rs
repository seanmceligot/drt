#[macro_use]
extern crate log;
extern crate simple_logger;
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
use drt::Mode;
use drt::template::{create_from, generate_recommended_file, Generated, TemplateFiles};
use std::io::Error;
use std::slice::Iter;
//use std::collections::hash_map::Iter as HashIter;
//use log::{info, trace, warn};
use log::Level;

#[derive(Debug)]
enum Sink<'r> {
    SyncBinaryFile(&'r TemplateFiles),
    SyncTextFile(&'r TemplateFiles),
    SyncNewFile(&'r TemplateFiles),
}

fn _report(sink: Sink) {
    match sink {
        Sink::SyncNewFile(files) => {
            debug!("new file: {:?}", files.dest);
        }
        Sink::SyncTextFile(files) => {
            debug!("report SyncTextFile: diff");
            diff(&files.gen, &files.dest);
        }
        Sink::SyncBinaryFile(files) => {
            debug!("binary {:?}", files.dest);
            if !files.dest.exists() {
                debug!("new binary: {}", files.dest.display());
            } else {
                debug!("report SyncBinaryFile: diff");
                diff(&files.gen, &files.dest);
            }
        }
    }
}
fn create_or_diff(mode: Mode, sink: Sink) -> Result<DiffStatus, Error> {
    match sink {
        Sink::SyncNewFile(files) => create_from(mode, &files.template, &files.gen, &files.dest),
        Sink::SyncTextFile(files) => {
            debug!("create_or_diff: diff");
            diff(&files.gen, &files.dest);
            create_from(mode, &files.template, &files.gen, &files.dest)
        }
        Sink::SyncBinaryFile(files) => {
            if !files.dest.exists() {
                create_from(mode, &files.template, &files.gen, &files.dest)
            } else {
                create_from(mode, &files.template, &files.gen, &files.dest)
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
    debug!("glob {}", pattern);

    match glob(pattern) {
        Err(e1) => debug!("{:?}", e1),
        Ok(paths) => {
            for globresult in paths {
                match globresult {
                    Err(glob_error) => debug!("{:?}", glob_error),
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
#[test]
fn test_parse_type() {
    match parse_type(&mut vec![String::from("t")].iter()) { Type::Template => {}, _ => panic!("expected Template"), }
    match parse_type(&mut vec![String::from("of")].iter()) { Type::OutputFile => {}, _ => panic!("expected Template"), }
    match parse_type(&mut vec![String::from("if")].iter()) { Type::InputFile => {}, _ => panic!("expected Template"), }
    match parse_type(&mut vec![String::from("v")].iter()) { Type::Variable => {}, _ => panic!("expected Template"), }
}

//|| input i:n=1 s:name=sean t:myconfig:project/my.config
//|| input of:f:mkdir,out1/my.config
fn parse_type(inputs: &mut Iter<String>) -> Type {
    let maybe_input = inputs.next();
    match maybe_input {
        None => Type::End,
        Some(input) => match input.as_str() {
            "t" => Type::Template,
            "of" => Type::OutputFile,
            "if" => Type::InputFile,
            "v" => Type::Variable,
            _ => { debug!("Unknown {}", input); Type::Unknown},
        },
    }
}
// v n=1 v y=hello of out1/my.config t project/my.config
fn process_template_file<'t>(
    mode: Mode,
    vars: &'t HashMap<&'_ str, &'_ str>,
    template_file: TemplateFiles,
) -> Result<DiffStatus, Error> {
    let gen = generate_recommended_file(vars, &template_file)?;
    let sink = generated_to_sink(gen)?;
    create_or_diff(mode, sink)
}

///
/// # Example
///
/// ```
/// let remain = v!["of", "out.file", "t", "in.template"]
/// let (current_action, next_action) = process_type(vars, task_vars, remain)?;
/// ```
fn process_type<'a,'b>(
    vars:      &'b mut HashMap<&'a str, &'a str>,
    task_vars: &'b mut HashMap<&'a str, &'a str>,
    remain:    &'a mut Iter   <'a, String>,
) -> Result<(Action, Action), Error> {
    let mut current_action: Action = Action::None;
    loop {
        let t = parse_type(remain); 
    	debug!("parse_type retuned {:#?}", t);
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
                let output_file_name = remain
                    .next()
                    .expect("of output_file_name: output_file_name required argument");
                task_vars.insert("output_file_name", output_file_name);
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
                debug!("split{:#?}", ss);
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
    mode: Mode,
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
            process_template_file(mode, &vars, files)?;
            Ok(())
        }
        Action::None => {
            Ok(())
        }
    }
}

#[test]
fn test_process_type() {
    let mut vars: HashMap<&str, &str> = HashMap::new();
    let remain = vec![
	String::from("v"), 
	String::from("test=1"), 
	String::from("v"), 
	String::from("user=myuser"), 
	String::from("v"), 
	String::from("base.dir=mybase"), 
	String::from("of"),
	String::from("out1/my.config"),
	String::from("t"), 
	String::from("project/my.config")];
    let mut task_vars: HashMap<&str, &str> = HashMap::new();
    let mut ri = remain.iter();
    let (action, next_type) = process_type(&mut vars, &mut task_vars, &mut ri).expect("process_type failed");
    debug!("action {:#?}", action);
    debug!("output_file_name {:#?}", vars.get("output_file_name"));
    match action { Action::Template => {}, _ => panic!("expected Template"), }
    do_action(Mode::Passive, &vars, &task_vars, Action::Template);
}
//#[test]
//fn test_do_action() {
//    let mut vars: HashMap<&str, &str> = HashMap::new();
//    let mut task_vars: HashMap<&str, &str> = HashMap::new();
//    task_vars.insert("template_file_name", "project/my.config");
//    task_vars.insert("output_file_name", "out.config");
//    do_action(Mode::Passive, &vars, &task_vars, Action::Template);
//}
fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    //opts.optopt("d", "dir", "project dir", "DIR");
    opts.optflag("D", "debug", "debug logging");
    opts.optflag("i", "interactive", "ask before overwrite");
    opts.optflag("a", "active", "overwrite without asking");
    //opts.optflag("p", "prop", "system properties");
    opts.optflag("h", "help", "print this help menu");
    let matches = opts.parse(&args[1..]).unwrap();
    //debug!("args {:#?}", matches);

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(());
    }
    if matches.opt_present("debug") {
        simple_logger::init_with_level(Level::Trace).unwrap();
    } else {
        simple_logger::init_with_level(Level::Warn).unwrap();
    }
    let mode = if matches.opt_present("interactive") {
	Mode::Interactive
    } else if matches.opt_present("active") {
	Mode::Active
    } else {
	Mode::Passive
    };
    let mut vars: HashMap<&str, &str> = HashMap::new();

    let remain = &mut matches.free.iter();
    //debug!("remain {:#?}", remain);

    {
        let mut task_vars: HashMap<&str, &str> = HashMap::new();
        let (action, next_type) = process_type(&mut vars, &mut task_vars, remain)?;
        //debug!("task_vars {:#?}", &task_vars);
        //debug!("vars {:#?}", &vars);
        debug!("action {:#?}", action);
        debug!("next_type {:#?}", next_type);
        //do_action(vars.as_mut(), &task_vars, action);
        do_action(mode, &vars, &task_vars, action)?;
    }
    Ok(())
}
