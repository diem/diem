use ::libra_types::proto::types;
mod consensus {
    pub use network::proto::Block;
}
pub mod block_storage {
    include!(concat!(env!("OUT_DIR"), "/block_storage.rs"));
}
