use drt::diff::diff;
use drt::diff::DiffStatus;
use drt::fs::create_dir;
use drt::userinput::ask;
use drt::Mode;
#[allow(dead_code)]
use regex::Regex;
use std::collections::HashMap;
use std::convert::AsRef;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::path::PathBuf;
use std::process::Command;

pub struct TemplateFiles {
    pub template: PathBuf,
    pub gen: PathBuf,
    pub dest: PathBuf,
}

impl TemplateFiles {
#[allow(dead_code)]
    pub fn new(in_template: PathBuf, out_file: PathBuf) -> Option<TemplateFiles> {
        let gen = PathBuf::from("/tmp").join(PathBuf::from(out_file.file_name().unwrap()));
        Some(TemplateFiles {
            template: in_template,
            gen: gen,
            dest: out_file,
        })
    }
}

#[allow(dead_code)]
pub enum Generated<'r> {
    RText(&'r TemplateFiles),
    _RBinary(&'r TemplateFiles),
}

// creates the tmp file for comparing to the dest file
#[allow(dead_code)]
pub fn generate_recommended_file<'fs, 'f>(
    vars: &'fs HashMap<&str, &str>,
    files: &'f TemplateFiles,
) -> Result<Generated<'f>, Error> {
    println!("open template {:?}", files.template);
    let infile: Result<File, Error> = File::open(&files.template);
    //let re = Regex::new(r"^(?P<k>[[:alnum:]\._]*)=(?P<v>.*)").unwrap();
    let re = Regex::new(r"@@(?P<t>[fns]):(?P<k>[A-Za-z0-9\.-]*)@@").unwrap();
    let reader = BufReader::new(infile.unwrap());
    let mut tmpfile = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&files.gen)
        .unwrap();
    for line in reader.lines() {
        let l = line.unwrap();
        match re.captures(l.as_str()) {
            Some(cap) => {
                let t = cap.name("t").unwrap().as_str();
                let k = cap.name("k").unwrap().as_str();
                let v = vars.get(k);
                println!("t {:?}", t);
                println!("k {:?}", k);
                println!("v {:?}", v);
                println!("l {:?}", l);

                if v.is_some() {
                    // change match to vars[k[
                    let value: &str = v.unwrap();
                    //let value: &str = "VALUE";
                    // replace line: |@@key@@| with line |value|
                    writeln!(tmpfile, "{}", re.replace(l.as_str(), value))
                        .expect("Cannot Write to tmp file")
                } else {
                    // found varable but no key in vars so leave line alone
                    panic!("warn: NOT FOUND {}", k);
                    //writeln!(tmpfile, "{}", l).expect("Cannot write to tmp file");
                };
            }
            None => {
                // no variable in line
                writeln!(tmpfile, "{}", l).expect("Cannot write to tmp file");
            }
        }
    }
    return Ok(Generated::RText(files));
}

fn merge(mode: Mode, template: &PathBuf, path: &PathBuf, path2: &PathBuf) -> bool {
    match mode {
        Mode::Interactive => merge_interactive(template, path, path2),
        Mode::Passive => merge_passive(template, path, path2),
        Mode::Active=> merge_active(template, path, path2),
    }
}
fn merge_passive(_template: &PathBuf, path: &PathBuf, path2: &PathBuf) -> bool {
    let status = Command::new("diff")
        .arg(path.as_os_str())
        .arg(path2.as_os_str())
        .status()
        .expect("failed to execute process");

    println!("with: {}", status);
    status.success()
}
fn merge_active(_template: &PathBuf, path: &PathBuf, path2: &PathBuf) -> bool {
    let status = Command::new("cp")
        .arg("-v")
        .arg(path.as_os_str())
        .arg(path2.as_os_str())
        .status()
        .expect("failed to execute process");

    println!("with: {}", status);
    status.success()
}
fn merge_interactive(template: &PathBuf, path: &PathBuf, path2: &PathBuf) -> bool {
    let status = Command::new("vim")
        .arg("-d")
        .arg(template.as_os_str())
        .arg(path.as_os_str())
        .arg(path2.as_os_str())
        .status()
        .expect("failed to execute process");

    println!("with: {}", status);
    status.success()
}

pub fn create_from<'f>(
    mode: Mode,
    template: &'f PathBuf,
    path: &'f PathBuf,
    dest: &'f PathBuf,
) -> Result<DiffStatus, Error> {
    create_dir(dest.parent());
    let status = diff(&path, &dest);
    match status {
        DiffStatus::NoChanges => {
            println!("no changes '{}'", dest.display());
            Ok(DiffStatus::NoChanges)
        }
        DiffStatus::Failed => {
            println!("diff failed '{}'", dest.display());
            Ok(DiffStatus::Failed)
        }
        DiffStatus::NewFile => {
            println!("create '{}'", dest.display());
            std::fs::copy(path, dest).expect("create_from: copy failed:");
            Ok(DiffStatus::NewFile)
        }
        DiffStatus::Changed(difftext) => {
            let ans = ask(format!(
                "files don't match: {} {} (c)opy (m)erge (s)kip (p)rint diff",
                path.to_str().expect("invalid path"),
                dest.to_str().expect("invalid test"),
            ));
            println!("you answered '{}'", ans);
            match ans.as_ref() {
                "p" => {
                    println!("{:?}", difftext);
                }
                "s" => {
                    println!("skipping cp {} {}", path.display(), dest.display());
                }
                "m" => {
                    let _status_code = merge(mode, template, path, dest);
                    let _status = diff(&path, &dest);
                    create_from(mode, template, &path, dest).expect("cannot create dest from template");
                }
                "c" => {
                    std::fs::copy(path, dest).expect("copy failed");
                }
                _ => {
                    create_from(mode, template, &path, dest).expect("cannot create dest from template");
                }
            }
            Ok(DiffStatus::Changed(difftext))
        }
    }
}
