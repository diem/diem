// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{ffi::CString, ptr};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

/// Tell jemalloc to dump the heap memory profile. This only works if
/// the binary is started with environment variable
///
///   MALLOC_CONF="prof:true,prof_prefix:jeprof.out"
///
/// Calling this function will cause jemalloc to write the memory
/// profile to the current working directory. Then one can process the
/// heap profile to various format with the 'jeprof' utility. ie
///
///  jeprof --pdf target/debug/libra_node jeprof.out.2141437.2.m2.heap > out.pdf
///
/// Returns the error code coming out of jemalloc if heap dump fails.
pub fn dump_jemalloc_memory_profile() -> Result<(), i32> {
    let opt_name = CString::new("prof.dump").expect("CString::new failed.");
    unsafe {
        let err_code = jemalloc_sys::mallctl(
            opt_name.as_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
        );
        if err_code == 0 {
            Ok(())
        } else {
            Err(err_code)
        }
    }
}
