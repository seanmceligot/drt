pub mod diff;
pub mod fs;
pub mod parse;
pub mod properties;
pub mod template;
pub mod userinput;
use std::io::Error;
use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::fs::OpenOptions;
use log::trace;
use std::fmt;
use std::ffi::OsStr;
use drt::fs::create_dir;

#[derive(Clone, Copy)]
pub enum Mode {
    Active,
    Passive,
    Interactive
}
#[derive(Debug)]
pub struct SrcFile {
    path: PathBuf,
}

impl SrcFile {
    pub fn new(path: PathBuf) -> SrcFile {
        SrcFile { path: path }
    }
    pub fn open(&self) -> Result<File, Error> {
        trace!("open path {:?}", self.path);
        OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(&self.path)
    }
}

#[derive(Debug)]
pub struct DestFile {
    path: PathBuf,
}
impl DestFile {
    pub fn new(path: PathBuf) -> DestFile {
        DestFile { path: path }
    }
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
    pub fn path(&self) -> & Path {
        self.path.as_path()
    }
    pub fn mkdirs(&self) {
	let dir = self.path.parent();
	trace!("dest dir {:?}", dir);
	create_dir(dir);
    }
}
#[derive(Debug)]
pub struct GenFile {
    path: PathBuf,
}
impl GenFile {
    pub fn new() -> GenFile {
        let path_file = PathBuf::from("temp.patherated");
        GenFile { path: path_file }
    }
    pub fn open(&self) -> Result<File, Error> {
        trace!("open path {:?}", self.path);
        OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .open(&self.path)
    }
    pub fn path(&self) -> & Path {
        self.path.as_path()
    }
}

impl fmt::Display for SrcFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

impl fmt::Display for DestFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

impl fmt::Display for GenFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}
impl AsRef<OsStr> for DestFile {
    fn as_ref(&self) -> &OsStr {
        self.path.as_os_str()
    }
}
impl AsRef<OsStr> for SrcFile {
    fn as_ref(&self) -> &OsStr {
        self.path.as_os_str()
    }
}
impl AsRef<OsStr> for GenFile {
    fn as_ref(&self) -> &OsStr {
        self.path.as_os_str()
    }
}
