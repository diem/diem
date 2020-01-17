// Native functions and helpers for address_util

// TODO: fill in implementation
procedure {:inline 1} AddressUtil_address_to_bytes (arg0: Value) returns (ret0: Value);
  requires ExistsTxnSenderAccount(m, txn);
  ensures old(b#Boolean(Boolean(true))) ==> !abort_flag;
