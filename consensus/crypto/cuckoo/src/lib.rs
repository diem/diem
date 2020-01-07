pub use cuckaroo::CuckarooContext;
pub use cuckatoo::CuckatooContext;
pub use types::{PoWContext, Proof as CuckarooProof, PROOF_SIZE};

#[macro_use]
mod common;
#[allow(dead_code)]
mod cuckaroo;
#[allow(dead_code)]
mod cuckatoo;
mod error;
mod siphash;

mod types;
