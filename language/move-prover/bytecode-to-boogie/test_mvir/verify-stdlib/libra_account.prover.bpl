// Native functions and helpers for libra_account

// TODO: fill in implementation

procedure {:inline 1} LibraAccount_save_account (arg0: Value, arg1: Value) returns ();
  requires ExistsTxnSenderAccount(m, txn);
  ensures !abort_flag ==> b#Boolean(ExistsResource(m, LibraAccount_T_type_value(), a#Address(arg0)));
  ensures !abort_flag ==> b#Boolean(Boolean((Dereference(m, GetResourceReference(LibraAccount_T_type_value(), a#Address(arg0)))) == (arg1)));
  ensures old(!(b#Boolean(ExistsResource(m, LibraAccount_T_type_value(), a#Address(arg0))))) ==> !abort_flag;
  ensures old(b#Boolean(ExistsResource(m, LibraAccount_T_type_value(), a#Address(arg0)))) ==> abort_flag;

procedure {:inline 1} LibraAccount_write_to_event_store (tv0: TypeValue, arg0: Value, arg1: Value, arg2: Value) returns ();
  requires ExistsTxnSenderAccount(m, txn);
  ensures old(b#Boolean(Boolean(true))) ==> !abort_flag;
