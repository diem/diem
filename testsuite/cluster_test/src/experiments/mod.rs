mod reboot_random_validator;

pub use reboot_random_validator::RebootRandomValidators;
use std::{collections::HashSet, fmt::Display};

pub trait Experiment: Display {
    fn affected_validators(&self) -> HashSet<String>;
    fn run(&self) -> failure::Result<()>;
}
