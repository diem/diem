// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;
use std::cmp::min;
use std::os::raw::c_char;

const MAX_ERROR_LENGTH: usize = 1024;

thread_local! {
    static LAST_ERROR: RefCell<Box<[u8; MAX_ERROR_LENGTH]>> = RefCell::new(Box::new([0u8; MAX_ERROR_LENGTH]));
}

/// Update the most recent error.
pub fn update_last_error(err: String) {
    LAST_ERROR.with(|prev| {
        let mut last_error = prev.borrow_mut();
        let slice_str = err.into_bytes().into_boxed_slice();
        let min_error_length = min(slice_str.len(), MAX_ERROR_LENGTH - 1);
        last_error[..min_error_length].copy_from_slice(&slice_str[..min_error_length]);
        // null terminate the string
        last_error[min_error_length] = 0;
    });
}

/// Clear the most recent error.
pub fn clear_error() {
    LAST_ERROR.with(|prev| {
        let mut last_error = prev.borrow_mut();
        last_error[0] = 0;
    });
}

#[no_mangle]
/// Return the most recent error string.
///
/// Errors are thread local, so this function must be called on the same
/// thread that called the API function which got an error. The returned
/// string pointer is only valid until the next API call. Callers should not
/// store it or use it past then.
pub unsafe extern "C" fn libra_strerror() -> *const c_char {
    LAST_ERROR.with(|prev| prev.borrow().as_ptr().cast())
}
