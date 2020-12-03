// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Wrapper structs for types that need [RustCrypto](https://github.com/RustCrypto)
//! traits implemented.

use digest::{
    consts::{U136, U32},
    generic_array::GenericArray,
    BlockInput, Digest, FixedOutput, Reset, Update,
};
use tiny_keccak::{Hasher, Sha3};

/// A wrapper for [`tiny_keccak::Sha3::v256`] that
/// implements RustCrypto [`digest`] traits [`BlockInput`], [`Update`], [`Reset`],
/// and [`FixedOutput`]. Consequently, this wrapper can be used in RustCrypto
/// APIs that require a hash function (usually something that impls [`Digest`]).
#[derive(Clone)]
pub struct Sha3_256(Sha3);

// ensure that we impl all of the sub-traits required for the Digest trait alias
static_assertions::assert_impl_all!(Sha3_256: Digest);

impl Default for Sha3_256 {
    #[inline]
    fn default() -> Self {
        Self(Sha3::v256())
    }
}

impl BlockInput for Sha3_256 {
    type BlockSize = U136;
}

impl Update for Sha3_256 {
    #[inline]
    fn update(&mut self, data: impl AsRef<[u8]>) {
        self.0.update(data.as_ref());
    }
}

impl Reset for Sha3_256 {
    #[inline]
    fn reset(&mut self) {
        *self = Self::default();
    }
}

impl FixedOutput for Sha3_256 {
    type OutputSize = U32;

    #[inline]
    fn finalize_into(self, out: &mut GenericArray<u8, Self::OutputSize>) {
        self.0.finalize(out.as_mut());
    }

    #[inline]
    fn finalize_into_reset(&mut self, out: &mut GenericArray<u8, Self::OutputSize>) {
        self.clone().finalize_into(out);
        Reset::reset(self)
    }
}
