use consensus_types::block::Block;

//TODO
pub trait MintBlockProvider<T> {
    fn create_block() -> Block<T> {
        unimplemented!()
    }
}

//TODO
pub trait MainChainProvider<T> {
    fn connect_block(new_block:Block<T>) -> Option<Block<T>> {
        unimplemented!()
    }
}