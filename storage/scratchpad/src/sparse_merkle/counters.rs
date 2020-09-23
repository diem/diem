use libra_metrics::{register_histogram, Histogram};
use once_cell::sync::Lazy;

pub static LEAF_NODE_HASH: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "scratchpad_leaf_node_hash",
        "Time spent in hashing leaf node"
    )
    .unwrap()
});

pub static STATE_BLOB_SIZE: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "scratchpad_state_blob_size",
        "Size of the leaf account blob size that is being hashed"
    )
    .unwrap()
});

pub static SMT_UPDATE_ONE: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "scratchpad_smt_update_one",
        "Time spent in sparse_merkle_tree::update_one"
    )
    .unwrap()
});
