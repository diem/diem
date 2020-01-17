// Native functions and helpers for bytearray_util

// TODO: fill in implementation
procedure {:inline 1} BytearrayUtil_bytearray_concat (arg0: Value, arg1: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(m, txn);
ensures old(b#Boolean(Boolean(true))) ==> !abort_flag;
