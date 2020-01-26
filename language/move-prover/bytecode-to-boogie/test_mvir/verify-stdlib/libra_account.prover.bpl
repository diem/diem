// Native functions and helpers for libra_account

// TODO: fill in implementation

procedure {:inline 1} LibraAccount_save_account (arg0: Value, arg1: Value) returns ();
  requires ExistsTxnSenderAccount(__m, __txn);
  ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(arg0)));
  ensures !__abort_flag ==> b#Boolean(Boolean((Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(arg0)))) == (arg1)));
  ensures old(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(arg0))))) ==> !__abort_flag;
  ensures old(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(arg0)))) ==> __abort_flag;

procedure {:inline 1} LibraAccount_write_to_event_store (tv0: TypeValue, arg0: Value, arg1: Value, arg2: Value) returns ();
  requires ExistsTxnSenderAccount(__m, __txn);
  ensures old(b#Boolean(Boolean(true))) ==> !__abort_flag;

function {:inline 1} max_balance(): Value {
  Integer(9223372036854775807)
}
