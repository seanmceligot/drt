use drt::diff::diff;
use drt::diff::DiffStatus;
use drt::userinput::ask;
use drt::Mode;
use drt::SrcFile;
use drt::GenFile;
use regex::Regex;
use regex::Match;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::io::ErrorKind;
use std::process::Command;
use log::trace;
use drt::DestFile;
use std::ops::Range;
use log::debug;
use ansi_term::Colour::{Green, Yellow};

#[test]
fn test_regex() {
    match match_line(String::from("a@@foo@@").as_str()) { 
        Some(("foo",_)) => {}, 
        Some((_,_)) => panic!("fail"), 
        None => panic!("expected Template") 
    }
    match match_line(String::from("@@foo@@a").as_str()) { 
        Some(("foo",_)) => {}, 
        Some((_,_)) => panic!("fail"), 
        None => panic!("expected Template") 
    }
    match match_line(String::from("@@foo@@").as_str()) { 
        Some(("foo",_)) => {}, 
        Some((_,_)) => panic!("fail"), 
        None => panic!("expected Template") 
    }
}
fn match_line<'a>(line: &'a str) -> Option<(&'a str,Range<usize>)> {
    let re = Regex::new(r"@@(?P<k>[A-Za-z0-9_\.-]*)@@").unwrap();
    match re.captures(line) {
        Some(cap) => {
            let all: Match = cap.get(0).unwrap();
            let k: Match = cap.name("k").unwrap();
            let key = k.as_str();
            Some( (key, all.range()) )
        }
        None => None
    }
}
pub fn replace_line( 
    vars: & HashMap<&str, &str>,
    line: String) -> Result<Option<String>,Error> { 
    match match_line(line.as_str()) {
        Some((key,range)) => {
            let mut new_line: String = String::new();
            let v = vars.get(key);
            trace!("key {}", key);
            trace!("val {:?}", v);
            trace!("line {:?}", line);
            let before: &str = &line[..range.start];
            let after: &str = &line[range.end..];
            new_line.push_str(before);
            //write!(tmpfile, "{}", before).expect("write failed");

            if let Some(value) = v {
                new_line.push_str(value);
                new_line.push_str(after);
                new_line.push('\n');
                //write!(tmpfile, "{}", value).expect("write failed");
                //writeln!(tmpfile, "{}", after).expect("write failed");
                Ok(Some(new_line))
            } else {
                Err(Error::new(ErrorKind::Other, format!("warn: variable NOT FOUND {}", key)))
            }
        },
        None => Ok(None)
    }
}
// creates the tmp file for comparing to the dest file
pub fn generate_recommended_file<'a, 'b>(
    vars: &'a HashMap<&str, &str>,
    template: &'b SrcFile
) -> Result<GenFile, Error> {
    let gen = GenFile::new();

    let infile: Result<File, Error> = template.open();
    //let re = Regex::new(r"^(?P<k>[[:alnum:]\._]*)=(?P<v>.*)").unwrap();
    let reader = BufReader::new(infile.unwrap());
    let mut tmpfile: &File = gen.open();
    for maybe_line in reader.lines() {
        let line: String = maybe_line.unwrap();
        match replace_line(vars, line.clone()) {
            Ok(maybe_new_line) => {
                match maybe_new_line {
                    Some(new_line) => {
                        writeln!(tmpfile, "{}", new_line).expect("write failed");
                    },
                    None => {
                            trace!("no vars in line {:?}", line);
                            writeln!(tmpfile, "{}", line).expect("Cannot write to tmp file");
                    }
                }
            },
            Err(e) => {
                panic!(e);
            }, 
        }
    }
    Ok(gen)
}

fn merge(mode: Mode, template: &SrcFile, gen: &GenFile, dest: &DestFile) -> bool {
    match mode {
        Mode::Interactive => merge_interactive(template, gen, dest),
        Mode::Passive => merge_passive(template, gen, dest),
        Mode::Active=> merge_active(template, gen, dest),
    }
}
fn merge_passive(_template: &SrcFile, path: &GenFile, path2: &DestFile) -> bool {
    let status = Command::new("diff")
        .arg(path)
        .arg(path2)
        .status()
        .expect("failed to execute process");
    println!("{} {}", Yellow.paint("WOULD: modify "), Yellow.paint(path2.to_string()) );
    println!("with: {}", status);
    status.success()
}
fn merge_active(_template: &SrcFile, path: &GenFile, dest: &DestFile) -> bool {
    let status = Command::new("cp")
        .arg("-v")
        .arg(path)
        .arg(dest)
        .status()
        .expect("failed to execute process");

    println!("{} {}", Green.paint("LIVE: modify "), Green.paint(dest.to_string()) );
    println!("with: {}", status);
    status.success()
}
fn merge_into_template(template: &SrcFile, _path: &GenFile, dest: &DestFile) -> bool {
    let status = Command::new("vim")
        .arg("-d")
        .arg(dest)
        .arg(template)
        .status()
        .expect("failed to execute process");

    println!("with: {}", status);
    status.success()
}
fn merge_interactive(_template: &SrcFile, path: &GenFile, dest: &DestFile) -> bool {
    let status = Command::new("vim")
        .arg("-d")
        .arg(path)
        .arg(dest)
        .status()
        .expect("failed to execute process");

    println!("with: {}", status);
    status.success()
}
pub fn create_from<'f>( mode: Mode, template: &'f SrcFile, gen: &'f GenFile, dest: &'f DestFile) -> Result<DiffStatus, Error> {
    trace!("dest {:?}", dest);
    dest.mkdirs();
    trace!("create_from");
    let status = diff(gen.path(), dest.path());
    match status {
        DiffStatus::NoChanges => {
            println!("{} {}", Yellow.paint("NO CHANGE: "), Yellow.paint(dest.to_string()) );
            Ok(DiffStatus::NoChanges)
        }
        DiffStatus::Failed => {
            debug!("diff failed '{}'", dest);
            Ok(DiffStatus::Failed)
        }
        DiffStatus::NewFile => {
            debug!("create '{}'", dest);
            debug!("cp {:?} {:?}", gen, dest);
            match mode {
                Mode::Passive =>  copy_passive(gen, dest)?,
                Mode::Active => copy_active(gen, dest)?,
                Mode::Interactive => copy_interactive(gen, dest)?
            };
            Ok(DiffStatus::NewFile)
        }
        DiffStatus::Changed(difftext) => {
            let ans = match mode {
                Mode::Passive => 'd',
                Mode::Active => 'c',
                Mode::Interactive => ask(
                    &format!( "{}: {} {} (c)opy (m)erge s(k)ip print (d)iff merge to (t)emplate", "files don't match", 
                        gen, 
                        dest))
            };
            match ans {
                'd' => {
                    println!("{} {}", Yellow.paint("WOULD: modify"), Yellow.paint(dest.to_string()) );
                    for ch in difftext {
                        print!("{}", ch as char)
                    }
                }
                'k' => {
                    println!("skipping cp {} {}", gen, dest);
                }
                't' => {
                    let _status_code = merge_into_template(template, gen, dest);
                    let _status = diff(&gen.path(), &dest.path());
                    create_from(mode, template, &gen, dest).expect("cannot create dest from template");
                }
                'm' => {
                    let _status_code = merge(mode, template, gen, dest);
                    let _status = diff(&gen.path(), &dest.path());
                    create_from(Mode::Interactive, template, &gen, dest).expect("cannot create dest from template");
                }
                'c' => {
                    copy_active(gen, dest).expect("copy failed");
                }
                _ => {
                    create_from(mode, template, &gen, dest).expect("cannot create dest from template");
                }
            }
            Ok(diff(&gen.path(), &dest.path()))
        }
    }
}
fn copy_passive(_path: &GenFile, path2: &DestFile) -> Result<(), Error> {
    println!("{} {}", Yellow.paint("WOULD: create "), Yellow.paint(path2.to_string()) );
    // TODO: check if we can write to path2
    Ok(())
}
fn copy_active(path: &GenFile, path2: &DestFile) -> Result<(), Error> {
    println!("{} {}", Green.paint("LIVE: create "), Green.paint(path2.to_string()) );
    std::fs::copy(path.path(), path2.path()).expect("create_from: copy failed:");
    Ok(())
}
fn copy_interactive(path: &GenFile, path2: &DestFile) -> Result<(), Error> {
    let status = Command::new("cp")
        .arg("-i")
        .arg(path)
        .arg(path2)
        .status()
        .expect("failed to execute process");

    println!("with: {}", status);
    if status.success() {
        Ok(())
    } else {
        panic!("cp failed: {:?} -> {:?}", path, path2)
    }
}
