address 0x1 {
/// An error raised from a module is represented as `MODULE_ERROR_BASE +
/// ERROR_CODE`.
/// `MODULE_ERROR_BASE`'s are segmented into 1000-block ranges. The first
/// `0-255` numbers of this range are reserved for errors coming from `CoreErrors`
/// (this module). Module-specific `ERROR_CODE`s are then defined from `256` on-up.
/// The error codes in this module must remain in this reserved `0-255`
/// range.
/// As an example, `SEQ_NUM_TOO_NEW` raised from a module with
/// `MODULE_ERROR_BASE = 1000` would be `1011`, additional errors not using
/// errors codes defined in this module would start at `1256`.
module CoreErrors {
    public fun ACCOUNT_DNE(): u64 { 0 }
    public fun INSUFFICIENT_PRIVILEGE(): u64 { 1 }
    public fun INVALID_MONETARY_VALUE(): u64 { 2 }
    public fun INVALID_SIGNATURE(): u64 { 3 }
    public fun INVALID_SINGLETON_ADDRESS(): u64 { 4 }
    public fun NOT_GENESIS(): u64 { 5 }
    public fun CONFIG_DNE(): u64 { 6 }
    public fun INVALID_AUTH_KEY(): u64 { 7 }
    public fun PRIVILEGE_ALREADY_EXTRACTED(): u64 { 8 }
    public fun INVALID_TIMESTAMP(): u64 { 9 }
    public fun SEQ_NUM_TOO_OLD(): u64 { 10 }
    public fun SEQ_NUM_TOO_NEW(): u64 { 11 }

    public fun CORE_ERR_RANGE(): u64 { 255 }
}
}
