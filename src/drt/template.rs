use ansi_term::Colour::{Green, Yellow, Red};
use drt::diff::diff;
use drt::diff::DiffStatus;
use drt::userinput::ask;
use drt::DestFile;
use drt::GenFile;
use drt::Mode;
use drt::SrcFile;
use log::debug;
use log::trace;
use regex::Match;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::ops::Range;
use std::process::Command;
use drt::DrtError;

#[test]
fn test_regex() {
    match match_line(String::from("a@@foo@@").as_str()) {
        Some(("foo", _)) => {}
        Some((_, _)) => panic!("fail"),
        None => panic!("expected Template"),
    }
    match match_line(String::from("@@foo@@a").as_str()) {
        Some(("foo", _)) => {}
        Some((_, _)) => panic!("fail"),
        None => panic!("expected Template"),
    }
    match match_line(String::from("@@foo@@").as_str()) {
        Some(("foo", _)) => {}
        Some((_, _)) => panic!("fail"),
        None => panic!("expected Template"),
    }
}
fn match_line<'a>(line: &'a str) -> Option<(&'a str, Range<usize>)> {
    let re = Regex::new(r"@@(?P<k>[A-Za-z0-9_\.-]*)@@").unwrap();
    match re.captures(line) {
        Some(cap) => {
            let all: Match = cap.get(0).unwrap();
            let k: Match = cap.name("k").unwrap();
            let key = k.as_str();
            Some((key, all.range()))
        }
        None => None,
    }
}
pub enum ChangeString {
    Changed(String),
    Unchanged,
}
pub fn replace_line(vars: &HashMap<&str, &str>, line: String) -> Result<ChangeString, DrtError> {
    match match_line(line.as_str()) {
        Some((key, range)) => {
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
                Ok(ChangeString::Changed(new_line))
            } else {
                Err(DrtError::VarNotFound(String::from(key)))
            }
        }
        None => Ok(ChangeString::Unchanged)
    }
}
// creates the tmp file for comparing to the dest file
pub fn generate_recommended_file<'a, 'b>(
    vars: &'a HashMap<&str, &str>,
    template: &'b SrcFile,
) -> Result<GenFile, DrtError> {
    let gen = GenFile::new();
    let infile: Result<File, Error> = template.open();
    //let re = Regex::new(r"^(?P<k>[[:alnum:]\._]*)=(?P<v>.*)").unwrap();
    let reader = BufReader::new(infile.unwrap());
    let mut tmpfile: &File = gen.open();
    for maybe_line in reader.lines() {
        let line: String = maybe_line.unwrap();
        match replace_line(vars, line.clone()) {
            Ok(replaced_line) => match replaced_line {
                ChangeString::Changed(new_line) => {
                    writeln!(tmpfile, "{}", new_line).expect("write failed");
                }
                ChangeString::Unchanged => {
                    trace!("no vars in line {:?}", line);
                    writeln!(tmpfile, "{}", line).expect("Cannot write to tmp file");
                }
            },
            Err(e) => {
                return Err(e)
            }
        }
    }
    Ok(gen)
}

fn merge(mode: Mode, template: &SrcFile, gen: &GenFile, dest: &DestFile) -> bool {
    match mode {
        Mode::Interactive => merge_interactive(template, gen, dest),
        Mode::Passive => merge_passive(template, gen, dest),
        Mode::Active => merge_active(template, gen, dest),
    }
}
fn merge_passive(_template: &SrcFile, path: &GenFile, path2: &DestFile) -> bool {
    let status = Command::new("diff")
        .arg(path)
        .arg(path2)
        .status()
        .expect("failed to execute process");
    println!(
        "{} {}",
        Yellow.paint("WOULD: modify "),
        Yellow.paint(path2.to_string())
    );
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

    println!(
        "{} {}",
        Green.paint("LIVE: modify "),
        Green.paint(dest.to_string())
    );
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
pub fn create_from<'f>(
    mode: Mode,
    template: &'f SrcFile,
    gen: &'f GenFile,
    dest: &'f DestFile,
) -> Result<(), DrtError> {
    trace!("dest {:?}", dest);
    dest.mkdirs();
    trace!("create_from");
    let status = diff(gen.path(), dest.path());
    match status {
        DiffStatus::NoChanges => {
            println!(
                "{} {}",
                Yellow.paint("NO CHANGE: "),
                Yellow.paint(dest.to_string())
            );
            Ok(())
        }
        DiffStatus::Failed => {
            debug!("diff failed '{}'", dest);
            Err(DrtError::Error)
        }
        DiffStatus::NewFile => {
            debug!("create '{}'", dest);
            debug!("cp {:?} {:?}", gen, dest);
            match mode {
                Mode::Passive => create_passive(gen, dest, template),
                Mode::Active => copy_active(gen, dest, template),
                Mode::Interactive => copy_interactive(gen, dest, template),
            }
        }
        DiffStatus::Changed(difftext) => {
            let ans = match mode {
                Mode::Passive => 'd',
                Mode::Active => 'o',
                Mode::Interactive => ask(
                    &format!( "{}: {} {} (o)verwrite / (m)erge[vimdiff] / (c)ontinue / (d)iff / merge to (t)emplate", "files don't match", gen, dest))
            };
            match ans {
                'd' => {
                    println!(
                        "{} {}",
                        Yellow.paint("WOULD: modify"),
                        Yellow.paint(dest.to_string())
                    );
                    for ch in difftext {
                        print!("{}", ch as char)
                    }
                    Ok(())
                }
                'c' => {
                    println!("skipping cp {} {}", gen, dest);
                    Ok(())
                }
                't' => {
                    merge_into_template(template, gen, dest);
                    Ok(())
                }
                'm' => {
                    merge(mode, template, gen, dest);
                    Ok(())
                }
                'o' => {
                    copy_active(gen, dest, template)
                }
                _ => {
                    create_from(mode, template, &gen, dest)
                }
            }
        }
    }
}
fn create_passive(path: &GenFile, path2: &DestFile, template: &SrcFile) -> Result<(), DrtError> {
    println!(
        "{} {} [{}]  ->{}",
        Yellow.paint("WOULD: create from template"),
        Yellow.paint(template.to_string()),
        Yellow.paint(path.to_string()),
        Yellow.paint(path2.to_string())
    );
    // TODO: check if we can write to path2
    Ok(())
}
fn copy_active(path: &GenFile, path2: &DestFile, template: &SrcFile) -> Result<(), DrtError> {
    println!(
        "{} {} [{}]  ->{}",
        Yellow.paint("LIVE: create from template"),
        Yellow.paint(template.to_string()),
        Yellow.paint(path.to_string()),
        Yellow.paint(path2.to_string())
    );
    println!(
        "{} {}",
        Green.paint("LIVE: create "),
        Green.paint(path2.to_string())
    );
    match std::fs::copy(path.path(), path2.path()) {
        Err(e) => {
            println!("{} {}", 
                Red.paint("error: copy failed"), 
                Red.paint(e.to_string())
            );
            Err(DrtError::Error)
        }
        Ok(_) => Ok(())
    }
}
fn copy_interactive(path: &GenFile, path2: &DestFile, _template: &SrcFile) -> Result<(), DrtError> {
    // TODO: add vimdiff support
    // TODO: use ask and copy_passive
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
