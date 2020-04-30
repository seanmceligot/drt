pub mod diff;
pub mod fs;
pub mod parse;
pub mod properties;
pub mod template;
pub mod userinput;

#[derive(Clone, Copy)]
pub enum Mode {
    Active,
    Passive,
    Interactive
}
