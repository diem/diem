// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

/// A fallible wrapper around [`std::vec::Vec::copy_from_slice`]
pub fn copy_slice_to_vec<T>(slice: &[T], vec: &mut [T]) -> Result<(), CopySliceError>
where
    T: Copy,
{
    if slice.len() != vec.len() {
        return Err(CopySliceError);
    }

    vec.copy_from_slice(slice);

    Ok(())
}

#[derive(Error, Debug)]
#[error("can't copy source slice into destination slice: sizes don't match")]
pub struct CopySliceError;
