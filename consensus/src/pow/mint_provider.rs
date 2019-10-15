use crate::base::MintBlockProvider;
use consensus_types::block::Block;

pub struct MintBlock {}

impl MintBlockProvider for MintBlock {
    fn create_block() -> Block<T> {
        unimplemented!()
    }
}