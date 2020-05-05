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
use std::process::Command;
use log::trace;
use drt::DestFile;

// creates the tmp file for comparing to the dest file
pub fn generate_recommended_file<'a, 'b>(
    vars: &'a HashMap<&str, &str>,
    template: &'b SrcFile
) -> Result<GenFile, Error> {
    let gen = GenFile::new();

    let infile: Result<File, Error> = template.open();
    //let re = Regex::new(r"^(?P<k>[[:alnum:]\._]*)=(?P<v>.*)").unwrap();
    let re = Regex::new(r"@@(?P<t>[fns]):(?P<k>[A-Za-z0-9\.-]*)@@").unwrap();
    let reader = BufReader::new(infile.unwrap());
    let mut tmpfile: File = gen.open()?;
    for line in reader.lines() {
        let l: String = line.unwrap();
        match re.captures(l.as_str()) {
            Some(cap) => {
                let all: Match = cap.get(0).unwrap();
                let t: Match = cap.name("t").unwrap();
                let k: Match = cap.name("k").unwrap();
                let before: &str = &l[..all.start()];
                trace!("before {:?}", before);
                write!(tmpfile, "{}", before).expect("write failed");
                let key = k.as_str();
                let v = vars.get(key);
                trace!("template {:?}", t);
                trace!("key {}", key);
                trace!("val {:?}", v);
                trace!("line {:?}", l);

                if v.is_some() {
                    let value: &str = v.unwrap();
                    write!(tmpfile, "{}", value);
                } else {
                    // found varable but no key in vars so leave line alone
                    panic!("warn: NOT FOUND {}", key);
                };
                let after: &str = &l[all.end()..];
                trace!("after {:?}", after);
                writeln!(tmpfile, "{}", after).expect("write failed");
            }
            None => {
                trace!("no vars in line {:?}", l);
                writeln!(tmpfile, "{}", l).expect("Cannot write to tmp file");
            }
        }
    }
    return Ok(gen);
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
pub fn create_from<'f,'g>( mode: Mode, template: &'f SrcFile, gen: &'f GenFile, dest: &'f DestFile) -> Result<DiffStatus, Error> {
    trace!("dest {:?}", dest);
    dest.mkdirs();
    trace!("create_from");
    let status = diff(gen.path(), dest.path());
    match status {
        DiffStatus::NoChanges => {
            println!("no changes '{}'", dest);
            Ok(DiffStatus::NoChanges)
        }
        DiffStatus::Failed => {
            println!("diff failed '{}'", dest);
            Ok(DiffStatus::Failed)
        }
        DiffStatus::NewFile => {
            println!("create '{}'", dest);
            println!("cp {:?} {:?}", gen, dest);
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
                    &format!( "{}: {} {} (c)opy (m)erge (s)kip print (d)iff merge to (t)emplate", "files don't match", 
                        gen, 
                        dest))
            };
            match ans {
                'd' => {
                    for ch in difftext {
                        print!("{}", ch as char)
                    }
                }
                's' => {
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
                    create_from(mode, template, &gen, dest).expect("cannot create dest from template");
                }
                'c' => {
                    std::fs::copy(gen.path(), dest.path()).expect("copy failed");
                }
                _ => {
                    create_from(mode, template, &gen, dest).expect("cannot create dest from template");
                }
            }
            Ok(diff(&gen.path(), &dest.path()))
        }
    }
}
fn copy_passive(path: &GenFile, path2: &DestFile) -> Result<(), Error> {
    println!("WOULD: cp {:?} {:?}", path, path2 );
    // TODO: check if we can write to path2
    Ok(())
}
fn copy_active(path: &GenFile, path2: &DestFile) -> Result<(), Error> {
    println!("cp {:?} {:?}", path, path2 );
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
