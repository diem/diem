pub use cuckaroo::CuckarooContext;
pub use cuckatoo::CuckatooContext;
pub use types::{PoWContext, Proof as CuckarooProof, PROOF_SIZE};

#[macro_use]
mod common;
mod cuckaroo;
mod cuckatoo;
mod error;
mod siphash;

mod types;
