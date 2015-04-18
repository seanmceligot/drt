#![feature(path_ext)] 
#![feature(convert)]
#![feature(collections)]

use std::error::Error;
use std::fs::copy;
use std::fs::File;
use std::path::PathBuf;
use std::path::Path;
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::HashMap;

extern crate regex;
use regex::Regex;

extern crate getopts;
use getopts::Options;
use std::env;

extern crate glob;
use self::glob::glob;
use self::glob::Paths;
use self::glob::PatternError;

use std::fs::OpenOptions;
use std::process::Command;

//mod files {
//use std::path::Path;
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
//}
//#[derive(Debug)]
fn properties<'a>(property_file:String) ->  HashMap<String,String> {
  //let system =  "system.config";
  let mut map: HashMap<String,String> = HashMap::new();
  let path = Path::new(property_file.as_str());
  println!("open {:?}", path);
  let file = match File::open(&path) {
    Ok(file) => file,
    Err(why) => panic!("couldn't open {}: {}", &path.display(), Error::description(&why)), 
  };
  let re = match Regex::new(r"^(?P<k>[[:alnum:]\._]*)=(?P<v>.*)") { Ok(re) => re, Err(err) => panic!("{}", err) };

  let reader = BufReader::new( file );
  for line in reader.lines() { 
    match line { 
        Err(_) => panic!("error readling line from {}", path.display()),
        Ok( l ) => {
            match re.captures(l.as_str()) { 
                None =>  println!("no match line \"{}\"", l),
                Some(c) => match c.name("k") {
                    None => { println!("no match k {}", l); }
                    Some(k) => {
                        match c.name("v") {
                            None => { println!("no match v {}", l); }
                            Some(v) => {
                                println!("insert {}={}", k,v);
                                map.insert(k.to_string(), v.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
  }  
  println!("map {:?}", map);
  map
}
enum Generated<'r> {
   RText(&'r Files),
   RBinary(&'r Files)
}
enum Sink<'r> {
    BSink(&'r Files),
    TSink(&'r Files),
    NewSink(&'r Files)
}
fn path_to_generate<'r>(map: &HashMap<String,String>, files: &'r Files) -> Result<Generated<'r>,String> {
  let path = Path::new(files.template.as_str());
  println!("open template {:?}", path);
  let file = match File::open(&path) {
    Ok(file) => file, 
    Err(why) => return Err(why.description().to_string())
  };
  let re = match Regex::new(r"@@[fns]:([A-Za-z0-9\.-]*)@@") { 
    Ok(re) => re, 
    Err(why) => return Err(format!("couldn't open {}: {} at {}", &path.display(), why.msg, why.pos))};

  let reader = BufReader::new( file );
  let lines = reader.lines();

  let mut tmpfile = match OpenOptions::new() .read(false).write(true).create(true) .open(Path::new(files.gen.as_str())) {
    Ok(f) => f,
    Err(why) => return Err(format!("couldn't open {}: {}", &path.display(), Error::description(&why)))
  };
  for line in lines { 
    match line { 
        Err(why) => { 
            println!("couldn't read lines (assuming binary) {}: {}", &path.display(), Error::description(&why));
            return Ok(Generated::RBinary(files)) }, 
        Ok( l ) => {
            match re.captures(l.as_str()) { 
                None =>  println!("no match line {}", l),
                Some(c) => match c.at(1) {
                    None =>  println!("111: {}", l),
                    Some(k) => {
                        println!("found variable {} in {:?} ", k, l);
                        match map.get(k) {
                            None => println!("222: {}", l),
                            Some(v) => { 
                            let newl = re.replace(l.as_str(), v.as_str());
                            //println!("old: {}", l);
                            //println!("new: {}", newl);
                            writeln!(tmpfile, "{}", newl);
    }}}}}}}}
    return Ok(Generated::RText(files));
}
fn diff(path:&Path, path2:&Path) -> Result<Vec<u8>,String>  {
    println!("diff {} {}", path.display(), path2.display());
    match Command::new("diff").arg(path).arg(path2).output() { 
        Err(e) => Err(e.description().to_string()),
        Ok(output) => { 
            println!("status: {}", output.status);
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            return Ok(output.stdout);
        }
    }
}

fn create_dir(maybe_path:Option<&Path>) {
    
    match maybe_path {
        None => {},
        Some(dir) => {
            std::fs::create_dir_all(dir);
            if ! dir.exists() {
                println!("dir not found {}", dir.display());
            }
            }}
}
fn create_from(path:&Path, destp:&Path) {
        (destp.parent());
    create_dir(destp.parent());
    println!("cp {} {} exists", path.display(), destp.display()); 
    std::fs::copy(path, destp);

}
fn report(sink:Sink) {
    match sink {
        Sink::NewSink(files) => {
            println!("new file: {}", files.dest);
        },
        Sink::TSink(files) => {
           let genp = Path::new(files.gen.as_str());
           let destp = Path::new(files.dest.as_str());
           diff(genp, destp );
        },
        Sink::BSink(files) => {
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
fn create_or_diff(sink:Sink) {
    match sink {
        Sink::NewSink(files) => {
           let genp = Path::new(files.gen.as_str());
           let destp = Path::new(files.dest.as_str());
            create_from(&genp, destp);
        },
        Sink::TSink(files) => {
           let genp = Path::new(files.gen.as_str());
           let destp = Path::new(files.dest.as_str());
           diff(genp, destp );
        },
        Sink::BSink(files) => {
           let genp = Path::new(files.gen.as_str());
           let destp = Path::new(files.dest.as_str());
            println!("binary {}", destp.display());
            if !destp.exists() { 
                create_from(genp, destp);
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
                    return Ok(Sink::TSink(files));
                } else {
                    return Ok(Sink::NewSink(files));
                }},
            Generated::RBinary(files) => { 
                let exists = Path::new(files.dest.as_str()).exists();
                if exists { 
                    return Ok(Sink::BSink(files));
                } else {
                    return Ok(Sink::NewSink(files));
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
     // process_dir(&system,"project/**");
      let paths = process_dir(dir);
    
    //let files_it = paths.iter().map(|path:&PathBuf| Files::new(path, dest_dir));
    for path in paths {
        let dest_dir = "/home/sean/rust/confsync/out/".to_string();
        let files = Files::new(path.as_path(), dest_dir);
        //for files in files_it {
          //files: core::iter::Map<core::slice::Iter<'_, std::path::PathBuf>
            let gen = path_to_generate(&map, &files);
            let result_sink = generated_to_sink(gen);
            //  let gens = files_it.map(|&files| path_to_generate(&map, files));
              //let sinks = gens.map(|gen| generated_to_sink(gen));
              //for result_sink in sinks {
                match result_sink {
                    Err(msg) => { println!("err: {}",msg)},
                    Ok(sink)=>  {
                        report(sink); 
                        //create_or_diff(sink); 
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

    let map = properties(prop);
    sync(map, dirs);   
}
