use std::convert::AsRef;

//use std::fs::copy;
use std::str;
use std::fs::File;
use std::path::PathBuf;
use std::path::Path;
use std::io::BufReader;
use std::io::Error;
use std::io::prelude::*;
use std::collections::HashMap;
use std::io::stdin;
use std::process::Command;

use std::io::{self, Write};

extern crate regex;
use regex::Regex;

extern crate getopts;
use getopts::Options;
use std::env;

extern crate glob;
use self::glob::glob;
//use self::glob::Paths;
//use self::glob::PatternError;

use std::fs::OpenOptions;

//extern crate libc;

pub struct Files{
    template: String,
    gen: String ,
    dest: String
}

impl Files {
    fn new(ppath: &Path, dest_dir:String) -> Files {
          let gen =  Path::new("/tmp").join(Path::new(ppath.file_name().unwrap()));
          let dest = Path::new(&*dest_dir).join(*&ppath);
        Files { 
        template: ppath.to_str().unwrap().to_string(), 
        gen: gen.to_str().unwrap().to_string(), 
        dest:dest.to_str().unwrap().to_string() }
    }
}

//fn properties<'a>(map: mut HashMap<String,String>,  property_file:String) -> Result<(), Error> {
//#[derive(Debug)]
fn properties(map: &mut HashMap<String,String>,  property_file:String) -> Result<(), Error> {
  //let system =  "system.config";
  let path = Path::new(property_file.as_str());
  println!("open {:?}", path);
  let file = File::open(&path)?;
  let re = Regex::new(r"^(?P<k>[[:alnum:]\._]*)=(?P<v>.*)").unwrap();

  let reader = BufReader::new( file );
  for line in reader.lines() {
    for cap in re.captures_iter(line.unwrap().as_str()) {
        map.insert(String::from(cap.name("k").unwrap().as_str()), String::from(cap.name("v").unwrap().as_str()));
    }
  }
  println!("map {:?}", map);
  Ok(())
}
enum DiffStatus {
   NoChanges, 
   NewFile, 
   Changed,
   Failed
 }
enum Generated<'r> {
   RText(&'r Files),
   RBinary(&'r Files)
}
enum Sink<'r> {
    SyncBinaryFile(&'r Files),
    SyncTextFile(&'r Files),
    SyncNewFile(&'r Files)
}
fn generate_recommended<'r>(map: &HashMap<String,String>, files: &'r Files) -> Result<Generated<'r>,String> {
  let path = Path::new(files.template.as_str());
  println!("open template {:?}", path);
  let infile: Result<File,Error> = File::open(&path);
  //let re = Regex::new(r"^(?P<k>[[:alnum:]\._]*)=(?P<v>.*)").unwrap();
  let re = Regex::new(r"@@(?P<t>[fns]):(?P<k>[A-Za-z0-9\.-]*)@@").unwrap();
  let reader = BufReader::new( infile.unwrap() );
  let mut tmpfile = OpenOptions::new().read(false).write(true).create(true).truncate(true).open(Path::new(files.gen.as_str())).unwrap();
  for line in reader.lines() {
    let l = line.unwrap();
    match re.captures(l.as_str()) {
        Some(cap) => {
            let t = cap.name("t").unwrap().as_str();
            let k = cap.name("k").unwrap().as_str();
            let v = map.get(k);
            println!("t {:?}", t);
            println!("k {:?}", k);
            println!("v {:?}", v);
            println!("l {:?}", l);

            if v.is_some() {
                // change match to map[k[
                writeln!(tmpfile, "{}", re.replace(l.as_str(), v.unwrap().as_str())).expect("Cannot Write to tmp file")
            } else {
                // found varable but no key in map so leave line alone
                println!("warn: NOT FOUND {}", k);
                writeln!(tmpfile, "{}", l).expect("Cannot write to tmp file");
            };
        },
        None => {
            // no variable in line
            writeln!(tmpfile, "{}", l).expect("Cannot write to tmp file");
        }
    }
  }
  return Ok(Generated::RText(files));
}
fn merge(template:&Path, path:&Path, path2:&Path) -> ()  {
    let status = Command::new("vim")
        .arg("-d")
        .arg(template.as_os_str())
        .arg(path.as_os_str())
        .arg(path2.as_os_str())
        .status()
        .expect("failed to execute process");

    println!("with: {}", status);
    assert!(status.success());
}
fn diff(path:&Path, path2:&Path) -> DiffStatus  {
    println!("diff {} {}", path.display(), path2.display());
    if ! path2.exists() {
        DiffStatus::NewFile
    } else {
        let output = Command::new("diff").arg(path).arg(path2).output().expect("diff failed");
        io::stdout().write_all(&output.stdout).unwrap();
        match output.status.code().unwrap() {
            1 => DiffStatus::Changed,
            2 => DiffStatus::Failed,
            0 => DiffStatus::NoChanges,
            _ => DiffStatus::Failed
        }
    }
   
}

fn create_dir(maybe_path:Option<&Path>) {
    
    match maybe_path {
        None => {},
        Some(dir) => {
           if ! dir.exists() {
               let ans = ask(format!("create directory {} (y/n)", dir.display()));
               match ans.as_ref() {
                   "n" => {
                      println!("skipping mkdir {}", dir.display());
                   },
                   "y" => {
                        println!("mkdir {}", dir.display());
                        std::fs::create_dir_all(dir) .expect(&format!("create dir failed: {}", dir.display()));
                        if ! dir.exists() {
                            println!("dir not found {}", dir.display());
                        }
                   },
                   _ => create_dir(maybe_path) //repeat the question
               }
             } } }
}
fn create_from(template:&Path, path:&Path, destp:&Path) { 
    create_dir(destp.parent());
    match diff(path, destp) { 
        DiffStatus::NoChanges => println!("no changes '{}'", destp.display()),
        DiffStatus::Failed => println!("diff failed '{}'", destp.display()),
        DiffStatus::NewFile => {
            println!("create '{}'", destp.display());
            std::fs::copy(path, destp);
        },
        DiffStatus::Changed => {
            let ans = ask(format!("files don't match: {} {} (c)opy (m)erge (s)kip", path.display(), destp.display())); 
            println!("you answered '{}'", ans);
            match ans.as_ref() {
                "s" => {
                   println!("skipping cp {} {}", path.display(), destp.display()); 
                },
                "m" => {
                     merge(template, path, destp);
                     diff(path,destp);
                     create_from(template, path, destp);
                },
                "c" => {
                     std::fs::copy(path, destp);
                },
                _ => create_from(template, path, destp) //repeat diff and question
             }
        }
    }
}
fn report(sink:Sink) {
    match sink {
        Sink::SyncNewFile(files) => {
            println!("new file: {}", files.dest);
        },
        Sink::SyncTextFile(files) => {
           let genp = Path::new(files.gen.as_str());
           let destp = Path::new(files.dest.as_str());
           diff(genp, destp );
        },
        Sink::SyncBinaryFile(files) => {
           println!("binary {}", files.dest);
           let genp = Path::new(files.gen.as_str());
           let destp = Path::new(files.dest.as_str());
            if !destp.exists() { 
                println!("new binary: {}", destp.display());
            } else {
                diff(genp, destp );
            }
        }
    }

}
fn ask(question:String) -> String {
     println!("{}", question);
     let mut line = String::new();
     stdin().read_line(&mut line).expect("No User Input");
     return line.trim().to_string();

      //BufReader::new(std::io::stdin()).read_line().unwrap_or("");
}
fn create_or_diff(sink:Sink) {
    match sink {
        Sink::SyncNewFile(files) => {
           let genp = Path::new(files.gen.as_str());
           let destp = Path::new(files.dest.as_str());
           let template = Path::new(files.template.as_str());
           create_from(&template, &genp, destp);
        },
        Sink::SyncTextFile(files) => {
           let genp = Path::new(files.gen.as_str());
           let destp = Path::new(files.dest.as_str());
           let template = Path::new(files.template.as_str());
           diff(genp, destp );
           create_from(&template, genp, destp);
        },
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
fn generated_to_sink<'r>(generated:Result<Generated<'r>,String>) -> Result<Sink<'r>,String> {
    match generated {
        Err(why) => return Err(why),
        Ok(g) => match g {
            Generated::RText(files) => { 
                let exists = Path::new(files.dest.as_str()).exists();
                if exists { 
                    return Ok(Sink::SyncTextFile(files));
                } else {
                    return Ok(Sink::SyncNewFile(files));
                }},
            Generated::RBinary(files) => { 
                let exists = Path::new(files.dest.as_str()).exists();
                if exists { 
                    return Ok(Sink::SyncBinaryFile(files));
                } else {
                    return Ok(Sink::SyncNewFile(files));
                }
            }
        }
    }
}
fn process_dir(pattern:&str) -> Vec<PathBuf> {
    let mut vec = Vec::new();
    println!("glob {}", pattern);
    
    match glob(pattern) {
        Err(e1) => println!("{:?}", e1),
        Ok(paths) => {
            for globresult in paths {
               match globresult {  
                Err(glob_error) => println!("{:?}", glob_error),
                Ok(pathbuf) => vec.push(pathbuf)
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
fn sync(map: HashMap<String,String>, dirs:Vec<String>) {
    for dir in dirs.iter() { 
        println!("dir {}", dir);
      let paths = process_dir(dir);
    
    //let files_it = paths.iter().map(|path:&PathBuf| Files::new(path, dest_dir));
    for path in paths {
        let dest_dir = "out/".to_string();
        //let files = Files::new(path.as_path(), dest_dir);
        let files = Files::new(path.as_path(), dest_dir);
        //for files in files_it {
          //files: core::iter::Map<core::slice::Iter<'_, std::path::PathBuf>
            let gen = generate_recommended(&map, &files);
            let result_sink = generated_to_sink(gen);
            //  let gens = files_it.map(|&files| generate_recommended(&map, files));
              //let sinks = gens.map(|gen| generated_to_sink(gen));
              //for result_sink in sinks {
                match result_sink {
                    Err(msg) => { println!("err: {}", msg)},
                    Ok(sink)=>  {
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
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        println!("h");
        print_usage(&program, opts);
        return;
    }
    let prop:String = match matches.opt_str("p") {
        Some(p) => p,
        None => { 
            "".to_string()}//println!("p");}
            //print_usage(&program, opts); 
            //return; }
    };
    println!("prop {}", prop);
    let dirs:Vec<String> = if !matches.free.is_empty() {
        matches.free
    } else {
        println!("matches.free {:?}", matches.free.as_slice());
        print_usage(&program, opts);
        return;
    }; 
  
    let mut map: HashMap<String,String> = HashMap::new();
    properties(&mut map, prop);
    sync(map, dirs);   
}
