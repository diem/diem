//! This file contains models of the vm crate's dependencies for use with MIRAI.

pub mod types {
    pub mod transaction {
        pub const MAX_TRANSACTION_SIZE_IN_BYTES: usize = 4096;
    }
}
