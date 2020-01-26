// Native functions and helpers for u64_util

// TODO: fill in implementation
procedure {:inline 1} U64Util_u64_to_bytes (arg0: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(b#Boolean(Boolean(true))) ==> !__abort_flag;
