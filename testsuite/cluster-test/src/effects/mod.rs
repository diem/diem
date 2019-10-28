mod packet_loss;
mod reboot;
mod remove_network_effects;
mod stop_container;

use failure;
pub use packet_loss::PacketLoss;
pub use reboot::Reboot;
pub use remove_network_effects::RemoveNetworkEffects;
use std::fmt::Display;
pub use stop_container::StopContainer;

pub trait Action: Display + Send {
    fn apply(&self) -> failure::Result<()>;
    fn is_complete(&self) -> bool;
}

pub trait Effect: Display + Send {
    fn activate(&self) -> failure::Result<()>;
    fn deactivate(&self) -> failure::Result<()>;
}
