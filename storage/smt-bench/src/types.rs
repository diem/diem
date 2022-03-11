use diem_crypto::{
    hash::{CryptoHash, TestOnlyHasher},
    HashValue,
};
use diem_types::proof::SparseMerkleProof;
use scratchpad::{FrozenSparseMerkleTree, SparseMerkleTree};

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct DummyValue(pub HashValue);

impl CryptoHash for DummyValue {
    type Hasher = TestOnlyHasher;

    fn hash(&self) -> HashValue {
        self.0
    }
}

pub type Kv = (HashValue, DummyValue);

pub type Smt = SparseMerkleTree<DummyValue>;
pub type FrozenSmt = FrozenSparseMerkleTree<DummyValue>;
pub type Proof = SparseMerkleProof<DummyValue>;
