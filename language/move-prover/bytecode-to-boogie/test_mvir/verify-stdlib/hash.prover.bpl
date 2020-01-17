// Native functions and helpers for hash

// TODO: fill in implementation
procedure {:inline 1} Hash_sha2_256 (arg0: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(m, txn);
ensures old(b#Boolean(Boolean(true))) ==> !abort_flag;

procedure {:inline 1} Hash_sha3_256 (arg0: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(m, txn);
ensures old(b#Boolean(Boolean(true))) ==> !abort_flag;
