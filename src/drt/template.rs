use regex::Regex;
use std::collections::HashMap;
use std::convert::AsRef;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::Error;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use drt::diff::DiffStatus;
use drt::fs::create_dir;
use drt::diff::diff;
use drt::userinput::ask;

pub struct TemplateFiles {
    pub template: String,
    pub gen: String,
    pub dest: String,
}

impl TemplateFiles {
    pub fn new(ppath: &Path, dest_dir: String) -> TemplateFiles {
        let gen = Path::new("/tmp").join(Path::new(ppath.file_name().unwrap()));
        let dest = Path::new(&*dest_dir).join(*&ppath);
        TemplateFiles {
            template: ppath.to_str().unwrap().to_string(),
            gen: gen.to_str().unwrap().to_string(),
            dest: dest.to_str().unwrap().to_string(),
        }
    }
}

pub enum Generated<'r> {
    RText(&'r TemplateFiles),
    RBinary(&'r TemplateFiles),
}

pub fn generate_recommended<'r>(
    map: &HashMap<String, String>,
    files: &'r TemplateFiles,
) -> Result<Generated<'r>, String> {
    let path = Path::new(files.template.as_str());
    println!("open template {:?}", path);
    let infile: Result<File, Error> = File::open(&path);
    //let re = Regex::new(r"^(?P<k>[[:alnum:]\._]*)=(?P<v>.*)").unwrap();
    let re = Regex::new(r"@@(?P<t>[fns]):(?P<k>[A-Za-z0-9\.-]*)@@").unwrap();
    let reader = BufReader::new(infile.unwrap());
    let mut tmpfile = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(Path::new(files.gen.as_str()))
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

fn merge(template: &Path, path: &Path, path2: &Path) -> () {
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

pub fn create_from(template: &Path, path: &Path, destp: &Path) {
    create_dir(destp.parent());
    match diff(path, destp) {
        DiffStatus::NoChanges => println!("no changes '{}'", destp.display()),
        DiffStatus::Failed => println!("diff failed '{}'", destp.display()),
        DiffStatus::NewFile => {
            println!("create '{}'", destp.display());
            std::fs::copy(path, destp).expect("create_from: copy failed:");
        }
        DiffStatus::Changed => {
            let ans = ask(format!(
                "files don't match: {} {} (c)opy (m)erge (s)kip",
                path.display(),
                destp.display()
            ));
            println!("you answered '{}'", ans);
            match ans.as_ref() {
                "s" => {
                    println!("skipping cp {} {}", path.display(), destp.display());
                }
                "m" => {
                    merge(template, path, destp);
                    diff(path, destp);
                    create_from(template, path, destp);
                }
                "c" => {
                    std::fs::copy(path, destp).expect("copy failed");
                }
                _ => create_from(template, path, destp), //repeat diff and question
            }
        }
    }
}
