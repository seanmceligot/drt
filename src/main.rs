#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate getopts;
extern crate glob;
extern crate regex;
extern crate tempfile;

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
use drt::SrcFile;
use drt::DestFile;
use drt::GenFile;
use drt::template::{create_from, generate_recommended_file};
use std::io::Error;
use std::io::ErrorKind;
use std::slice::Iter;
//use std::collections::hash_map::Iter as HashIter;
//use log::{info, trace, warn};
use log::Level;

fn create_or_diff
    (mode: Mode, 
    template: & SrcFile,
    dest: & DestFile,
    gen: & GenFile
) -> Result<DiffStatus, Error> {
    if dest.exists() {
            debug!("create_or_diff: diff");
            diff(gen.path(), dest.path());
            create_from(mode, template, gen, dest)
    } else {
        create_from(mode, template, gen, dest)
    }
}
//fn get_x<'r>(p: &'r Point) -> &'r f64 { &p.x }
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
    Execute,
    None,
}
#[derive(Debug)]
enum Type {
    Template,
    Execute,
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
    match parse_type(&String::from("t")) { Type::Template => {}, _ => panic!("expected Template"), }
    match parse_type(&String::from("of")) { Type::OutputFile => {}, _ => panic!("expected Template"), }
    match parse_type(&String::from("if")) { Type::InputFile => {}, _ => panic!("expected Template"), }
    match parse_type(&String::from("v")) { Type::Variable => {}, _ => panic!("expected Template"), }
}

//|| input i:n=1 s:name=sean t:myconfig:project/my.config
//|| input of:f:mkdir,out1/my.config
fn parse_type(input: &String) -> Type {
    match input.as_str() {
        "t" => Type::Template,
        "of" => Type::OutputFile,
        "if" => Type::InputFile,
        "v" => Type::Variable,
        _ => { debug!("Unknown {}", input); Type::Unknown},
    }
}
// v n=1 v y=hello of out1/my.config t project/my.config
fn process_template_file<'t>(
    mode: Mode,
    vars: &'t HashMap<&'_ str, &'_ str>,
    template: & SrcFile,
    dest: & DestFile
) -> Result<DiffStatus, Error> {
    let gen = generate_recommended_file(vars, template)?;
    create_or_diff(mode, template, dest, &gen)
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
    t: Type,
    next: Option<&'a String>) 
    -> Result<(
        Action,
        &'b mut HashMap<&'a str, &'a str>,
        &'b mut HashMap<&'a str, &'a str> )
        , Error> {
    match t {
        Type::Template => {
                let template_file_name = next.expect("t template_file_name: template_file_name required argument");
                task_vars.insert("template_file_name", template_file_name);
                return Ok((Action::Template, vars, task_vars));
        },
        Type::Execute => {
            let cmd= next.expect("of required cmd");
            task_vars.insert("cmd", cmd);
            return Ok((Action::Execute, vars, task_vars));
        }
        Type::OutputFile => {
            let output_file_name = next.expect("of required (output file name)");
            task_vars.insert("output_file_name", output_file_name);
            return Ok((Action::None, vars, task_vars));
        }
        Type::InputFile => {
            let in_file_name = next.expect("of in_file_name: in_file_name required argument");
            task_vars.insert("in_file_name", in_file_name);
            return Ok((Action::None, vars, task_vars));
        }
        Type::Variable => {
            let ss = next.expect("expected key=val")
                .splitn(2, "=")
                .collect::<Vec<_>>();
            debug!("split{:#?}", ss);
            assert!(ss.len() == 2, "expected key=val after v");
            let (key, val) = (ss[0], ss[1]);
            vars.insert(key, val);
            return Ok((Action::None, vars, task_vars));
        }
        Type::Unknown => {
            return Err(Error::new(ErrorKind::Other, "Type::Unknown"));
        }
        Type::End => {
            return Ok((Action::None, vars, task_vars));
        }
    }
}
//fn execute<'g>(
//    mode: Mode,
//    vars: &'g HashMap<&'g str, &'g str>,
//    task_vars: &'g HashMap<&'g str, &'g str>,
//    remain: &'g Iter<&'g str>,
//) -> Result<(), Error> {
//    Ok(())
//}
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
            let template_file = SrcFile::new(PathBuf::from(template_file_name));
            let output_file_name = task_vars
                .get("output_file_name")
                .expect("output_file_name required");
            let output_file = DestFile::new(PathBuf::from(output_file_name));

            process_template_file(mode, &vars, &template_file, &output_file)?;
            Ok(())
        },
        Action::Execute => {
            //execute(mode, &vars, &task_vars, remain)?;
            Ok(())
        },
        Action::None => {
            Ok(())
        }
    }
}

//#[test]
//fn test_exec() -> Result<(), Error> {
//
//    let mut vars: HashMap<&str, &str> = HashMap::new();
//    let _remain = vec![
//	String::from("x"), 
//	String::from("ls"),
//	String::from("-l"),
//	String::from("Makefile")];
//    let remain = vec![
//	"x", 
//	"ls",
//	"-l",
//	"Makefile"];
//    let mut task_vars: HashMap<&str, &str> = HashMap::new();
//    let mut ri = remain.iter();
//    let (action, _next_type) = process_type(&mut vars, &mut task_vars, &mut ri).expect("process_type failed");
//    let output_file_name: &str = task_vars.get("output_file_name").unwrap();
//    assert_eq!( output_file_name, "out1/my.config");
//    let template_file_name: &str = task_vars.get("template_file_name").unwrap();
//    assert_eq!( template_file_name, "project/my.config");
//    let var_test: &str = vars.get("test").unwrap();
//    assert_eq!( var_test, "1");
//    let var_user: &str = vars.get("user").unwrap();
//    assert_eq!( var_user, "myuser");
//    let var_base_dir: &str = vars.get("base.dir").unwrap();
//    assert_eq!( var_base_dir, "mybase");
//    match action { Action::Template => {}, _ => panic!("expected Template"), }
//    do_action(Mode::Passive, &vars, &task_vars, &mut ri, Action::Template)
//}
////#[test]
////fn test_process_type() -> Result<(), Error> {
////    let mut vars: HashMap<&str, &str> = HashMap::new();
////    let remain = vec![
////	&"v", 
////	&"test=1", 
////	&"v", 
////	&"user=myuser", 
////	&"v", 
////	&"base.dir=mybase", 
////	&"of",
////	&"out1/my.config",
////	&"t", 
////	&"project/my.config"];
////    let remain_string = vec![
////	String::from("v"), 
////	String::from("test=1"), 
////	String::from("v"), 
////	String::from("user=myuser"), 
////	String::from("v"), 
////	String::from("base.dir=mybase"), 
////	String::from("of"),
////	String::from("out1/my.config"),
////	String::from("t"), 
////	String::from("project/my.config")];
////    let mut task_vars: HashMap<&str, &str> = HashMap::new();
////    let ri = remain.iter();
////    let mut ri_string = remain_string.iter();
////    let (action, _next_type) = process_type(&mut vars, &mut task_vars, ri).expect("process_type failed");
////    let output_file_name: &str = task_vars.get("output_file_name").unwrap();
////    assert_eq!( output_file_name, "out1/my.config");
////    let template_file_name: &str = task_vars.get("template_file_name").unwrap();
////    assert_eq!( template_file_name, "project/my.config");
////    let var_test: &str = vars.get("test").unwrap();
////    assert_eq!( var_test, "1");
////    let var_user: &str = vars.get("user").unwrap();
////    assert_eq!( var_user, "myuser");
////    let var_base_dir: &str = vars.get("base.dir").unwrap();
////    assert_eq!( var_base_dir, "mybase");
////    match action { Action::Template => {}, _ => panic!("expected Template"), }
////    do_action(Mode::Passive, &vars, &task_vars, &mut ri, Action::Template)
////}
#[test]
fn test_do_action() {
    let mut vars: HashMap<&str, &str> = HashMap::new();
    let mut task_vars: HashMap<&str, &str> = HashMap::new();
    vars.insert("base.dir", "/foo/bar");
    vars.insert("test", "I_AM_TEST");
    vars.insert("user", "I_AM_USER");
    task_vars.insert("template_file_name", "project/my.config");
    task_vars.insert("output_file_name", "out.config");
    match do_action(Mode::Passive, &vars, &task_vars, Action::Template) {
        Ok(_) =>  {}
        Err(e) => panic!("{}", e)
    }
}
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
    {
        let mut task_vars: HashMap<&str, &str> = HashMap::new();
        let mut input_list:Iter<String>= matches.free.iter(); 
        while let Some(input) =  input_list.next() {
            let t:Type = parse_type(input);
            let next:Option<&String> = input_list.next();
            let (action, vars, task_vars) = process_type(&mut vars, &mut task_vars, t, next)?;
            //debug!("task_vars {:#?}", &task_vars);
            //debug!("vars {:#?}", &vars);
            debug!("action {:#?}", action);
            //do_action(vars.as_mut(), &task_vars, action);
            match do_action(mode, &vars, &task_vars, action) {
                Ok(_) =>  {}
                Err(e) => println!("{}", e)
            }
        }
    }
    Ok(())
}
