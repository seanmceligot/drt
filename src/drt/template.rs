#[allow(dead_code)]
use regex::Regex;
use std::collections::HashMap;
use std::convert::AsRef;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::Error;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use drt::diff::DiffStatus;
use drt::fs::create_dir;
use drt::diff::diff;
use drt::userinput::ask;

pub struct TemplateFiles {
    pub template: PathBuf,
    pub gen: PathBuf,
    pub dest: PathBuf,
}

impl TemplateFiles {
    pub fn new(in_template: PathBuf, out_file: PathBuf) -> Option<TemplateFiles> {
        let gen = PathBuf::from("/tmp").join(PathBuf::from(out_file.file_name().unwrap()));
        Some(TemplateFiles {
            template: in_template,
            gen: gen,
            dest: out_file,
        })
    }
}

pub enum Generated<'r> {
    RText(&'r TemplateFiles),
    _RBinary(&'r TemplateFiles),
}

// creates the tmp file for comparing to the dest file
pub fn generate_recommended_file<'fs,'f>( map: &'fs HashMap<String, String>, files: &'f TemplateFiles) -> Result<Generated<'f>, Error> {
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
                let v = map.get(k);
                println!("t {:?}", t);
                println!("k {:?}", k);
                println!("v {:?}", v);
                println!("l {:?}", l);

                if v.is_some() {
                    // change match to map[k[
                    writeln!(tmpfile, "{}", re.replace(l.as_str(), v.unwrap().as_str()))
                        .expect("Cannot Write to tmp file")
                } else {
                    // found varable but no key in map so leave line alone
                    println!("warn: NOT FOUND {}", k);
                    writeln!(tmpfile, "{}", l).expect("Cannot write to tmp file");
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

fn merge(template: & PathBuf, path: & PathBuf, path2: & PathBuf) -> bool {
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

pub fn create_from<'f>(template: &'f PathBuf, path: &'f PathBuf, dest: &'f PathBuf) -> Result< DiffStatus, Error> {
    create_dir(dest.parent());
    let status = diff(&path, &dest);
    match status  {
        DiffStatus::NoChanges => {
            println!("no changes '{}'", dest.display());
            Ok(DiffStatus::NoChanges)
        },
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
                    let _status_code = merge(template, path, dest);
                    let _status = diff(&path, &dest);
                    create_from(template, &path, dest).expect("cannot create dest from template");
                }
                "c" => {
                    std::fs::copy(path, dest).expect("copy failed");
                }
                _ => {
                    create_from(template, &path, dest).expect("cannot create dest from template");
                }
            }
            Ok(DiffStatus::Changed(difftext))
        }
    }
}
