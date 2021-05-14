mod bcs;
mod debug;
mod event;
mod hash;
mod signer;
mod vector;

pub use bcs::*;
pub use debug::*;
pub use event::*;
pub use event::*;
pub use hash::*;
pub use signer::*;
pub use vector::*;

pub fn all_natives<N>() -> Vec<(&'static str, &'static str, N)>
where
    N: From<NativeSignerBorrowAddress>
        + From<NativeVectorLength>
        + From<NativeVectorEmpty>
        + From<NativeVectorBorrow>
        + From<NativeVectorPushBack>
        + From<NativeVectorPopBack>
        + From<NativeVectorDestroyEmpty>
        + From<NativeVectorSwap>
        + From<NativeBCSToBytes>
        + From<NativeHashSha2_256>
        + From<NativeHashSha3_256>
        + From<NativeEventWriteToEventStore>,
{
    vec![
        ("Signer", "borrow_address", NativeSignerBorrowAddress.into()),
        ("Vector", "length", NativeVectorLength.into()),
        ("Vector", "empty", NativeVectorEmpty.into()),
        ("Vector", "borrow", NativeVectorBorrow.into()),
        ("Vector", "borrow_mut", NativeVectorBorrow.into()),
        ("Vector", "push_back", NativeVectorPushBack.into()),
        ("Vector", "pop_back", NativeVectorPopBack.into()),
        ("Vector", "destroy_empty", NativeVectorDestroyEmpty.into()),
        ("Vector", "swap", NativeVectorSwap.into()),
        ("BCS", "to_bytes", NativeBCSToBytes.into()),
        ("Hash", "sha2_256", NativeHashSha2_256.into()),
        ("Hash", "sha3_256", NativeHashSha3_256.into()),
        (
            "Event",
            "write_to_event_store",
            NativeEventWriteToEventStore.into(),
        ),
    ]
}
