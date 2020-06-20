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
use std::slice::Iter;
use log::Level;
use drt::userinput::ask;
use std::process::Command;
use std::io::{self, Write};

static INFILE: &str = "if";
static OUTFILE: &str = "of";
static CMD: &str = "x";

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
    //InputFile,
    //OutputFile,
    Variable,
    Unknown
}
fn _drain_to(mut input: String, ch: char) -> String {
    let offset = input.find(ch).unwrap_or(input.len());
    let st: String = input.drain(..offset + 1).collect();
    return st;
}
#[test]
fn test_parse_type() {
    match parse_type(&String::from("t")) { Type::Template => {}, _ => panic!("expected Template"), }
    match parse_type(&String::from("x")) { Type::Execute => {}, _ => panic!("expected Execute"), }
    match parse_type(&String::from("v")) { Type::Variable => {}, _ => panic!("expected Template"), }
}

fn parse_type(input: &String) -> Type {
    match input.as_str() {
        "t" => Type::Template,
        "x" => Type::Execute,
        "v" => Type::Variable,
        _ => { debug!("Unknown {}", input); Type::Unknown},
    }
}
fn process_template_file<'t>(
    mode: Mode,
    vars: &'t HashMap<&'_ str, &'_ str>,
    template: & SrcFile,
    dest: & DestFile
) -> Result<DiffStatus, Error> {
    let gen = generate_recommended_file(vars, template)?;
    create_or_diff(mode, template, dest, &gen)
}
#[test]
fn test_execute_active() {
    execute_active("/bin/true");
    execute_active("/bin/false");
    execute_active("echo echo_ping");
}
fn execute_active(cmd: &str) {
	let parts: Vec<&str> = cmd.split(' ').collect();
	let output = Command::new(parts[0])
		.args(&parts[1..])
		.output()
		.expect("cmd failed");
	println!("LIVE: run: {}", cmd);
	io::stdout().write_all(&output.stdout);
	println!("status code: {}", output.status.code().unwrap());
}
fn execute_interactive(cmd: &str) {
	match ask(&format!("run (y/n): {}", cmd)) {
		'n' => {
			println!("SKIPPED: run: {}", cmd);
		},
		'y' => {
			execute_active(cmd)
		}
		_ => { 
			execute(Mode::Interactive, cmd);
		}
	}
}
fn execute<'g>(
    mode: Mode,
    cmd: &str
) -> Result<(), Error> {
    match mode {
        Mode::Interactive => { execute_interactive(cmd); },
        Mode::Passive => println!("WOULD: run: {}", cmd),
        Mode::Active=> { execute_active(cmd) }
    }
    Ok(())
}

fn do_action<'g>(
    mode: Mode,
    vars: &'g HashMap<&'g str, &'g str>,
    special_vars: &'g HashMap<&'g str, &'g str>,
    action: Action,
) -> Result<(), Error> {
    match action {
        Action::Template => {
            let template_file_name = special_vars.get(INFILE).expect("template_file_name required");
            let template_file = SrcFile::new(PathBuf::from(template_file_name));
            let output_file_name = special_vars.get(OUTFILE).expect("output_file_name required");
            let output_file = DestFile::new(PathBuf::from(output_file_name));

            process_template_file(mode, &vars, &template_file, &output_file)?;
            Ok(())
        },
        Action::Execute => {
            let cmd = special_vars.get(CMD).expect("Execute command required");
            execute(mode, cmd)?;
            Ok(())
        },
        Action::None => {
            Ok(())
        }
    }
}

#[test]
fn test_do_action() {
    let mut vars: HashMap<&str, &str> = HashMap::new();
    let mut special_vars: HashMap<&str, &str> = HashMap::new();
    vars.insert("value", "unit_test");
    special_vars.insert(INFILE, "template/test.config");
    special_vars.insert(OUTFILE, "template/out.config");
    match do_action(Mode::Passive, &vars, &special_vars, Action::Template) {
        Ok(_) =>  {}
        Err(e) => panic!("{}", e)
    }
}
fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("D", "debug", "debug logging");
    opts.optflag("i", "interactive", "ask before overwrite");
    opts.optflag("a", "active", "overwrite without asking");
    opts.optflag("h", "help", "print this help menu");
    let matches = opts.parse(&args[1..]).unwrap();

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
        let mut input_list:Iter<String>= matches.free.iter(); 
        while let Some(input) =  input_list.next() {
            let t:Type = parse_type(input);
        	let mut special_vars: HashMap<&str, &str> = HashMap::new();
			let mut cmd = String::new();
            let action = match t {
                Type::Template => {
                    special_vars.insert(INFILE, input_list.next().expect("expected template: tp template output"));
                    special_vars.insert(OUTFILE, input_list.next().expect("expected output: tp template output"));
                    Action::Template
                },
                Type::Variable=> {
                    let key = input_list.next().expect("expected key: v key value");
                    let val = input_list.next().expect("expected value: v key value");
                    vars.insert(key,val);
                    Action::None
                },
                Type::Execute => {
        			while let Some(input) =  input_list.next() {
						if cmd.is_empty() {
							cmd.push_str(&input.to_string());
						} else {
							cmd.push_str(" ");
							cmd.push_str(&input.to_string());
						}
					}
					let cmd_str: &str = cmd.as_str();
                    special_vars.insert(CMD, cmd_str);
                    Action::Execute
                },
                Type::Unknown => {
                    panic!("Unknown type: {}", input);
                }
            };
            //debug!("special_vars {:#?}", &special_vars);
            //debug!("vars {:#?}", &vars);
            debug!("action {:#?}", action);
            match do_action(mode, &vars, &special_vars, action) {
                Ok(_) =>  {}
                Err(e) => println!("{}", e)
            }
        }
    }
    Ok(())
}
