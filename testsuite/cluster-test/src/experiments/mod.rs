mod packet_loss_random_validators;
mod reboot_random_validator;

pub use packet_loss_random_validators::PacketLossRandomValidators;
pub use reboot_random_validator::RebootRandomValidators;
use std::{collections::HashSet, fmt::Display};

pub trait Experiment: Display + Send {
    fn affected_validators(&self) -> HashSet<String>;
    fn run(&self) -> failure::Result<()>;
}
