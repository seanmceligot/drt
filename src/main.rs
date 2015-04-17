#![feature(path_ext)] 
#![feature(convert)]
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

use std::fs::OpenOptions;
use std::process::Command;


//#[derive(Debug)]
fn properties<'a>(property_file:&str) ->  HashMap<String,String> {
  //let system =  "system.config";
  let mut map: HashMap<String,String> = HashMap::new();
  let path = Path::new(property_file);
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
enum Generated {
   RText(String),
   RBinary(String)
}
enum Sink {
    BSink(String,String),
    TSink(String,String),
    NewSink(String,String)
}
//#![feature(collections)]
//#![feature(str_words)]
fn generate<'r>(map: &HashMap<String,String> , file:&Path) -> Result<Generated,String>  {
  let gen: String =  ("/tmp/sink.tmp").to_string();
  let path = Path::new(file);
  println!("open {:?}", path);
  let file = match File::open(&path) {
    Ok(file) => file, 
    Err(why) => return Err(why.description().to_string())
  };
  let re = match Regex::new(r"@@[fns]:([A-Za-z0-9\.-]*)@@") { 
    Ok(re) => re, 
    Err(why) => return Err(format!("couldn't open {}: {} at {}", &path.display(), why.msg, why.pos))};

  let reader = BufReader::new( file );
  let lines = reader.lines();

  let mut tmpfile = match OpenOptions::new() .read(false).write(true).create(true) .open(Path::new(gen.as_str())) {
    Ok(f) => f,
    Err(why) => return Err(format!("couldn't open {}: {}", &path.display(), Error::description(&why)))
  };
  for line in lines { 
    match line { 
        Err(why) => { 
            println!("couldn't read lines (assuming binary) {}: {}", &path.display(), Error::description(&why));
            return Ok(Generated::RBinary(gen)) }, 
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
    //diff(path, gen);
    return Ok(Generated::RText(gen));
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
fn create_or_diff(sink:Sink) {
    match sink {
        Sink::NewSink(gen,dest) => {
           let genp = Path::new(gen.as_str());
           let destp = Path::new(dest.as_str());
            create_from(&genp, destp);
        },
        Sink::TSink(gen,dest) => {
           let genp = Path::new(gen.as_str());
           let destp = Path::new(dest.as_str());
           diff(genp, destp );
        },
        Sink::BSink(gen,dest) => {
           let genp = Path::new(gen.as_str());
           let destp = Path::new(dest.as_str());
            println!("binary {}", destp.display());
            if (!destp.exists()) { 
                create_from(genp, destp);
            }
        }
    }

}
//fn get_x<'r>(p: &'r Point) -> &'r f64 { &p.x }
fn textOrBinary<'r>(map: &HashMap<String,String>, path: &Path) -> Result<Sink,String> {
    println!("path {}", path.display());
    let dest = "/home/sean/rust/sink/out/".to_string() + (path.to_str().unwrap());
    println!("dest {}", dest);
    match generate(&map, &path) {
        Err(why) => return Err(why),
        Ok(g) => match g {
            Generated::RText(gen) => { 
                let exists = Path::new(dest.as_str()).exists();
                if (exists) { 
                    return Ok(Sink::TSink(gen,dest));
                } else {
                    return Ok(Sink::NewSink(gen, dest));
                }},
            Generated::RBinary(gen) => { 
                let exists = Path::new(dest.as_str()).exists();
                if (exists) { 
                    return Ok(Sink::BSink(gen,dest));
                } else {
                    return Ok(Sink::NewSink(gen, dest));
                }
            }
        }
    }
}
fn process_dir(prop:&str, pattern:&str) {
    let map = properties(prop);

    println!("glob {}", pattern);
     match glob(pattern) {
       Err(e) => println!("glob error {}", e.msg),
       Ok(result) => {
         for entry in result {
              match entry {
                 Ok(path) => {
                     textOrBinary(&map, &path);
                  },
                  Err(e) => println!("err {:?}", e),
              };
          }
       }
     }
    
       
}
fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
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
    let prop = match matches.opt_str("p") {
        Some(p) => p,
        None => { 
            "".to_string()}//println!("p");}
            //print_usage(&program, opts); 
            //return; }
    };
    println!("prop {}", prop);
    let dirs = if !matches.free.is_empty() {
        matches.free
    } else {
        println!("matches.free {:?}", matches.free.as_slice());
        print_usage(&program, opts);
        return;
    };   
    for dir in dirs.iter() { 
        println!("dir {}", dir);
     // process_dir(&system,"project/**");
      process_dir(&prop,dir);
    }
}
