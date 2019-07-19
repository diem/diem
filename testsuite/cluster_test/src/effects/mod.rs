mod reboot;

use failure;
pub use reboot::Reboot;

pub trait Effect {
    fn apply(&self) -> failure::Result<()>;
    fn is_complete(&self) -> bool;
}
