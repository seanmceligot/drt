extern crate tempfile;
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
use drt::fs::create_dir;


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

            if let Some(value) = v {
                new_line.push_str(value);
                new_line.push_str(after);
                new_line.push('\n');
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
fn merge_passive(_template: &SrcFile, gen: &GenFile, dest: &DestFile) -> bool {
    let status = Command::new("diff")
        .arg(gen)
        .arg(dest)
        .status()
        .expect("failed to execute process");
    println!(
        "{} {}",
        Yellow.paint("WOULD: modify "),
        Yellow.paint(dest.to_string())
    );
    println!("with: {}", status);
    status.success()
}
fn merge_active(_template: &SrcFile, gen: &GenFile, dest: &DestFile) -> bool {
    let status = Command::new("cp")
        .arg("-v")
        .arg(gen)
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
fn merge_into_template(template: &SrcFile, _gen: &GenFile, dest: &DestFile) -> bool {
    let status = Command::new("vim")
        .arg("-d")
        .arg(dest)
        .arg(template)
        .status()
        .expect("failed to execute process");

    println!("with: {}", status);
    status.success()
}
fn merge_interactive(_template: &SrcFile, gen: &GenFile, dest: &DestFile) -> bool {
    let status = Command::new("vim")
        .arg("-d")
        .arg(gen)
        .arg(dest)
        .status()
        .expect("failed to execute process");

    println!("with: {}", status);
    status.success()
}
fn update_from_template_passive(difftext: std::vec::IntoIter<u8>,template: &SrcFile, gen: &GenFile, dest : &DestFile) -> Result<(), DrtError> { 
    print_template_command(Yellow, "WOULD", template, gen, dest);
    for ch in difftext { print!("{}", ch as char) }
    Ok(())
}
fn print_template_command(color: ansi_term::Colour, verb: &'static str, template: &SrcFile, gen: &GenFile, dest : &DestFile) {
    println!(
        "{}: {} {} [{}]  ->{}",
        color.paint(verb),
        color.paint("create from template"),
        color.paint(template.to_string()),
        color.paint(gen.to_string()),
        color.paint(dest.to_string())
    );
}
fn update_from_template_active(template: &SrcFile, gen: &GenFile, dest: &DestFile) -> Result<(), DrtError> {
    copy_active(gen, dest, template)
}
fn update_from_template_interactive(difftext: std::vec::IntoIter<u8>, template: &SrcFile, gen: &GenFile, dest: &DestFile) -> Result<(), DrtError> {
    let ans = ask( &format!( "{}: {} {} (o)verwrite / (m)erge[vimdiff] / s(k)ip / (d)iff / merge to (t)emplate", "files don't match", gen, dest));
    match ans {
        'd' => {
            update_from_template_passive(difftext, template, gen, dest)
        }
        'k' => {
            print_template_command(Yellow, "SKIP", template, gen, dest);
            Ok(())
        }
        't' => {
            merge_into_template(template, gen, dest);
            Ok(())
        }
        'm' => {
            merge(Mode::Interactive, template, gen, dest);
            Ok(())
        }
        'o' => {
            copy_active(gen, dest, template)
        }
        _ => {
            update_from_template(Mode::Interactive, template, &gen, dest)
        }
    }
}
pub fn update_from_template<'f>(
    mode: Mode,
    template: &'f SrcFile,
    gen: &'f GenFile,
    dest: &'f DestFile,
) -> Result<(), DrtError> {
    trace!("dest {:?}", dest);
    create_dir(mode, dest.path.parent());
    trace!("update_from_template");
    let status = diff(gen.path(), dest.path());
    match status {
        DiffStatus::NoChanges => {
            println!( "{} {}", Yellow.paint("NO CHANGE: "), Yellow.paint(dest.to_string()));
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
            match mode {
                Mode::Passive =>update_from_template_passive(difftext, template, gen, dest), 
                Mode::Active =>update_from_template_active(template, gen, dest), 
                Mode::Interactive => update_from_template_interactive(difftext, template, gen, dest)
            }
        }
    }
}
fn create_passive(gen: &GenFile, dest: &DestFile, template: &SrcFile) -> Result<(), DrtError> {
    print_template_command(Yellow, "WOULD", template, gen, dest);
    println!(
        "{} {} [{}]  ->{}",
        Yellow.paint("WOULD: create from template"),
        Yellow.paint(template.to_string()),
        Yellow.paint(gen.to_string()),
        Yellow.paint(dest.to_string())
    );
    // TODO: check if we can write to dest
    Ok(())
}
fn copy_active(gen: &GenFile, dest: &DestFile, template: &SrcFile) -> Result<(), DrtError> {
    print_template_command(Green, "LIVE", template, gen, dest);
    match std::fs::copy(gen.path(), dest.path()) {
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
fn copy_interactive(gen: &GenFile, dest: &DestFile, _template: &SrcFile) -> Result<(), DrtError> {
    // TODO: add vimdiff support
    // TODO: use ask and copy_passive
    let status = Command::new("cp")
        .arg("-i")
        .arg(gen)
        .arg(dest)
        .status()
        .expect("failed to execute process");

    println!("with: {}", status);
    if status.success() {
        Ok(())
    } else {
        panic!("cp failed: {:?} -> {:?}", gen, dest)
    }
}
