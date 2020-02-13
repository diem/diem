
// ** helpers from test_mvir/verify-stdlib/hash.prover.bpl
// Native functions and helpers for hash

// TODO: fill in implementation
procedure {:inline 1} Hash_sha2_256 (arg0: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(b#Boolean(Boolean(true))) ==> !__abort_flag;

procedure {:inline 1} Hash_sha3_256 (arg0: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(b#Boolean(Boolean(true))) ==> !__abort_flag;

// ** helpers from test_mvir/verify-stdlib/u64_util.prover.bpl
// Native functions and helpers for u64_util

// TODO: fill in implementation
procedure {:inline 1} U64Util_u64_to_bytes (arg0: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(b#Boolean(Boolean(true))) ==> !__abort_flag;

// ** helpers from test_mvir/verify-stdlib/address_util.prover.bpl
// Native functions and helpers for address_util

// TODO: fill in implementation
procedure {:inline 1} AddressUtil_address_to_bytes (arg0: Value) returns (ret0: Value);
  requires ExistsTxnSenderAccount(__m, __txn);
  ensures old(b#Boolean(Boolean(true))) ==> !__abort_flag;

// ** helpers from test_mvir/verify-stdlib/bytearray_util.prover.bpl
// Native functions and helpers for bytearray_util

// TODO: fill in implementation
procedure {:inline 1} BytearrayUtil_bytearray_concat (arg0: Value, arg1: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(b#Boolean(Boolean(true))) ==> !__abort_flag;

// ** helpers from test_mvir/verify-stdlib/libra_account.prover.bpl
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


// ** synthetics of module LibraCoin



// ** structs of module LibraCoin

const unique LibraCoin_T: TypeName;
const LibraCoin_T_value: FieldName;
axiom LibraCoin_T_value == 0;
function LibraCoin_T_type_value(): TypeValue {
    StructType(LibraCoin_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
function {:inline 1} $LibraCoin_T_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, LibraCoin_T_value))
}

procedure {:inline 1} Pack_LibraCoin_T(module_idx: int, func_idx: int, var_idx: int, code_idx: int, value: Value) returns (_struct: Value)
{
    assume IsValidU64(value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, value));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraCoin_T(_struct: Value) returns (value: Value)
{
    assume is#Vector(_struct);
    value := SelectField(_struct, LibraCoin_T_value);
    assume IsValidU64(value);
}

const unique LibraCoin_MintCapability: TypeName;
const LibraCoin_MintCapability__dummy: FieldName;
axiom LibraCoin_MintCapability__dummy == 0;
function LibraCoin_MintCapability_type_value(): TypeValue {
    StructType(LibraCoin_MintCapability, ExtendTypeValueArray(EmptyTypeValueArray, BooleanType()))
}
function {:inline 1} $LibraCoin_MintCapability_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && is#Boolean(SelectField(__this, LibraCoin_MintCapability__dummy))
}

procedure {:inline 1} Pack_LibraCoin_MintCapability(module_idx: int, func_idx: int, var_idx: int, code_idx: int, _dummy: Value) returns (_struct: Value)
{
    assume is#Boolean(_dummy);
    _struct := Vector(ExtendValueArray(EmptyValueArray, _dummy));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraCoin_MintCapability(_struct: Value) returns (_dummy: Value)
{
    assume is#Vector(_struct);
    _dummy := SelectField(_struct, LibraCoin_MintCapability__dummy);
    assume is#Boolean(_dummy);
}

const unique LibraCoin_MarketCap: TypeName;
const LibraCoin_MarketCap_total_value: FieldName;
axiom LibraCoin_MarketCap_total_value == 0;
function LibraCoin_MarketCap_type_value(): TypeValue {
    StructType(LibraCoin_MarketCap, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
function {:inline 1} $LibraCoin_MarketCap_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, LibraCoin_MarketCap_total_value))
}

procedure {:inline 1} Pack_LibraCoin_MarketCap(module_idx: int, func_idx: int, var_idx: int, code_idx: int, total_value: Value) returns (_struct: Value)
{
    assume IsValidU64(total_value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, total_value));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraCoin_MarketCap(_struct: Value) returns (total_value: Value)
{
    assume is#Vector(_struct);
    total_value := SelectField(_struct, LibraCoin_MarketCap_total_value);
    assume IsValidU64(total_value);
}



// ** functions of module LibraCoin

procedure {:inline 1} LibraCoin_mint_with_default_capability (amount: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value), Integer(i#Integer(old(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))) + i#Integer(amount)))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), amount)));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(amount) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(amount) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(amount) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(amount) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Value; // IntegerType()
    var __t2: Value; // AddressType()
    var __t3: Reference; // ReferenceType(LibraCoin_MintCapability_type_value())
    var __t4: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 0, amount);
    assume $DebugTrackLocal(0, 0, 0, 806, amount);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraCoin_MintCapability_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 0, 1432);
      goto Label_Abort;
    }

    call __t4 := LibraCoin_mint(GetLocal(__m, __frame + 1), __t3);
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 0, 1408);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(0, 0, 1, 1401, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_mint_with_default_capability_verify (amount: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_mint_with_default_capability(amount);
}

procedure {:inline 1} LibraCoin_mint (value: Value, capability: Reference) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value), Integer(i#Integer(old(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))) + i#Integer(value)))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), value)));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(value) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(value) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(value) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(value) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var market_cap_ref: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var market_cap_total_value: Value; // IntegerType()
    var __t4: Reference; // ReferenceType(LibraCoin_MintCapability_type_value())
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // BooleanType()
    var __t10: Value; // BooleanType()
    var __t11: Value; // IntegerType()
    var __t12: Value; // AddressType()
    var __t13: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var __t14: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var __t15: Reference; // ReferenceType(IntegerType())
    var __t16: Value; // IntegerType()
    var __t17: Value; // IntegerType()
    var __t18: Value; // IntegerType()
    var __t19: Value; // IntegerType()
    var __t20: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var __t21: Reference; // ReferenceType(IntegerType())
    var __t22: Value; // IntegerType()
    var __t23: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU64(value);
    __m := UpdateLocal(__m, __frame + 0, value);
    assume $DebugTrackLocal(0, 1, 0, 1723, value);
    assume $LibraCoin_MintCapability_is_well_formed(Dereference(__m, capability)) && IsValidReferenceParameter(__m, __local_counter, capability);
    assume $LibraCoin_MintCapability_is_well_formed(Dereference(__m, capability));
    assume $DebugTrackLocal(0, 1, 1, 1723, Dereference(__m, capability));

    // increase the local counter
    __local_counter := __local_counter + 24;

    // bytecode translation starts here
    call __t4 := CopyOrMoveRef(capability);

    // unimplemented instruction: NoOp

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(1000000000);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := LdConst(1000000);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 1, 2762);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Le(GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __tmp := GetLocal(__m, __frame + 10);
    if (!b#Boolean(__tmp)) { goto Label_11; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    if (true) { assume $DebugTrackAbort(0, 1, 2784); }
    goto Label_Abort;

Label_11:
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __t13 := BorrowGlobal(GetLocal(__m, __frame + 12), LibraCoin_MarketCap_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 1, 2915);
      goto Label_Abort;
    }

    call market_cap_ref := CopyOrMoveRef(__t13);
    assume $LibraCoin_MarketCap_is_well_formed(Dereference(__m, market_cap_ref));
    assume $DebugTrackLocal(0, 1, 2, 2898, Dereference(__m, market_cap_ref));

    call __t14 := CopyOrMoveRef(market_cap_ref);

    call __t15 := BorrowField(__t14, LibraCoin_MarketCap_total_value);

    call __tmp := ReadRef(__t15);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(0, 1, 3, 2964, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 17), GetLocal(__m, __frame + 18));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 1, 3076);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __t20 := CopyOrMoveRef(market_cap_ref);

    call __t21 := BorrowField(__t20, LibraCoin_MarketCap_total_value);

    call WriteRef(__t21, GetLocal(__m, __frame + 19));


    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    call __tmp := Pack_LibraCoin_T(0, 0, 0, 0, GetLocal(__m, __frame + 22));
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    __ret0 := GetLocal(__m, __frame + 23);
    assume $DebugTrackLocal(0, 1, 4, 3129, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_mint_verify (value: Value, capability: Reference) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_mint(value, capability);
}

procedure {:inline 1} LibraCoin_initialize () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn)))));
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(__txn)))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraCoin_MarketCap_total_value), Integer(0))));
ensures old(!(b#Boolean(Boolean(!IsEqual(Address(TxnSenderAddress(__txn)), Address(173345816)))) || b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn))))) || b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(__txn))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!IsEqual(Address(TxnSenderAddress(__txn)), Address(173345816)))) || b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn))))) || b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))) ==> __abort_flag;

{
    // declare local variables
    var __t0: Value; // AddressType()
    var __t1: Value; // AddressType()
    var __t2: Value; // BooleanType()
    var __t3: Value; // BooleanType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // LibraCoin_MintCapability_type_value()
    var __t7: Value; // IntegerType()
    var __t8: Value; // LibraCoin_MarketCap_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 9;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 0), GetLocal(__m, __frame + 1)));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __tmp := GetLocal(__m, __frame + 3);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    if (true) { assume $DebugTrackAbort(0, 2, 3875); }
    goto Label_Abort;

Label_7:
    call __tmp := LdTrue();
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Pack_LibraCoin_MintCapability(0, 0, 0, 0, GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call MoveToSender(LibraCoin_MintCapability_type_value(), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 2, 3888);
      goto Label_Abort;
    }

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := Pack_LibraCoin_MarketCap(0, 0, 0, 0, GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call MoveToSender(LibraCoin_MarketCap_type_value(), GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 2, 3960);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraCoin_initialize_verify () returns ()
{
    call InitVerification();
    call LibraCoin_initialize();
}

procedure {:inline 1} LibraCoin_market_cap () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(__ret0, SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))))))) ==> __abort_flag;

{
    // declare local variables
    var __t0: Value; // AddressType()
    var __t1: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var __t2: Reference; // ReferenceType(IntegerType())
    var __t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t1 := BorrowGlobal(GetLocal(__m, __frame + 0), LibraCoin_MarketCap_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 3, 4299);
      goto Label_Abort;
    }

    call __t2 := BorrowField(__t1, LibraCoin_MarketCap_total_value);

    call __tmp := ReadRef(__t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(0, 3, 0, 4289, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_market_cap_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_market_cap();
}

procedure {:inline 1} LibraCoin_zero () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), Integer(0))));
{
    // declare local variables
    var __t0: Value; // IntegerType()
    var __t1: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 2;

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := Pack_LibraCoin_T(0, 0, 0, 0, GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    assume $DebugTrackLocal(0, 4, 0, 4477, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_zero_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_zero();
}

procedure {:inline 1} LibraCoin_value (coin_ref: Reference) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, SelectField(Dereference(__m, coin_ref), LibraCoin_T_value))));
{
    // declare local variables
    var __t1: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t2: Reference; // ReferenceType(IntegerType())
    var __t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraCoin_T_is_well_formed(Dereference(__m, coin_ref)) && IsValidReferenceParameter(__m, __local_counter, coin_ref);
    assume $LibraCoin_T_is_well_formed(Dereference(__m, coin_ref));
    assume $DebugTrackLocal(0, 5, 0, 4555, Dereference(__m, coin_ref));

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(coin_ref);

    call __t2 := BorrowField(__t1, LibraCoin_T_value);

    call __tmp := ReadRef(__t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(0, 5, 1, 4644, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_value_verify (coin_ref: Reference) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_value(coin_ref);
}

procedure {:inline 1} LibraCoin_split (coin: Value, amount: Value) returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(b#Boolean(Boolean(IsEqual(SelectField(__ret1, LibraCoin_T_value), amount))) && b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), Integer(i#Integer(old(SelectField(coin, LibraCoin_T_value))) - i#Integer(amount)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(SelectField(coin, LibraCoin_T_value)) < i#Integer(amount))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(SelectField(coin, LibraCoin_T_value)) < i#Integer(amount)))) ==> __abort_flag;

{
    // declare local variables
    var other: Value; // LibraCoin_T_type_value()
    var __t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t4: Value; // IntegerType()
    var __t5: Value; // LibraCoin_T_type_value()
    var __t6: Value; // LibraCoin_T_type_value()
    var __t7: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraCoin_T_is_well_formed(coin);
    __m := UpdateLocal(__m, __frame + 0, coin);
    assume $DebugTrackLocal(0, 6, 0, 4818, coin);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(0, 6, 1, 4818, amount);

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __t3 := BorrowLoc(__frame + 0, LibraCoin_T_type_value());

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __t5 := LibraCoin_withdraw(__t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 6, 5045);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t5);

    __m := UpdateLocal(__m, __frame + 5, __t5);
    assume $DebugTrackLocal(0, 6, 0, 5045, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(0, 6, 2, 5037, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __ret0 := GetLocal(__m, __frame + 6);
    assume $DebugTrackLocal(0, 6, 3, 5093, __ret0);
    __ret1 := GetLocal(__m, __frame + 7);
    assume $DebugTrackLocal(0, 6, 4, 5093, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure LibraCoin_split_verify (coin: Value, amount: Value) returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := LibraCoin_split(coin, amount);
}

procedure {:inline 1} LibraCoin_withdraw (coin_ref: Reference, amount: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value), Integer(i#Integer(old(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value))) - i#Integer(amount)))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), amount)));
ensures old(!(b#Boolean(Boolean(i#Integer(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) < i#Integer(amount))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) < i#Integer(amount)))) ==> __abort_flag;

{
    // declare local variables
    var value: Value; // IntegerType()
    var __t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t4: Reference; // ReferenceType(IntegerType())
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // BooleanType()
    var __t9: Value; // BooleanType()
    var __t10: Value; // IntegerType()
    var __t11: Value; // IntegerType()
    var __t12: Value; // IntegerType()
    var __t13: Value; // IntegerType()
    var __t14: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t15: Reference; // ReferenceType(IntegerType())
    var __t16: Value; // IntegerType()
    var __t17: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraCoin_T_is_well_formed(Dereference(__m, coin_ref)) && IsValidReferenceParameter(__m, __local_counter, coin_ref);
    assume $LibraCoin_T_is_well_formed(Dereference(__m, coin_ref));
    assume $DebugTrackLocal(0, 7, 0, 5391, Dereference(__m, coin_ref));
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(0, 7, 1, 5391, amount);

    // increase the local counter
    __local_counter := __local_counter + 18;

    // bytecode translation starts here
    call __t3 := CopyOrMoveRef(coin_ref);

    call __t4 := BorrowField(__t3, LibraCoin_T_value);

    call __tmp := ReadRef(__t4);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(0, 7, 2, 5692, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := Ge(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __tmp := GetLocal(__m, __frame + 9);
    if (!b#Boolean(__tmp)) { goto Label_11; }

    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    if (true) { assume $DebugTrackAbort(0, 7, 5774); }
    goto Label_Abort;

Label_11:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 12));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 7, 5845);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __t14 := CopyOrMoveRef(coin_ref);

    call __t15 := BorrowField(__t14, LibraCoin_T_value);

    call WriteRef(__t15, GetLocal(__m, __frame + 13));
    assume $LibraCoin_T_is_well_formed(Dereference(__m, coin_ref));
    assume $DebugTrackLocal(0, 7, 0, 5814, Dereference(__m, coin_ref));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := Pack_LibraCoin_T(0, 0, 0, 0, GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    assume $DebugTrackLocal(0, 7, 3, 5881, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_withdraw_verify (coin_ref: Reference, amount: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_withdraw(coin_ref, amount);
}

procedure {:inline 1} LibraCoin_join (coin1: Value, coin2: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), Integer(i#Integer(old(SelectField(coin1, LibraCoin_T_value))) + i#Integer(old(SelectField(coin2, LibraCoin_T_value)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(coin1, LibraCoin_T_value)) + i#Integer(SelectField(coin2, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(coin1, LibraCoin_T_value)) + i#Integer(SelectField(coin2, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t3: Value; // LibraCoin_T_type_value()
    var __t4: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraCoin_T_is_well_formed(coin1);
    __m := UpdateLocal(__m, __frame + 0, coin1);
    assume $DebugTrackLocal(0, 8, 0, 6020, coin1);
    assume $LibraCoin_T_is_well_formed(coin2);
    __m := UpdateLocal(__m, __frame + 1, coin2);
    assume $DebugTrackLocal(0, 8, 1, 6020, coin2);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __t2 := BorrowLoc(__frame + 0, LibraCoin_T_type_value());

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call LibraCoin_deposit(__t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 8, 6215);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(0, 8, 0, 6215, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(0, 8, 2, 6262, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_join_verify (coin1: Value, coin2: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_join(coin1, coin2);
}

procedure {:inline 1} LibraCoin_deposit (coin_ref: Reference, check: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value), Integer(i#Integer(old(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value))) + i#Integer(old(SelectField(check, LibraCoin_T_value)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) + i#Integer(SelectField(check, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) + i#Integer(SelectField(check, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var value: Value; // IntegerType()
    var check_value: Value; // IntegerType()
    var __t4: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t5: Reference; // ReferenceType(IntegerType())
    var __t6: Value; // IntegerType()
    var __t7: Value; // LibraCoin_T_type_value()
    var __t8: Value; // IntegerType()
    var __t9: Value; // IntegerType()
    var __t10: Value; // IntegerType()
    var __t11: Value; // IntegerType()
    var __t12: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t13: Reference; // ReferenceType(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraCoin_T_is_well_formed(Dereference(__m, coin_ref)) && IsValidReferenceParameter(__m, __local_counter, coin_ref);
    assume $LibraCoin_T_is_well_formed(Dereference(__m, coin_ref));
    assume $DebugTrackLocal(0, 9, 0, 6465, Dereference(__m, coin_ref));
    assume $LibraCoin_T_is_well_formed(check);
    __m := UpdateLocal(__m, __frame + 1, check);
    assume $DebugTrackLocal(0, 9, 1, 6465, check);

    // increase the local counter
    __local_counter := __local_counter + 14;

    // bytecode translation starts here
    call __t4 := CopyOrMoveRef(coin_ref);

    call __t5 := BorrowField(__t4, LibraCoin_T_value);

    call __tmp := ReadRef(__t5);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(0, 9, 2, 6729, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := Unpack_LibraCoin_T(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(0, 9, 3, 6779, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 9, 6853);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := CopyOrMoveRef(coin_ref);

    call __t13 := BorrowField(__t12, LibraCoin_T_value);

    call WriteRef(__t13, GetLocal(__m, __frame + 11));
    assume $LibraCoin_T_is_well_formed(Dereference(__m, coin_ref));
    assume $DebugTrackLocal(0, 9, 0, 6823, Dereference(__m, coin_ref));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraCoin_deposit_verify (coin_ref: Reference, check: Value) returns ()
{
    call InitVerification();
    call LibraCoin_deposit(coin_ref, check);
}

procedure {:inline 1} LibraCoin_destroy_zero (coin: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(!IsEqual(SelectField(coin, LibraCoin_T_value), Integer(0)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!IsEqual(SelectField(coin, LibraCoin_T_value), Integer(0))))) ==> __abort_flag;

{
    // declare local variables
    var value: Value; // IntegerType()
    var __t2: Value; // LibraCoin_T_type_value()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // BooleanType()
    var __t7: Value; // BooleanType()
    var __t8: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraCoin_T_is_well_formed(coin);
    __m := UpdateLocal(__m, __frame + 0, coin);
    assume $DebugTrackLocal(0, 10, 0, 7117, coin);

    // increase the local counter
    __local_counter := __local_counter + 9;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := Unpack_LibraCoin_T(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(0, 10, 1, 7227, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5)));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __tmp := GetLocal(__m, __frame + 7);
    if (!b#Boolean(__tmp)) { goto Label_10; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    if (true) { assume $DebugTrackAbort(0, 10, 7282); }
    goto Label_Abort;

Label_10:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraCoin_destroy_zero_verify (coin: Value) returns ()
{
    call InitVerification();
    call LibraCoin_destroy_zero(coin);
}



// ** synthetics of module Hash



// ** structs of module Hash



// ** functions of module Hash



// ** synthetics of module U64Util



// ** structs of module U64Util



// ** functions of module U64Util



// ** synthetics of module AddressUtil



// ** structs of module AddressUtil



// ** functions of module AddressUtil



// ** synthetics of module BytearrayUtil



// ** structs of module BytearrayUtil



// ** functions of module BytearrayUtil



// ** synthetics of module LibraAccount



// ** structs of module LibraAccount

const unique LibraAccount_T: TypeName;
const LibraAccount_T_authentication_key: FieldName;
axiom LibraAccount_T_authentication_key == 0;
const LibraAccount_T_balance: FieldName;
axiom LibraAccount_T_balance == 1;
const LibraAccount_T_delegated_key_rotation_capability: FieldName;
axiom LibraAccount_T_delegated_key_rotation_capability == 2;
const LibraAccount_T_delegated_withdrawal_capability: FieldName;
axiom LibraAccount_T_delegated_withdrawal_capability == 3;
const LibraAccount_T_received_events: FieldName;
axiom LibraAccount_T_received_events == 4;
const LibraAccount_T_sent_events: FieldName;
axiom LibraAccount_T_sent_events == 5;
const LibraAccount_T_sequence_number: FieldName;
axiom LibraAccount_T_sequence_number == 6;
const LibraAccount_T_event_generator: FieldName;
axiom LibraAccount_T_event_generator == 7;
axiom LibraAccount_T_type_value() == StructType(LibraAccount_T, ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, ByteArrayType()), LibraCoin_T_type_value()), BooleanType()), BooleanType()), LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())), LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())), IntegerType()), LibraAccount_EventHandleGenerator_type_value()));
function {:inline 1} $LibraAccount_T_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && is#ByteArray(SelectField(__this, LibraAccount_T_authentication_key))
        && $LibraCoin_T_is_well_formed(SelectField(__this, LibraAccount_T_balance))
        && is#Boolean(SelectField(__this, LibraAccount_T_delegated_key_rotation_capability))
        && is#Boolean(SelectField(__this, LibraAccount_T_delegated_withdrawal_capability))
        && $LibraAccount_EventHandle_is_well_formed(SelectField(__this, LibraAccount_T_received_events))
        && $LibraAccount_EventHandle_is_well_formed(SelectField(__this, LibraAccount_T_sent_events))
        && IsValidU64(SelectField(__this, LibraAccount_T_sequence_number))
        && $LibraAccount_EventHandleGenerator_is_well_formed(SelectField(__this, LibraAccount_T_event_generator))
}

procedure {:inline 1} Pack_LibraAccount_T(module_idx: int, func_idx: int, var_idx: int, code_idx: int, authentication_key: Value, balance: Value, delegated_key_rotation_capability: Value, delegated_withdrawal_capability: Value, received_events: Value, sent_events: Value, sequence_number: Value, event_generator: Value) returns (_struct: Value)
{
    assume is#ByteArray(authentication_key);
    assume $LibraCoin_T_is_well_formed(balance);
    assume is#Boolean(delegated_key_rotation_capability);
    assume is#Boolean(delegated_withdrawal_capability);
    assume $LibraAccount_EventHandle_is_well_formed(received_events);
    assume $LibraAccount_EventHandle_is_well_formed(sent_events);
    assume IsValidU64(sequence_number);
    assume $LibraAccount_EventHandleGenerator_is_well_formed(event_generator);
    _struct := Vector(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(EmptyValueArray, authentication_key), balance), delegated_key_rotation_capability), delegated_withdrawal_capability), received_events), sent_events), sequence_number), event_generator));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraAccount_T(_struct: Value) returns (authentication_key: Value, balance: Value, delegated_key_rotation_capability: Value, delegated_withdrawal_capability: Value, received_events: Value, sent_events: Value, sequence_number: Value, event_generator: Value)
{
    assume is#Vector(_struct);
    authentication_key := SelectField(_struct, LibraAccount_T_authentication_key);
    assume is#ByteArray(authentication_key);
    balance := SelectField(_struct, LibraAccount_T_balance);
    assume $LibraCoin_T_is_well_formed(balance);
    delegated_key_rotation_capability := SelectField(_struct, LibraAccount_T_delegated_key_rotation_capability);
    assume is#Boolean(delegated_key_rotation_capability);
    delegated_withdrawal_capability := SelectField(_struct, LibraAccount_T_delegated_withdrawal_capability);
    assume is#Boolean(delegated_withdrawal_capability);
    received_events := SelectField(_struct, LibraAccount_T_received_events);
    assume $LibraAccount_EventHandle_is_well_formed(received_events);
    sent_events := SelectField(_struct, LibraAccount_T_sent_events);
    assume $LibraAccount_EventHandle_is_well_formed(sent_events);
    sequence_number := SelectField(_struct, LibraAccount_T_sequence_number);
    assume IsValidU64(sequence_number);
    event_generator := SelectField(_struct, LibraAccount_T_event_generator);
    assume $LibraAccount_EventHandleGenerator_is_well_formed(event_generator);
}

const unique LibraAccount_WithdrawalCapability: TypeName;
const LibraAccount_WithdrawalCapability_account_address: FieldName;
axiom LibraAccount_WithdrawalCapability_account_address == 0;
function LibraAccount_WithdrawalCapability_type_value(): TypeValue {
    StructType(LibraAccount_WithdrawalCapability, ExtendTypeValueArray(EmptyTypeValueArray, AddressType()))
}
function {:inline 1} $LibraAccount_WithdrawalCapability_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && is#Address(SelectField(__this, LibraAccount_WithdrawalCapability_account_address))
}

procedure {:inline 1} Pack_LibraAccount_WithdrawalCapability(module_idx: int, func_idx: int, var_idx: int, code_idx: int, account_address: Value) returns (_struct: Value)
{
    assume is#Address(account_address);
    _struct := Vector(ExtendValueArray(EmptyValueArray, account_address));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraAccount_WithdrawalCapability(_struct: Value) returns (account_address: Value)
{
    assume is#Vector(_struct);
    account_address := SelectField(_struct, LibraAccount_WithdrawalCapability_account_address);
    assume is#Address(account_address);
}

const unique LibraAccount_KeyRotationCapability: TypeName;
const LibraAccount_KeyRotationCapability_account_address: FieldName;
axiom LibraAccount_KeyRotationCapability_account_address == 0;
function LibraAccount_KeyRotationCapability_type_value(): TypeValue {
    StructType(LibraAccount_KeyRotationCapability, ExtendTypeValueArray(EmptyTypeValueArray, AddressType()))
}
function {:inline 1} $LibraAccount_KeyRotationCapability_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && is#Address(SelectField(__this, LibraAccount_KeyRotationCapability_account_address))
}

procedure {:inline 1} Pack_LibraAccount_KeyRotationCapability(module_idx: int, func_idx: int, var_idx: int, code_idx: int, account_address: Value) returns (_struct: Value)
{
    assume is#Address(account_address);
    _struct := Vector(ExtendValueArray(EmptyValueArray, account_address));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraAccount_KeyRotationCapability(_struct: Value) returns (account_address: Value)
{
    assume is#Vector(_struct);
    account_address := SelectField(_struct, LibraAccount_KeyRotationCapability_account_address);
    assume is#Address(account_address);
}

const unique LibraAccount_SentPaymentEvent: TypeName;
const LibraAccount_SentPaymentEvent_amount: FieldName;
axiom LibraAccount_SentPaymentEvent_amount == 0;
const LibraAccount_SentPaymentEvent_payee: FieldName;
axiom LibraAccount_SentPaymentEvent_payee == 1;
const LibraAccount_SentPaymentEvent_metadata: FieldName;
axiom LibraAccount_SentPaymentEvent_metadata == 2;
function LibraAccount_SentPaymentEvent_type_value(): TypeValue {
    StructType(LibraAccount_SentPaymentEvent, ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), AddressType()), ByteArrayType()))
}
function {:inline 1} $LibraAccount_SentPaymentEvent_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, LibraAccount_SentPaymentEvent_amount))
        && is#Address(SelectField(__this, LibraAccount_SentPaymentEvent_payee))
        && is#ByteArray(SelectField(__this, LibraAccount_SentPaymentEvent_metadata))
}

procedure {:inline 1} Pack_LibraAccount_SentPaymentEvent(module_idx: int, func_idx: int, var_idx: int, code_idx: int, amount: Value, payee: Value, metadata: Value) returns (_struct: Value)
{
    assume IsValidU64(amount);
    assume is#Address(payee);
    assume is#ByteArray(metadata);
    _struct := Vector(ExtendValueArray(ExtendValueArray(ExtendValueArray(EmptyValueArray, amount), payee), metadata));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraAccount_SentPaymentEvent(_struct: Value) returns (amount: Value, payee: Value, metadata: Value)
{
    assume is#Vector(_struct);
    amount := SelectField(_struct, LibraAccount_SentPaymentEvent_amount);
    assume IsValidU64(amount);
    payee := SelectField(_struct, LibraAccount_SentPaymentEvent_payee);
    assume is#Address(payee);
    metadata := SelectField(_struct, LibraAccount_SentPaymentEvent_metadata);
    assume is#ByteArray(metadata);
}

const unique LibraAccount_ReceivedPaymentEvent: TypeName;
const LibraAccount_ReceivedPaymentEvent_amount: FieldName;
axiom LibraAccount_ReceivedPaymentEvent_amount == 0;
const LibraAccount_ReceivedPaymentEvent_payer: FieldName;
axiom LibraAccount_ReceivedPaymentEvent_payer == 1;
const LibraAccount_ReceivedPaymentEvent_metadata: FieldName;
axiom LibraAccount_ReceivedPaymentEvent_metadata == 2;
function LibraAccount_ReceivedPaymentEvent_type_value(): TypeValue {
    StructType(LibraAccount_ReceivedPaymentEvent, ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), AddressType()), ByteArrayType()))
}
function {:inline 1} $LibraAccount_ReceivedPaymentEvent_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, LibraAccount_ReceivedPaymentEvent_amount))
        && is#Address(SelectField(__this, LibraAccount_ReceivedPaymentEvent_payer))
        && is#ByteArray(SelectField(__this, LibraAccount_ReceivedPaymentEvent_metadata))
}

procedure {:inline 1} Pack_LibraAccount_ReceivedPaymentEvent(module_idx: int, func_idx: int, var_idx: int, code_idx: int, amount: Value, payer: Value, metadata: Value) returns (_struct: Value)
{
    assume IsValidU64(amount);
    assume is#Address(payer);
    assume is#ByteArray(metadata);
    _struct := Vector(ExtendValueArray(ExtendValueArray(ExtendValueArray(EmptyValueArray, amount), payer), metadata));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraAccount_ReceivedPaymentEvent(_struct: Value) returns (amount: Value, payer: Value, metadata: Value)
{
    assume is#Vector(_struct);
    amount := SelectField(_struct, LibraAccount_ReceivedPaymentEvent_amount);
    assume IsValidU64(amount);
    payer := SelectField(_struct, LibraAccount_ReceivedPaymentEvent_payer);
    assume is#Address(payer);
    metadata := SelectField(_struct, LibraAccount_ReceivedPaymentEvent_metadata);
    assume is#ByteArray(metadata);
}

const unique LibraAccount_EventHandleGenerator: TypeName;
const LibraAccount_EventHandleGenerator_counter: FieldName;
axiom LibraAccount_EventHandleGenerator_counter == 0;
function LibraAccount_EventHandleGenerator_type_value(): TypeValue {
    StructType(LibraAccount_EventHandleGenerator, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
function {:inline 1} $LibraAccount_EventHandleGenerator_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, LibraAccount_EventHandleGenerator_counter))
}

procedure {:inline 1} Pack_LibraAccount_EventHandleGenerator(module_idx: int, func_idx: int, var_idx: int, code_idx: int, counter: Value) returns (_struct: Value)
{
    assume IsValidU64(counter);
    _struct := Vector(ExtendValueArray(EmptyValueArray, counter));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraAccount_EventHandleGenerator(_struct: Value) returns (counter: Value)
{
    assume is#Vector(_struct);
    counter := SelectField(_struct, LibraAccount_EventHandleGenerator_counter);
    assume IsValidU64(counter);
}

const unique LibraAccount_EventHandle: TypeName;
const LibraAccount_EventHandle_counter: FieldName;
axiom LibraAccount_EventHandle_counter == 0;
const LibraAccount_EventHandle_guid: FieldName;
axiom LibraAccount_EventHandle_guid == 1;
function LibraAccount_EventHandle_type_value(tv0: TypeValue): TypeValue {
    StructType(LibraAccount_EventHandle, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), ByteArrayType()))
}
function {:inline 1} $LibraAccount_EventHandle_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, LibraAccount_EventHandle_counter))
        && is#ByteArray(SelectField(__this, LibraAccount_EventHandle_guid))
}

procedure {:inline 1} Pack_LibraAccount_EventHandle(module_idx: int, func_idx: int, var_idx: int, code_idx: int, tv0: TypeValue, counter: Value, guid: Value) returns (_struct: Value)
{
    assume IsValidU64(counter);
    assume is#ByteArray(guid);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, counter), guid));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraAccount_EventHandle(_struct: Value) returns (counter: Value, guid: Value)
{
    assume is#Vector(_struct);
    counter := SelectField(_struct, LibraAccount_EventHandle_counter);
    assume IsValidU64(counter);
    guid := SelectField(_struct, LibraAccount_EventHandle_guid);
    assume is#ByteArray(guid);
}



// ** functions of module LibraAccount

procedure {:inline 1} LibraAccount_make (fresh_address: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(SelectField(SelectField(__ret0, LibraAccount_T_balance), LibraCoin_T_value), Integer(0))));
ensures b#Boolean(Boolean(b#Boolean(Boolean(!(b#Boolean(SelectField(__ret0, LibraAccount_T_delegated_key_rotation_capability))))) && b#Boolean(Boolean(!(b#Boolean(SelectField(__ret0, LibraAccount_T_delegated_withdrawal_capability)))))));
ensures b#Boolean(Boolean(b#Boolean(Boolean(IsEqual(SelectField(SelectField(__ret0, LibraAccount_T_received_events), LibraAccount_EventHandle_counter), Integer(0)))) && b#Boolean(Boolean(IsEqual(SelectField(SelectField(__ret0, LibraAccount_T_sent_events), LibraAccount_EventHandle_counter), Integer(0))))));
{
    // declare local variables
    var zero_balance: Value; // LibraCoin_T_type_value()
    var generator: Value; // LibraAccount_EventHandleGenerator_type_value()
    var sent_handle: Value; // LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())
    var received_handle: Value; // LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())
    var auth_key: Value; // ByteArrayType()
    var __t6: Value; // AddressType()
    var __t7: Value; // ByteArrayType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // LibraAccount_EventHandleGenerator_type_value()
    var __t10: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var __t11: Value; // AddressType()
    var __t12: Value; // LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())
    var __t13: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var __t14: Value; // AddressType()
    var __t15: Value; // LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())
    var __t16: Value; // LibraCoin_T_type_value()
    var __t17: Value; // ByteArrayType()
    var __t18: Value; // LibraCoin_T_type_value()
    var __t19: Value; // BooleanType()
    var __t20: Value; // BooleanType()
    var __t21: Value; // LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())
    var __t22: Value; // LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())
    var __t23: Value; // IntegerType()
    var __t24: Value; // LibraAccount_EventHandleGenerator_type_value()
    var __t25: Value; // LibraAccount_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(fresh_address);
    __m := UpdateLocal(__m, __frame + 0, fresh_address);
    assume $DebugTrackLocal(5, 0, 0, 3283, fresh_address);

    // increase the local counter
    __local_counter := __local_counter + 26;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := AddressUtil_address_to_bytes(GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 0, 3823);
      goto Label_Abort;
    }
    assume is#ByteArray(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 5, __tmp);
    assume $DebugTrackLocal(5, 0, 5, 3812, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Pack_LibraAccount_EventHandleGenerator(5, 0, 2, 3894, GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(5, 0, 2, 3882, __tmp);

    call __t10 := BorrowLoc(__frame + 2, LibraAccount_EventHandleGenerator_type_value());

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := LibraAccount_new_event_handle_impl(LibraAccount_SentPaymentEvent_type_value(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 0, 3951);
      goto Label_Abort;
    }
    assume $LibraAccount_EventHandle_is_well_formed(__t12);

    __m := UpdateLocal(__m, __frame + 12, __t12);
    assume $DebugTrackLocal(5, 0, 2, 3951, GetLocal(__m, __frame + 2));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(5, 0, 3, 3937, __tmp);

    call __t13 := BorrowLoc(__frame + 2, LibraAccount_EventHandleGenerator_type_value());

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __t15 := LibraAccount_new_event_handle_impl(LibraAccount_ReceivedPaymentEvent_type_value(), __t13, GetLocal(__m, __frame + 14));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 0, 4065);
      goto Label_Abort;
    }
    assume $LibraAccount_EventHandle_is_well_formed(__t15);

    __m := UpdateLocal(__m, __frame + 15, __t15);
    assume $DebugTrackLocal(5, 0, 2, 4065, GetLocal(__m, __frame + 2));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 15));
    __m := UpdateLocal(__m, __frame + 4, __tmp);
    assume $DebugTrackLocal(5, 0, 4, 4047, __tmp);

    call __t16 := LibraCoin_zero();
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 0, 4180);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t16);

    __m := UpdateLocal(__m, __frame + 16, __t16);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(5, 0, 1, 4165, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 24, __tmp);

    call __tmp := Pack_LibraAccount_T(0, 0, 0, 0, GetLocal(__m, __frame + 17), GetLocal(__m, __frame + 18), GetLocal(__m, __frame + 19), GetLocal(__m, __frame + 20), GetLocal(__m, __frame + 21), GetLocal(__m, __frame + 22), GetLocal(__m, __frame + 23), GetLocal(__m, __frame + 24));
    __m := UpdateLocal(__m, __frame + 25, __tmp);

    __ret0 := GetLocal(__m, __frame + 25);
    assume $DebugTrackLocal(5, 0, 6, 4207, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_make_verify (fresh_address: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_make(fresh_address);
}

procedure {:inline 1} LibraAccount_deposit (payee: Value, to_deposit: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value), Integer(i#Integer(old(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value))) + i#Integer(SelectField(to_deposit, LibraCoin_T_value))))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)))))) || b#Boolean(Boolean(IsEqual(SelectField(to_deposit, LibraCoin_T_value), Integer(0)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value)) + i#Integer(SelectField(to_deposit, LibraCoin_T_value)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_sent_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_received_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)))))) || b#Boolean(Boolean(IsEqual(SelectField(to_deposit, LibraCoin_T_value), Integer(0)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value)) + i#Integer(SelectField(to_deposit, LibraCoin_T_value)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_sent_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_received_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64())))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // AddressType()
    var __t3: Value; // LibraCoin_T_type_value()
    var __t4: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume $DebugTrackLocal(5, 1, 0, 4674, payee);
    assume $LibraCoin_T_is_well_formed(to_deposit);
    __m := UpdateLocal(__m, __frame + 1, to_deposit);
    assume $DebugTrackLocal(5, 1, 1, 4674, to_deposit);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    // unimplemented instruction: LdByteArray(4, ByteArrayPoolIndex(0))

    call LibraAccount_deposit_with_metadata(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 1, 5265);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_deposit_verify (payee: Value, to_deposit: Value) returns ()
{
    call InitVerification();
    call LibraAccount_deposit(payee, to_deposit);
}

procedure {:inline 1} LibraAccount_deposit_with_metadata (payee: Value, to_deposit: Value, metadata: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value), Integer(i#Integer(old(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value))) + i#Integer(SelectField(to_deposit, LibraCoin_T_value))))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)))))) || b#Boolean(Boolean(IsEqual(SelectField(to_deposit, LibraCoin_T_value), Integer(0)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value)) + i#Integer(SelectField(to_deposit, LibraCoin_T_value)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_sent_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_received_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)))))) || b#Boolean(Boolean(IsEqual(SelectField(to_deposit, LibraCoin_T_value), Integer(0)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value)) + i#Integer(SelectField(to_deposit, LibraCoin_T_value)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_sent_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_received_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64())))) ==> __abort_flag;

{
    // declare local variables
    var __t3: Value; // AddressType()
    var __t4: Value; // AddressType()
    var __t5: Value; // LibraCoin_T_type_value()
    var __t6: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume $DebugTrackLocal(5, 2, 0, 5450, payee);
    assume $LibraCoin_T_is_well_formed(to_deposit);
    __m := UpdateLocal(__m, __frame + 1, to_deposit);
    assume $DebugTrackLocal(5, 2, 1, 5450, to_deposit);
    assume is#ByteArray(metadata);
    __m := UpdateLocal(__m, __frame + 2, metadata);
    assume $DebugTrackLocal(5, 2, 2, 5450, metadata);

    // increase the local counter
    __local_counter := __local_counter + 7;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call LibraAccount_deposit_with_sender_and_metadata(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 2, 6106);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_deposit_with_metadata_verify (payee: Value, to_deposit: Value, metadata: Value) returns ()
{
    call InitVerification();
    call LibraAccount_deposit_with_metadata(payee, to_deposit, metadata);
}

procedure {:inline 1} LibraAccount_deposit_with_sender_and_metadata (payee: Value, sender: Value, to_deposit: Value, metadata: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value), Integer(i#Integer(old(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value))) + i#Integer(SelectField(to_deposit, LibraCoin_T_value))))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(sender)))))) || b#Boolean(Boolean(IsEqual(SelectField(to_deposit, LibraCoin_T_value), Integer(0)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value)) + i#Integer(SelectField(to_deposit, LibraCoin_T_value)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(sender))), LibraAccount_T_sent_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_received_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(sender)))))) || b#Boolean(Boolean(IsEqual(SelectField(to_deposit, LibraCoin_T_value), Integer(0)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value)) + i#Integer(SelectField(to_deposit, LibraCoin_T_value)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(sender))), LibraAccount_T_sent_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_received_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64())))) ==> __abort_flag;

{
    // declare local variables
    var deposit_value: Value; // IntegerType()
    var payee_account_ref: Reference; // ReferenceType(LibraAccount_T_type_value())
    var sender_account_ref: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t7: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t8: Value; // IntegerType()
    var __t9: Value; // IntegerType()
    var __t10: Value; // IntegerType()
    var __t11: Value; // BooleanType()
    var __t12: Value; // BooleanType()
    var __t13: Value; // IntegerType()
    var __t14: Value; // AddressType()
    var __t15: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t16: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t17: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value()))
    var __t18: Value; // IntegerType()
    var __t19: Value; // AddressType()
    var __t20: Value; // ByteArrayType()
    var __t21: Value; // LibraAccount_SentPaymentEvent_type_value()
    var __t22: Value; // AddressType()
    var __t23: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t24: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t25: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t26: Value; // LibraCoin_T_type_value()
    var __t27: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t28: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value()))
    var __t29: Value; // IntegerType()
    var __t30: Value; // AddressType()
    var __t31: Value; // ByteArrayType()
    var __t32: Value; // LibraAccount_ReceivedPaymentEvent_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume $DebugTrackLocal(5, 3, 0, 6415, payee);
    assume is#Address(sender);
    __m := UpdateLocal(__m, __frame + 1, sender);
    assume $DebugTrackLocal(5, 3, 1, 6415, sender);
    assume $LibraCoin_T_is_well_formed(to_deposit);
    __m := UpdateLocal(__m, __frame + 2, to_deposit);
    assume $DebugTrackLocal(5, 3, 2, 6415, to_deposit);
    assume is#ByteArray(metadata);
    __m := UpdateLocal(__m, __frame + 3, metadata);
    assume $DebugTrackLocal(5, 3, 3, 6415, metadata);

    // increase the local counter
    __local_counter := __local_counter + 33;

    // bytecode translation starts here
    call __t7 := BorrowLoc(__frame + 2, LibraCoin_T_type_value());

    call __t8 := LibraCoin_value(__t7);
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 3, 7286);
      goto Label_Abort;
    }
    assume IsValidU64(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 4, __tmp);
    assume $DebugTrackLocal(5, 3, 4, 7270, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    __tmp := GetLocal(__m, __frame + 12);
    if (!b#Boolean(__tmp)) { goto Label_10; }

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    if (true) { assume $DebugTrackAbort(5, 3, 7356); }
    goto Label_Abort;

Label_10:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __t15 := BorrowGlobal(GetLocal(__m, __frame + 14), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 3, 7427);
      goto Label_Abort;
    }

    call sender_account_ref := CopyOrMoveRef(__t15);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account_ref));
    assume $DebugTrackLocal(5, 3, 6, 7406, Dereference(__m, sender_account_ref));

    call __t16 := CopyOrMoveRef(sender_account_ref);

    call __t17 := BorrowField(__t16, LibraAccount_T_sent_events);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := Pack_LibraAccount_SentPaymentEvent(0, 0, 0, 0, GetLocal(__m, __frame + 18), GetLocal(__m, __frame + 19), GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    call LibraAccount_emit_event(LibraAccount_SentPaymentEvent_type_value(), __t17, GetLocal(__m, __frame + 21));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 3, 7499);
      goto Label_Abort;
    }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    call __t23 := BorrowGlobal(GetLocal(__m, __frame + 22), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 3, 7838);
      goto Label_Abort;
    }

    call payee_account_ref := CopyOrMoveRef(__t23);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, payee_account_ref));
    assume $DebugTrackLocal(5, 3, 5, 7818, Dereference(__m, payee_account_ref));

    call __t24 := CopyOrMoveRef(payee_account_ref);

    call __t25 := BorrowField(__t24, LibraAccount_T_balance);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 26, __tmp);

    call LibraCoin_deposit(__t25, GetLocal(__m, __frame + 26));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 3, 7922);
      goto Label_Abort;
    }

    call __t27 := CopyOrMoveRef(payee_account_ref);

    call __t28 := BorrowField(__t27, LibraAccount_T_received_events);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 29, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 30, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 31, __tmp);

    call __tmp := Pack_LibraAccount_ReceivedPaymentEvent(0, 0, 0, 0, GetLocal(__m, __frame + 29), GetLocal(__m, __frame + 30), GetLocal(__m, __frame + 31));
    __m := UpdateLocal(__m, __frame + 32, __tmp);

    call LibraAccount_emit_event(LibraAccount_ReceivedPaymentEvent_type_value(), __t28, GetLocal(__m, __frame + 32));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 3, 8037);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_deposit_with_sender_and_metadata_verify (payee: Value, sender: Value, to_deposit: Value, metadata: Value) returns ()
{
    call InitVerification();
    call LibraAccount_deposit_with_sender_and_metadata(payee, sender, to_deposit, metadata);
}

procedure {:inline 1} LibraAccount_mint_to_address (payee: Value, amount: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)));
ensures !__abort_flag ==> b#Boolean(Boolean(b#Boolean(Boolean(!(b#Boolean(Boolean(!(b#Boolean(old(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)))))))))) || b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value), amount)))));
ensures !__abort_flag ==> b#Boolean(Boolean(b#Boolean(Boolean(!(b#Boolean(old(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee))))))) || b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value), Integer(i#Integer(old(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value))) + i#Integer(amount)))))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value), Integer(i#Integer(old(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))) + i#Integer(amount)))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(amount) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(IsEqual(amount, Integer(0)))) || b#Boolean(Boolean(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee))) && b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value)) + i#Integer(amount))) > i#Integer(max_u64()))))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_sent_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee))) && b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_received_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(amount) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(max_u64()))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(amount) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(IsEqual(amount, Integer(0)))) || b#Boolean(Boolean(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee))) && b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value)) + i#Integer(amount))) > i#Integer(max_u64()))))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_sent_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee))) && b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_received_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(amount) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(max_u64())))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // AddressType()
    var __t3: Value; // BooleanType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // AddressType()
    var __t6: Value; // AddressType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume $DebugTrackLocal(5, 4, 0, 8679, payee);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(5, 4, 1, 8679, amount);

    // increase the local counter
    __local_counter := __local_counter + 9;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 2), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __tmp := GetLocal(__m, __frame + 4);
    if (!b#Boolean(__tmp)) { goto Label_6; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call LibraAccount_create_account(GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 4, 9936);
      goto Label_Abort;
    }

Label_6:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := LibraCoin_mint_with_default_capability(GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 4, 10052);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);

    call LibraAccount_deposit(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 4, 10026);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_mint_to_address_verify (payee: Value, amount: Value) returns ()
{
    call InitVerification();
    call LibraAccount_mint_to_address(payee, amount);
}

procedure {:inline 1} LibraAccount_withdraw_from_account (account: Reference, amount: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), amount)));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, account), LibraAccount_T_balance), LibraCoin_T_value), Integer(i#Integer(old(SelectField(SelectField(Dereference(__m, account), LibraAccount_T_balance), LibraCoin_T_value))) - i#Integer(amount)))));
ensures old(!(b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(__m, account), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(amount))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(__m, account), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(amount)))) ==> __abort_flag;

{
    // declare local variables
    var to_withdraw: Value; // LibraCoin_T_type_value()
    var __t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t4: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t5: Value; // IntegerType()
    var __t6: Value; // LibraCoin_T_type_value()
    var __t7: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account)) && IsValidReferenceParameter(__m, __local_counter, account);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(5, 5, 0, 10231, Dereference(__m, account));
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(5, 5, 1, 10231, amount);

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __t3 := CopyOrMoveRef(account);

    call __t4 := BorrowField(__t3, LibraAccount_T_balance);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := LibraCoin_withdraw(__t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 5, 10530);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t6);

    __m := UpdateLocal(__m, __frame + 6, __t6);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(5, 5, 0, 10530, Dereference(__m, account));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(5, 5, 2, 10516, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __ret0 := GetLocal(__m, __frame + 7);
    assume $DebugTrackLocal(5, 5, 3, 10600, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_from_account_verify (account: Reference, amount: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_withdraw_from_account(account, amount);
}

procedure {:inline 1} LibraAccount_withdraw_from_sender (amount: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), amount)));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value), Integer(i#Integer(old(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value))) - i#Integer(amount)))));
ensures old(!(b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_withdrawal_capability)) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(amount))))) ==> !__abort_flag;
ensures old(b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_withdrawal_capability)) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(amount)))) ==> __abort_flag;

{
    // declare local variables
    var sender_account: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t2: Value; // AddressType()
    var __t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t5: Reference; // ReferenceType(BooleanType())
    var __t6: Value; // BooleanType()
    var __t7: Value; // IntegerType()
    var __t8: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t9: Value; // IntegerType()
    var __t10: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 0, amount);
    assume $DebugTrackLocal(5, 6, 0, 10712, amount);

    // increase the local counter
    __local_counter := __local_counter + 11;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 6, 11146);
      goto Label_Abort;
    }

    call sender_account := CopyOrMoveRef(__t3);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account));
    assume $DebugTrackLocal(5, 6, 1, 11129, Dereference(__m, sender_account));

    call __t4 := CopyOrMoveRef(sender_account);

    call __t5 := BorrowField(__t4, LibraAccount_T_delegated_withdrawal_capability);

    call __tmp := ReadRef(__t5);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_9; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    if (true) { assume $DebugTrackAbort(5, 6, 11369); }
    goto Label_Abort;

Label_9:
    call __t8 := CopyOrMoveRef(sender_account);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := LibraAccount_withdraw_from_account(__t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 6, 11491);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t10);

    __m := UpdateLocal(__m, __frame + 10, __t10);

    __ret0 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(5, 6, 2, 11484, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_from_sender_verify (amount: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_withdraw_from_sender(amount);
}

procedure {:inline 1} LibraAccount_withdraw_with_capability (cap: Reference, amount: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), amount)));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(SelectField(Dereference(__m, cap), LibraAccount_WithdrawalCapability_account_address)))), LibraAccount_T_balance), LibraCoin_T_value), Integer(i#Integer(old(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(SelectField(Dereference(__m, cap), LibraAccount_WithdrawalCapability_account_address)))), LibraAccount_T_balance), LibraCoin_T_value))) - i#Integer(amount)))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(SelectField(Dereference(__m, cap), LibraAccount_WithdrawalCapability_account_address))))))) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(SelectField(Dereference(__m, cap), LibraAccount_WithdrawalCapability_account_address)))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(amount))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(SelectField(Dereference(__m, cap), LibraAccount_WithdrawalCapability_account_address))))))) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(SelectField(Dereference(__m, cap), LibraAccount_WithdrawalCapability_account_address)))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(amount)))) ==> __abort_flag;

{
    // declare local variables
    var account: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t3: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var __t4: Reference; // ReferenceType(AddressType())
    var __t5: Value; // AddressType()
    var __t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t7: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t8: Value; // IntegerType()
    var __t9: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_WithdrawalCapability_is_well_formed(Dereference(__m, cap)) && IsValidReferenceParameter(__m, __local_counter, cap);
    assume $LibraAccount_WithdrawalCapability_is_well_formed(Dereference(__m, cap));
    assume $DebugTrackLocal(5, 7, 0, 11652, Dereference(__m, cap));
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(5, 7, 1, 11652, amount);

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __t3 := CopyOrMoveRef(cap);

    call __t4 := BorrowField(__t3, LibraAccount_WithdrawalCapability_account_address);

    call __tmp := ReadRef(__t4);
    assume is#Address(__tmp);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := BorrowGlobal(GetLocal(__m, __frame + 5), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 7, 12135);
      goto Label_Abort;
    }

    call account := CopyOrMoveRef(__t6);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(5, 7, 2, 12125, Dereference(__m, account));

    call __t7 := CopyOrMoveRef(account);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := LibraAccount_withdraw_from_account(__t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 7, 12201);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    __ret0 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(5, 7, 3, 12194, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_with_capability_verify (cap: Reference, amount: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_withdraw_with_capability(cap, amount);
}

procedure {:inline 1} LibraAccount_extract_sender_withdrawal_capability () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_withdrawal_capability));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraAccount_WithdrawalCapability_account_address), Address(TxnSenderAddress(__txn)))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_withdrawal_capability)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_withdrawal_capability))) ==> __abort_flag;

{
    // declare local variables
    var sender: Value; // AddressType()
    var sender_account: Reference; // ReferenceType(LibraAccount_T_type_value())
    var delegated_ref: Reference; // ReferenceType(BooleanType())
    var __t3: Value; // AddressType()
    var __t4: Value; // AddressType()
    var __t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t7: Reference; // ReferenceType(BooleanType())
    var __t8: Reference; // ReferenceType(BooleanType())
    var __t9: Value; // BooleanType()
    var __t10: Value; // IntegerType()
    var __t11: Value; // BooleanType()
    var __t12: Reference; // ReferenceType(BooleanType())
    var __t13: Value; // AddressType()
    var __t14: Value; // LibraAccount_WithdrawalCapability_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 15;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(5, 8, 0, 12840, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __t5 := BorrowGlobal(GetLocal(__m, __frame + 4), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 8, 12892);
      goto Label_Abort;
    }

    call sender_account := CopyOrMoveRef(__t5);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account));
    assume $DebugTrackLocal(5, 8, 1, 12875, Dereference(__m, sender_account));

    call __t6 := CopyOrMoveRef(sender_account);

    call __t7 := BorrowField(__t6, LibraAccount_T_delegated_withdrawal_capability);

    call delegated_ref := CopyOrMoveRef(__t7);
    assume is#Boolean(Dereference(__m, delegated_ref));
    assume $DebugTrackLocal(5, 8, 2, 12936, Dereference(__m, delegated_ref));

    call __t8 := CopyOrMoveRef(delegated_ref);

    call __tmp := ReadRef(__t8);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __tmp := GetLocal(__m, __frame + 9);
    if (!b#Boolean(__tmp)) { goto Label_13; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    if (true) { assume $DebugTrackAbort(5, 8, 13146); }
    goto Label_Abort;

Label_13:
    call __tmp := LdTrue();
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := CopyOrMoveRef(delegated_ref);

    call WriteRef(__t12, GetLocal(__m, __frame + 11));


    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := Pack_LibraAccount_WithdrawalCapability(0, 0, 0, 0, GetLocal(__m, __frame + 13));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __ret0 := GetLocal(__m, __frame + 14);
    assume $DebugTrackLocal(5, 8, 3, 13266, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_extract_sender_withdrawal_capability_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_extract_sender_withdrawal_capability();
}

procedure {:inline 1} LibraAccount_restore_withdrawal_capability (cap: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(!(b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(SelectField(cap, LibraAccount_WithdrawalCapability_account_address)))), LibraAccount_T_delegated_withdrawal_capability)))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(SelectField(cap, LibraAccount_WithdrawalCapability_account_address))))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(SelectField(cap, LibraAccount_WithdrawalCapability_account_address)))))))) ==> __abort_flag;

{
    // declare local variables
    var account_address: Value; // AddressType()
    var account: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t3: Value; // LibraAccount_WithdrawalCapability_type_value()
    var __t4: Value; // AddressType()
    var __t5: Value; // AddressType()
    var __t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t7: Value; // BooleanType()
    var __t8: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t9: Reference; // ReferenceType(BooleanType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_WithdrawalCapability_is_well_formed(cap);
    __m := UpdateLocal(__m, __frame + 0, cap);
    assume $DebugTrackLocal(5, 9, 0, 13429, cap);

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Unpack_LibraAccount_WithdrawalCapability(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(5, 9, 1, 13808, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := BorrowGlobal(GetLocal(__m, __frame + 5), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 9, 13857);
      goto Label_Abort;
    }

    call account := CopyOrMoveRef(__t6);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(5, 9, 2, 13847, Dereference(__m, account));

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := CopyOrMoveRef(account);

    call __t9 := BorrowField(__t8, LibraAccount_T_delegated_withdrawal_capability);

    call WriteRef(__t9, GetLocal(__m, __frame + 7));


    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_restore_withdrawal_capability_verify (cap: Value) returns ()
{
    call InitVerification();
    call LibraAccount_restore_withdrawal_capability(cap);
}

procedure {:inline 1} LibraAccount_pay_from_capability (payee: Value, cap: Reference, amount: Value, metadata: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)));
ensures b#Boolean(Boolean(b#Boolean(Boolean(!(b#Boolean(Boolean(!(b#Boolean(old(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)))))))))) || b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value), amount)))));
{
    // declare local variables
    var __t4: Value; // AddressType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // BooleanType()
    var __t7: Value; // AddressType()
    var __t8: Value; // AddressType()
    var __t9: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var __t10: Reference; // ReferenceType(AddressType())
    var __t11: Value; // AddressType()
    var __t12: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var __t13: Value; // IntegerType()
    var __t14: Value; // LibraCoin_T_type_value()
    var __t15: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume $DebugTrackLocal(5, 10, 0, 14433, payee);
    assume $LibraAccount_WithdrawalCapability_is_well_formed(Dereference(__m, cap)) && IsValidReferenceParameter(__m, __local_counter, cap);
    assume $LibraAccount_WithdrawalCapability_is_well_formed(Dereference(__m, cap));
    assume $DebugTrackLocal(5, 10, 1, 14433, Dereference(__m, cap));
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 2, amount);
    assume $DebugTrackLocal(5, 10, 2, 14433, amount);
    assume is#ByteArray(metadata);
    __m := UpdateLocal(__m, __frame + 3, metadata);
    assume $DebugTrackLocal(5, 10, 3, 14433, metadata);

    // increase the local counter
    __local_counter := __local_counter + 16;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 4), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_6; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call LibraAccount_create_account(GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 10, 15573);
      goto Label_Abort;
    }

Label_6:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := CopyOrMoveRef(cap);

    call __t10 := BorrowField(__t9, LibraAccount_WithdrawalCapability_account_address);

    call __tmp := ReadRef(__t10);
    assume is#Address(__tmp);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := CopyOrMoveRef(cap);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __t14 := LibraAccount_withdraw_with_capability(__t12, GetLocal(__m, __frame + 13));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 10, 15742);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t14);

    __m := UpdateLocal(__m, __frame + 14, __t14);

    // unimplemented instruction: LdByteArray(15, ByteArrayPoolIndex(0))

    call LibraAccount_deposit_with_sender_and_metadata(GetLocal(__m, __frame + 8), GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 10, 15625);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_pay_from_capability_verify (payee: Value, cap: Reference, amount: Value, metadata: Value) returns ()
{
    call InitVerification();
    call LibraAccount_pay_from_capability(payee, cap, amount, metadata);
}

procedure {:inline 1} LibraAccount_pay_from_sender_with_metadata (payee: Value, amount: Value, metadata: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)));
ensures !__abort_flag ==> b#Boolean(Boolean(b#Boolean(Boolean(!(b#Boolean(old(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee))))))) || b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value), Integer(i#Integer(old(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value))) + i#Integer(amount)))))));
ensures !__abort_flag ==> b#Boolean(Boolean(b#Boolean(Boolean(!(b#Boolean(Boolean(!(b#Boolean(old(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)))))))))) || b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value), amount)))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value), Integer(i#Integer(old(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value))) - i#Integer(amount)))));
ensures old(!(b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_withdrawal_capability)) || b#Boolean(Boolean(IsEqual(Address(TxnSenderAddress(__txn)), payee))) || b#Boolean(Boolean(IsEqual(amount, Integer(0)))) || b#Boolean(Boolean(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee))) && b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value)) + i#Integer(amount))) > i#Integer(max_u64()))))) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(amount))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_sent_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_received_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))))) ==> !__abort_flag;
ensures old(b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_withdrawal_capability)) || b#Boolean(Boolean(IsEqual(Address(TxnSenderAddress(__txn)), payee))) || b#Boolean(Boolean(IsEqual(amount, Integer(0)))) || b#Boolean(Boolean(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee))) && b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value)) + i#Integer(amount))) > i#Integer(max_u64()))))) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(amount))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_sent_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_received_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64())))) ==> __abort_flag;

{
    // declare local variables
    var __t3: Value; // AddressType()
    var __t4: Value; // AddressType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // BooleanType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // AddressType()
    var __t9: Value; // BooleanType()
    var __t10: Value; // BooleanType()
    var __t11: Value; // AddressType()
    var __t12: Value; // AddressType()
    var __t13: Value; // IntegerType()
    var __t14: Value; // LibraCoin_T_type_value()
    var __t15: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume $DebugTrackLocal(5, 11, 0, 16060, payee);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(5, 11, 1, 16060, amount);
    assume is#ByteArray(metadata);
    __m := UpdateLocal(__m, __frame + 2, metadata);
    assume $DebugTrackLocal(5, 11, 2, 16060, metadata);

    // increase the local counter
    __local_counter := __local_counter + 16;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __tmp := Boolean(!IsEqual(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4)));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(12);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    if (true) { assume $DebugTrackAbort(5, 11, 17209); }
    goto Label_Abort;

Label_7:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 8), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __tmp := GetLocal(__m, __frame + 10);
    if (!b#Boolean(__tmp)) { goto Label_13; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call LibraAccount_create_account(GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 11, 17266);
      goto Label_Abort;
    }

Label_13:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __t14 := LibraAccount_withdraw_from_sender(GetLocal(__m, __frame + 13));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 11, 17383);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t14);

    __m := UpdateLocal(__m, __frame + 14, __t14);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call LibraAccount_deposit_with_metadata(GetLocal(__m, __frame + 12), GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 11, 17318);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_pay_from_sender_with_metadata_verify (payee: Value, amount: Value, metadata: Value) returns ()
{
    call InitVerification();
    call LibraAccount_pay_from_sender_with_metadata(payee, amount, metadata);
}

procedure {:inline 1} LibraAccount_pay_from_sender (payee: Value, amount: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)));
ensures !__abort_flag ==> b#Boolean(Boolean(b#Boolean(Boolean(!(b#Boolean(old(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee))))))) || b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value), Integer(i#Integer(old(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value))) + i#Integer(amount)))))));
ensures !__abort_flag ==> b#Boolean(Boolean(b#Boolean(Boolean(!(b#Boolean(Boolean(!(b#Boolean(old(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee)))))))))) || b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value), amount)))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value), Integer(i#Integer(old(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value))) - i#Integer(amount)))));
ensures old(!(b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_withdrawal_capability)) || b#Boolean(Boolean(IsEqual(Address(TxnSenderAddress(__txn)), payee))) || b#Boolean(Boolean(IsEqual(amount, Integer(0)))) || b#Boolean(Boolean(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee))) && b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value)) + i#Integer(amount))) > i#Integer(max_u64()))))) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(amount))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_sent_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_received_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))))) ==> !__abort_flag;
ensures old(b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_withdrawal_capability)) || b#Boolean(Boolean(IsEqual(Address(TxnSenderAddress(__txn)), payee))) || b#Boolean(Boolean(IsEqual(amount, Integer(0)))) || b#Boolean(Boolean(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(payee))) && b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_balance), LibraCoin_T_value)) + i#Integer(amount))) > i#Integer(max_u64()))))) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(amount))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_sent_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(payee))), LibraAccount_T_received_events), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64())))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // AddressType()
    var __t3: Value; // AddressType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // AddressType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume $DebugTrackLocal(5, 12, 0, 17668, payee);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(5, 12, 1, 17668, amount);

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __tmp := Boolean(!IsEqual(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3)));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(12);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    if (true) { assume $DebugTrackAbort(5, 12, 18744); }
    goto Label_Abort;

Label_7:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    // unimplemented instruction: LdByteArray(9, ByteArrayPoolIndex(0))

    call LibraAccount_pay_from_sender_with_metadata(GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8), GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 12, 18758);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_pay_from_sender_verify (payee: Value, amount: Value) returns ()
{
    call InitVerification();
    call LibraAccount_pay_from_sender(payee, amount);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key_for_account (account: Reference, new_authentication_key: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(SelectField(Dereference(__m, account), LibraAccount_T_authentication_key), new_authentication_key)));
{
    // declare local variables
    var __t2: Value; // ByteArrayType()
    var __t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t4: Reference; // ReferenceType(ByteArrayType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account)) && IsValidReferenceParameter(__m, __local_counter, account);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(5, 13, 0, 18853, Dereference(__m, account));
    assume is#ByteArray(new_authentication_key);
    __m := UpdateLocal(__m, __frame + 1, new_authentication_key);
    assume $DebugTrackLocal(5, 13, 1, 18853, new_authentication_key);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := CopyOrMoveRef(account);

    call __t4 := BorrowField(__t3, LibraAccount_T_authentication_key);

    call WriteRef(__t4, GetLocal(__m, __frame + 2));
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(5, 13, 0, 19031, Dereference(__m, account));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_rotate_authentication_key_for_account_verify (account: Reference, new_authentication_key: Value) returns ()
{
    call InitVerification();
    call LibraAccount_rotate_authentication_key_for_account(account, new_authentication_key);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key (new_authentication_key: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_authentication_key), new_authentication_key)));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_key_rotation_capability)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_key_rotation_capability))) ==> __abort_flag;

{
    // declare local variables
    var sender_account: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t2: Value; // AddressType()
    var __t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t5: Reference; // ReferenceType(BooleanType())
    var __t6: Value; // BooleanType()
    var __t7: Value; // IntegerType()
    var __t8: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t9: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#ByteArray(new_authentication_key);
    __m := UpdateLocal(__m, __frame + 0, new_authentication_key);
    assume $DebugTrackLocal(5, 14, 0, 19253, new_authentication_key);

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 14, 19633);
      goto Label_Abort;
    }

    call sender_account := CopyOrMoveRef(__t3);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account));
    assume $DebugTrackLocal(5, 14, 1, 19616, Dereference(__m, sender_account));

    call __t4 := CopyOrMoveRef(sender_account);

    call __t5 := BorrowField(__t4, LibraAccount_T_delegated_key_rotation_capability);

    call __tmp := ReadRef(__t5);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_9; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    if (true) { assume $DebugTrackAbort(5, 14, 19846); }
    goto Label_Abort;

Label_9:
    call __t8 := CopyOrMoveRef(sender_account);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call LibraAccount_rotate_authentication_key_for_account(__t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 14, 19963);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_rotate_authentication_key_verify (new_authentication_key: Value) returns ()
{
    call InitVerification();
    call LibraAccount_rotate_authentication_key(new_authentication_key);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key_with_capability (cap: Reference, new_authentication_key: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(SelectField(Dereference(__m, cap), LibraAccount_KeyRotationCapability_account_address))))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(SelectField(Dereference(__m, cap), LibraAccount_KeyRotationCapability_account_address)))))))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var __t3: Reference; // ReferenceType(AddressType())
    var __t4: Value; // AddressType()
    var __t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t6: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_KeyRotationCapability_is_well_formed(Dereference(__m, cap)) && IsValidReferenceParameter(__m, __local_counter, cap);
    assume $LibraAccount_KeyRotationCapability_is_well_formed(Dereference(__m, cap));
    assume $DebugTrackLocal(5, 15, 0, 20225, Dereference(__m, cap));
    assume is#ByteArray(new_authentication_key);
    __m := UpdateLocal(__m, __frame + 1, new_authentication_key);
    assume $DebugTrackLocal(5, 15, 1, 20225, new_authentication_key);

    // increase the local counter
    __local_counter := __local_counter + 7;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(cap);

    call __t3 := BorrowField(__t2, LibraAccount_KeyRotationCapability_account_address);

    call __tmp := ReadRef(__t3);
    assume is#Address(__tmp);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __t5 := BorrowGlobal(GetLocal(__m, __frame + 4), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 15, 20617);
      goto Label_Abort;
    }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call LibraAccount_rotate_authentication_key_for_account(__t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 15, 20561);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_rotate_authentication_key_with_capability_verify (cap: Reference, new_authentication_key: Value) returns ()
{
    call InitVerification();
    call LibraAccount_rotate_authentication_key_with_capability(cap, new_authentication_key);
}

procedure {:inline 1} LibraAccount_extract_sender_key_rotation_capability () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraAccount_KeyRotationCapability_account_address), Address(TxnSenderAddress(__txn)))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_key_rotation_capability)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_key_rotation_capability))) ==> __abort_flag;

{
    // declare local variables
    var sender: Value; // AddressType()
    var delegated_ref: Reference; // ReferenceType(BooleanType())
    var __t2: Value; // AddressType()
    var __t3: Value; // AddressType()
    var __t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t5: Reference; // ReferenceType(BooleanType())
    var __t6: Reference; // ReferenceType(BooleanType())
    var __t7: Value; // BooleanType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // BooleanType()
    var __t10: Reference; // ReferenceType(BooleanType())
    var __t11: Value; // AddressType()
    var __t12: Value; // LibraAccount_KeyRotationCapability_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 13;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(5, 16, 0, 21202, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := BorrowGlobal(GetLocal(__m, __frame + 3), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 16, 21258);
      goto Label_Abort;
    }

    call __t5 := BorrowField(__t4, LibraAccount_T_delegated_key_rotation_capability);

    call delegated_ref := CopyOrMoveRef(__t5);
    assume is#Boolean(Dereference(__m, delegated_ref));
    assume $DebugTrackLocal(5, 16, 1, 21237, Dereference(__m, delegated_ref));

    call __t6 := CopyOrMoveRef(delegated_ref);

    call __tmp := ReadRef(__t6);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __tmp := GetLocal(__m, __frame + 7);
    if (!b#Boolean(__tmp)) { goto Label_11; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    if (true) { assume $DebugTrackAbort(5, 16, 21465); }
    goto Label_Abort;

Label_11:
    call __tmp := LdTrue();
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := CopyOrMoveRef(delegated_ref);

    call WriteRef(__t10, GetLocal(__m, __frame + 9));


    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := Pack_LibraAccount_KeyRotationCapability(0, 0, 0, 0, GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    __ret0 := GetLocal(__m, __frame + 12);
    assume $DebugTrackLocal(5, 16, 2, 21585, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_extract_sender_key_rotation_capability_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_extract_sender_key_rotation_capability();
}

procedure {:inline 1} LibraAccount_restore_key_rotation_capability (cap: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(!(b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(SelectField(cap, LibraAccount_KeyRotationCapability_account_address)))), LibraAccount_T_delegated_key_rotation_capability)))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(SelectField(cap, LibraAccount_KeyRotationCapability_account_address))))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(SelectField(cap, LibraAccount_KeyRotationCapability_account_address)))))))) ==> __abort_flag;

{
    // declare local variables
    var account_address: Value; // AddressType()
    var account: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t3: Value; // LibraAccount_KeyRotationCapability_type_value()
    var __t4: Value; // AddressType()
    var __t5: Value; // AddressType()
    var __t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t7: Value; // BooleanType()
    var __t8: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t9: Reference; // ReferenceType(BooleanType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_KeyRotationCapability_is_well_formed(cap);
    __m := UpdateLocal(__m, __frame + 0, cap);
    assume $DebugTrackLocal(5, 17, 0, 21751, cap);

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Unpack_LibraAccount_KeyRotationCapability(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(5, 17, 1, 22136, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := BorrowGlobal(GetLocal(__m, __frame + 5), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 17, 22185);
      goto Label_Abort;
    }

    call account := CopyOrMoveRef(__t6);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(5, 17, 2, 22175, Dereference(__m, account));

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := CopyOrMoveRef(account);

    call __t9 := BorrowField(__t8, LibraAccount_T_delegated_key_rotation_capability);

    call WriteRef(__t9, GetLocal(__m, __frame + 7));


    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_restore_key_rotation_capability_verify (cap: Value) returns ()
{
    call InitVerification();
    call LibraAccount_restore_key_rotation_capability(cap);
}

procedure {:inline 1} LibraAccount_create_account (fresh_address: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(fresh_address)));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(fresh_address))), LibraAccount_T_balance), LibraCoin_T_value), Integer(0))));
ensures !__abort_flag ==> b#Boolean(Boolean(!(b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(fresh_address))), LibraAccount_T_delegated_withdrawal_capability)))));
ensures old(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(fresh_address))))) ==> !__abort_flag;
ensures old(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(fresh_address)))) ==> __abort_flag;

{
    // declare local variables
    var generator: Value; // LibraAccount_EventHandleGenerator_type_value()
    var __t2: Value; // IntegerType()
    var __t3: Value; // LibraAccount_EventHandleGenerator_type_value()
    var __t4: Value; // AddressType()
    var __t5: Value; // AddressType()
    var __t6: Value; // ByteArrayType()
    var __t7: Value; // LibraCoin_T_type_value()
    var __t8: Value; // BooleanType()
    var __t9: Value; // BooleanType()
    var __t10: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var __t11: Value; // AddressType()
    var __t12: Value; // LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())
    var __t13: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var __t14: Value; // AddressType()
    var __t15: Value; // LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())
    var __t16: Value; // IntegerType()
    var __t17: Value; // LibraAccount_EventHandleGenerator_type_value()
    var __t18: Value; // LibraAccount_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(fresh_address);
    __m := UpdateLocal(__m, __frame + 0, fresh_address);
    assume $DebugTrackLocal(5, 18, 0, 22651, fresh_address);

    // increase the local counter
    __local_counter := __local_counter + 19;

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := Pack_LibraAccount_EventHandleGenerator(5, 18, 1, 23026, GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(5, 18, 1, 23014, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := AddressUtil_address_to_bytes(GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 18, 23173);
      goto Label_Abort;
    }
    assume is#ByteArray(__t6);

    __m := UpdateLocal(__m, __frame + 6, __t6);

    call __t7 := LibraCoin_zero();
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 18, 23249);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := BorrowLoc(__frame + 1, LibraAccount_EventHandleGenerator_type_value());

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := LibraAccount_new_event_handle_impl(LibraAccount_ReceivedPaymentEvent_type_value(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 18, 23414);
      goto Label_Abort;
    }
    assume $LibraAccount_EventHandle_is_well_formed(__t12);

    __m := UpdateLocal(__m, __frame + 12, __t12);
    assume $DebugTrackLocal(5, 18, 1, 23414, GetLocal(__m, __frame + 1));

    call __t13 := BorrowLoc(__frame + 1, LibraAccount_EventHandleGenerator_type_value());

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __t15 := LibraAccount_new_event_handle_impl(LibraAccount_SentPaymentEvent_type_value(), __t13, GetLocal(__m, __frame + 14));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 18, 23535);
      goto Label_Abort;
    }
    assume $LibraAccount_EventHandle_is_well_formed(__t15);

    __m := UpdateLocal(__m, __frame + 15, __t15);
    assume $DebugTrackLocal(5, 18, 1, 23535, GetLocal(__m, __frame + 1));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := Pack_LibraAccount_T(0, 0, 0, 0, GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8), GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 12), GetLocal(__m, __frame + 15), GetLocal(__m, __frame + 16), GetLocal(__m, __frame + 17));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call LibraAccount_save_account(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 18));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 18, 23069);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_create_account_verify (fresh_address: Value) returns ()
{
    call InitVerification();
    call LibraAccount_create_account(fresh_address);
}

procedure {:inline 1} LibraAccount_create_new_account (fresh_address: Value, initial_balance: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(fresh_address)));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(fresh_address))), LibraAccount_T_balance), LibraCoin_T_value), initial_balance)));
ensures !__abort_flag ==> b#Boolean(Boolean(!(b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(fresh_address))), LibraAccount_T_delegated_withdrawal_capability)))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value), Integer(i#Integer(old(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value))) - i#Integer(initial_balance)))));
ensures old(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(fresh_address))) || b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_withdrawal_capability)) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(initial_balance))) || b#Boolean(Boolean(i#Integer(initial_balance) > i#Integer(max_u64()))))) ==> !__abort_flag;
ensures old(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(fresh_address))) || b#Boolean(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_delegated_withdrawal_capability)) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(initial_balance))) || b#Boolean(Boolean(i#Integer(initial_balance) > i#Integer(max_u64())))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // AddressType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // AddressType()
    var __t7: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(fresh_address);
    __m := UpdateLocal(__m, __frame + 0, fresh_address);
    assume $DebugTrackLocal(5, 19, 0, 23890, fresh_address);
    assume IsValidU64(initial_balance);
    __m := UpdateLocal(__m, __frame + 1, initial_balance);
    assume $DebugTrackLocal(5, 19, 1, 23890, initial_balance);

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call LibraAccount_create_account(GetLocal(__m, __frame + 2));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 19, 24585);
      goto Label_Abort;
    }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_9; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call LibraAccount_pay_from_sender(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 19, 24680);
      goto Label_Abort;
    }

Label_9:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_create_new_account_verify (fresh_address: Value, initial_balance: Value) returns ()
{
    call InitVerification();
    call LibraAccount_create_new_account(fresh_address, initial_balance);
}

procedure {:inline 1} LibraAccount_balance_for_account (account: Reference) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, SelectField(SelectField(Dereference(__m, account), LibraAccount_T_balance), LibraCoin_T_value))));
{
    // declare local variables
    var balance_value: Value; // IntegerType()
    var __t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account)) && IsValidReferenceParameter(__m, __local_counter, account);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(5, 21, 0, 25151, Dereference(__m, account));

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(account);

    call __t3 := BorrowField(__t2, LibraAccount_T_balance);

    call __t4 := LibraCoin_value(__t3);
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 21, 25301);
      goto Label_Abort;
    }
    assume IsValidU64(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(5, 21, 1, 25285, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(5, 21, 2, 25350, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_balance_for_account_verify (account: Reference) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_balance_for_account(account);
}

procedure {:inline 1} LibraAccount_balance (addr: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(__ret0, SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(addr))), LibraAccount_T_balance), LibraCoin_T_value))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(addr)))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(addr))))))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Value; // AddressType()
    var __t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);
    assume $DebugTrackLocal(5, 22, 0, 25470, addr);

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 22, 25675);
      goto Label_Abort;
    }

    call __t3 := LibraAccount_balance_for_account(__t2);
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 22, 25650);
      goto Label_Abort;
    }
    assume IsValidU64(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(5, 22, 1, 25643, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_balance_verify (addr: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_balance(addr);
}

procedure {:inline 1} LibraAccount_sequence_number_for_account (account: Reference) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, SelectField(Dereference(__m, account), LibraAccount_T_sequence_number))));
{
    // declare local variables
    var __t1: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t2: Reference; // ReferenceType(IntegerType())
    var __t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account)) && IsValidReferenceParameter(__m, __local_counter, account);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(5, 23, 0, 25787, Dereference(__m, account));

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(account);

    call __t2 := BorrowField(__t1, LibraAccount_T_sequence_number);

    call __tmp := ReadRef(__t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(5, 23, 1, 25899, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_sequence_number_for_account_verify (account: Reference) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_sequence_number_for_account(account);
}

procedure {:inline 1} LibraAccount_sequence_number (addr: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(__ret0, SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(addr))), LibraAccount_T_sequence_number))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(addr)))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(addr))))))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Value; // AddressType()
    var __t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);
    assume $DebugTrackLocal(5, 24, 0, 26004, addr);

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 24, 26227);
      goto Label_Abort;
    }

    call __t3 := LibraAccount_sequence_number_for_account(__t2);
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 24, 26194);
      goto Label_Abort;
    }
    assume IsValidU64(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(5, 24, 1, 26187, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_sequence_number_verify (addr: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_sequence_number(addr);
}

procedure {:inline 1} LibraAccount_delegated_key_rotation_capability (addr: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(__ret0, SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(addr))), LibraAccount_T_delegated_key_rotation_capability))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(addr)))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(addr))))))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Value; // AddressType()
    var __t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t3: Reference; // ReferenceType(BooleanType())
    var __t4: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);
    assume $DebugTrackLocal(5, 25, 0, 26354, addr);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 25, 26584);
      goto Label_Abort;
    }

    call __t3 := BorrowField(__t2, LibraAccount_T_delegated_key_rotation_capability);

    call __tmp := ReadRef(__t3);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(5, 25, 1, 26574, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_delegated_key_rotation_capability_verify (addr: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_delegated_key_rotation_capability(addr);
}

procedure {:inline 1} LibraAccount_delegated_withdrawal_capability (addr: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(__ret0, SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(addr))), LibraAccount_T_delegated_withdrawal_capability))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(addr)))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(addr))))))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Value; // AddressType()
    var __t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t3: Reference; // ReferenceType(BooleanType())
    var __t4: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);
    assume $DebugTrackLocal(5, 26, 0, 26744, addr);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 26, 26970);
      goto Label_Abort;
    }

    call __t3 := BorrowField(__t2, LibraAccount_T_delegated_withdrawal_capability);

    call __tmp := ReadRef(__t3);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(5, 26, 1, 26960, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_delegated_withdrawal_capability_verify (addr: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_delegated_withdrawal_capability(addr);
}

procedure {:inline 1} LibraAccount_withdrawal_capability_address (cap: Reference) returns (__ret0: Reference)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean((__ret0) == (SelectFieldFromRef(cap, LibraAccount_WithdrawalCapability_account_address))));
{
    // declare local variables
    var __t1: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var __t2: Reference; // ReferenceType(AddressType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_WithdrawalCapability_is_well_formed(Dereference(__m, cap)) && IsValidReferenceParameter(__m, __local_counter, cap);
    assume $LibraAccount_WithdrawalCapability_is_well_formed(Dereference(__m, cap));
    assume $DebugTrackLocal(5, 27, 0, 27134, Dereference(__m, cap));

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(cap);

    call __t2 := BorrowField(__t1, LibraAccount_WithdrawalCapability_account_address);

    __ret0 := __t2;
    assume is#Address(Dereference(__m, __ret0));
    assume $DebugTrackLocal(5, 27, 1, 27273, Dereference(__m, __ret0));
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultReference;
}

procedure LibraAccount_withdrawal_capability_address_verify (cap: Reference) returns (__ret0: Reference)
{
    call InitVerification();
    call __ret0 := LibraAccount_withdrawal_capability_address(cap);
}

procedure {:inline 1} LibraAccount_key_rotation_capability_address (cap: Reference) returns (__ret0: Reference)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean((__ret0) == (SelectFieldFromRef(cap, LibraAccount_KeyRotationCapability_account_address))));
{
    // declare local variables
    var __t1: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var __t2: Reference; // ReferenceType(AddressType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_KeyRotationCapability_is_well_formed(Dereference(__m, cap)) && IsValidReferenceParameter(__m, __local_counter, cap);
    assume $LibraAccount_KeyRotationCapability_is_well_formed(Dereference(__m, cap));
    assume $DebugTrackLocal(5, 28, 0, 27410, Dereference(__m, cap));

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(cap);

    call __t2 := BorrowField(__t1, LibraAccount_KeyRotationCapability_account_address);

    __ret0 := __t2;
    assume is#Address(Dereference(__m, __ret0));
    assume $DebugTrackLocal(5, 28, 1, 27551, Dereference(__m, __ret0));
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultReference;
}

procedure LibraAccount_key_rotation_capability_address_verify (cap: Reference) returns (__ret0: Reference)
{
    call InitVerification();
    call __ret0 := LibraAccount_key_rotation_capability_address(cap);
}

procedure {:inline 1} LibraAccount_exists (check_addr: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, ExistsResource(__m, LibraAccount_T_type_value(), a#Address(check_addr)))));
{
    // declare local variables
    var __t1: Value; // AddressType()
    var __t2: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(check_addr);
    __m := UpdateLocal(__m, __frame + 0, check_addr);
    assume $DebugTrackLocal(5, 29, 0, 27648, check_addr);

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(5, 29, 1, 27760, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_exists_verify (check_addr: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_exists(check_addr);
}

procedure {:inline 1} LibraAccount_prologue (txn_sequence_number: Value, txn_public_key: Value, txn_gas_price: Value, txn_max_gas_units: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var transaction_sender: Value; // AddressType()
    var sender_account: Reference; // ReferenceType(LibraAccount_T_type_value())
    var imm_sender_account: Reference; // ReferenceType(LibraAccount_T_type_value())
    var max_transaction_fee: Value; // IntegerType()
    var balance_amount: Value; // IntegerType()
    var sequence_number_value: Value; // IntegerType()
    var __t10: Value; // AddressType()
    var __t11: Value; // AddressType()
    var __t12: Value; // BooleanType()
    var __t13: Value; // BooleanType()
    var __t14: Value; // IntegerType()
    var __t15: Value; // AddressType()
    var __t16: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t17: Value; // ByteArrayType()
    var __t18: Value; // ByteArrayType()
    var __t19: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t20: Reference; // ReferenceType(ByteArrayType())
    var __t21: Value; // ByteArrayType()
    var __t22: Value; // BooleanType()
    var __t23: Value; // BooleanType()
    var __t24: Value; // IntegerType()
    var __t25: Value; // IntegerType()
    var __t26: Value; // IntegerType()
    var __t27: Value; // IntegerType()
    var __t28: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t29: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t30: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t31: Value; // IntegerType()
    var __t32: Value; // IntegerType()
    var __t33: Value; // IntegerType()
    var __t34: Value; // BooleanType()
    var __t35: Value; // BooleanType()
    var __t36: Value; // IntegerType()
    var __t37: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t38: Reference; // ReferenceType(IntegerType())
    var __t39: Value; // IntegerType()
    var __t40: Value; // IntegerType()
    var __t41: Value; // IntegerType()
    var __t42: Value; // BooleanType()
    var __t43: Value; // BooleanType()
    var __t44: Value; // IntegerType()
    var __t45: Value; // IntegerType()
    var __t46: Value; // IntegerType()
    var __t47: Value; // BooleanType()
    var __t48: Value; // BooleanType()
    var __t49: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU64(txn_sequence_number);
    __m := UpdateLocal(__m, __frame + 0, txn_sequence_number);
    assume $DebugTrackLocal(5, 30, 0, 28109, txn_sequence_number);
    assume is#ByteArray(txn_public_key);
    __m := UpdateLocal(__m, __frame + 1, txn_public_key);
    assume $DebugTrackLocal(5, 30, 1, 28109, txn_public_key);
    assume IsValidU64(txn_gas_price);
    __m := UpdateLocal(__m, __frame + 2, txn_gas_price);
    assume $DebugTrackLocal(5, 30, 2, 28109, txn_gas_price);
    assume IsValidU64(txn_max_gas_units);
    __m := UpdateLocal(__m, __frame + 3, txn_max_gas_units);
    assume $DebugTrackLocal(5, 30, 3, 28109, txn_max_gas_units);

    // increase the local counter
    __local_counter := __local_counter + 50;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 10));
    __m := UpdateLocal(__m, __frame + 4, __tmp);
    assume $DebugTrackLocal(5, 30, 4, 28510, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 11), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    __tmp := GetLocal(__m, __frame + 13);
    if (!b#Boolean(__tmp)) { goto Label_8; }

    call __tmp := LdConst(5);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    if (true) { assume $DebugTrackAbort(5, 30, 28718); }
    goto Label_Abort;

Label_8:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __t16 := BorrowGlobal(GetLocal(__m, __frame + 15), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 30, 28797);
      goto Label_Abort;
    }

    call sender_account := CopyOrMoveRef(__t16);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account));
    assume $DebugTrackLocal(5, 30, 5, 28780, Dereference(__m, sender_account));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __t18 := Hash_sha3_256(GetLocal(__m, __frame + 17));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 30, 28968);
      goto Label_Abort;
    }
    assume is#ByteArray(__t18);

    __m := UpdateLocal(__m, __frame + 18, __t18);

    call __t19 := CopyOrMoveRef(sender_account);

    call __t20 := BorrowField(__t19, LibraAccount_T_authentication_key);

    call __tmp := ReadRef(__t20);
    assume is#ByteArray(__tmp);
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 18), GetLocal(__m, __frame + 21)));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 22));
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    __tmp := GetLocal(__m, __frame + 23);
    if (!b#Boolean(__tmp)) { goto Label_21; }

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 24, __tmp);

    if (true) { assume $DebugTrackAbort(5, 30, 29064); }
    goto Label_Abort;

Label_21:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 25, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 26, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 25), GetLocal(__m, __frame + 26));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 30, 29180);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 27, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 27));
    __m := UpdateLocal(__m, __frame + 7, __tmp);
    assume $DebugTrackLocal(5, 30, 7, 29158, __tmp);

    call __t28 := CopyOrMoveRef(sender_account);

    call __t29 := FreezeRef(__t28);

    call imm_sender_account := CopyOrMoveRef(__t29);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, imm_sender_account));
    assume $DebugTrackLocal(5, 30, 6, 29235, Dereference(__m, imm_sender_account));

    call __t30 := CopyOrMoveRef(imm_sender_account);

    call __t31 := LibraAccount_balance_for_account(__t30);
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 30, 29311);
      goto Label_Abort;
    }
    assume IsValidU64(__t31);

    __m := UpdateLocal(__m, __frame + 31, __t31);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 31));
    __m := UpdateLocal(__m, __frame + 8, __tmp);
    assume $DebugTrackLocal(5, 30, 8, 29294, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 32, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 33, __tmp);

    call __tmp := Ge(GetLocal(__m, __frame + 32), GetLocal(__m, __frame + 33));
    __m := UpdateLocal(__m, __frame + 34, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 34));
    __m := UpdateLocal(__m, __frame + 35, __tmp);

    __tmp := GetLocal(__m, __frame + 35);
    if (!b#Boolean(__tmp)) { goto Label_38; }

    call __tmp := LdConst(6);
    __m := UpdateLocal(__m, __frame + 36, __tmp);

    if (true) { assume $DebugTrackAbort(5, 30, 29429); }
    goto Label_Abort;

Label_38:
    call __t37 := CopyOrMoveRef(sender_account);

    call __t38 := BorrowField(__t37, LibraAccount_T_sequence_number);

    call __tmp := ReadRef(__t38);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 39, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 39));
    __m := UpdateLocal(__m, __frame + 9, __tmp);
    assume $DebugTrackLocal(5, 30, 9, 29539, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 40, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 41, __tmp);

    call __tmp := Ge(GetLocal(__m, __frame + 40), GetLocal(__m, __frame + 41));
    __m := UpdateLocal(__m, __frame + 42, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 42));
    __m := UpdateLocal(__m, __frame + 43, __tmp);

    __tmp := GetLocal(__m, __frame + 43);
    if (!b#Boolean(__tmp)) { goto Label_49; }

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 44, __tmp);

    if (true) { assume $DebugTrackAbort(5, 30, 29682); }
    goto Label_Abort;

Label_49:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 45, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 46, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 45), GetLocal(__m, __frame + 46)));
    __m := UpdateLocal(__m, __frame + 47, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 47));
    __m := UpdateLocal(__m, __frame + 48, __tmp);

    __tmp := GetLocal(__m, __frame + 48);
    if (!b#Boolean(__tmp)) { goto Label_56; }

    call __tmp := LdConst(4);
    __m := UpdateLocal(__m, __frame + 49, __tmp);

    if (true) { assume $DebugTrackAbort(5, 30, 29759); }
    goto Label_Abort;

Label_56:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_prologue_verify (txn_sequence_number: Value, txn_public_key: Value, txn_gas_price: Value, txn_max_gas_units: Value) returns ()
{
    call InitVerification();
    call LibraAccount_prologue(txn_sequence_number, txn_public_key, txn_gas_price, txn_max_gas_units);
}

procedure {:inline 1} LibraAccount_epilogue (txn_sequence_number: Value, txn_gas_price: Value, txn_max_gas_units: Value, gas_units_remaining: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var sender_account: Reference; // ReferenceType(LibraAccount_T_type_value())
    var transaction_fee_account: Reference; // ReferenceType(LibraAccount_T_type_value())
    var imm_sender_account: Reference; // ReferenceType(LibraAccount_T_type_value())
    var transaction_fee_amount: Value; // IntegerType()
    var transaction_fee: Value; // LibraCoin_T_type_value()
    var __t9: Value; // AddressType()
    var __t10: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t11: Value; // IntegerType()
    var __t12: Value; // IntegerType()
    var __t13: Value; // IntegerType()
    var __t14: Value; // IntegerType()
    var __t15: Value; // IntegerType()
    var __t16: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t17: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t18: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t19: Value; // IntegerType()
    var __t20: Value; // IntegerType()
    var __t21: Value; // BooleanType()
    var __t22: Value; // BooleanType()
    var __t23: Value; // IntegerType()
    var __t24: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t25: Value; // IntegerType()
    var __t26: Value; // LibraCoin_T_type_value()
    var __t27: Value; // IntegerType()
    var __t28: Value; // IntegerType()
    var __t29: Value; // IntegerType()
    var __t30: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t31: Reference; // ReferenceType(IntegerType())
    var __t32: Value; // AddressType()
    var __t33: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t34: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t35: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t36: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU64(txn_sequence_number);
    __m := UpdateLocal(__m, __frame + 0, txn_sequence_number);
    assume $DebugTrackLocal(5, 31, 0, 29902, txn_sequence_number);
    assume IsValidU64(txn_gas_price);
    __m := UpdateLocal(__m, __frame + 1, txn_gas_price);
    assume $DebugTrackLocal(5, 31, 1, 29902, txn_gas_price);
    assume IsValidU64(txn_max_gas_units);
    __m := UpdateLocal(__m, __frame + 2, txn_max_gas_units);
    assume $DebugTrackLocal(5, 31, 2, 29902, txn_max_gas_units);
    assume IsValidU64(gas_units_remaining);
    __m := UpdateLocal(__m, __frame + 3, gas_units_remaining);
    assume $DebugTrackLocal(5, 31, 3, 29902, gas_units_remaining);

    // increase the local counter
    __local_counter := __local_counter + 37;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := BorrowGlobal(GetLocal(__m, __frame + 9), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 31, 30348);
      goto Label_Abort;
    }

    call sender_account := CopyOrMoveRef(__t10);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account));
    assume $DebugTrackLocal(5, 31, 4, 30331, Dereference(__m, sender_account));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 12), GetLocal(__m, __frame + 13));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 31, 30483);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 14));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 31, 30460);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 15));
    __m := UpdateLocal(__m, __frame + 7, __tmp);
    assume $DebugTrackLocal(5, 31, 7, 30423, __tmp);

    call __t16 := CopyOrMoveRef(sender_account);

    call __t17 := FreezeRef(__t16);

    call imm_sender_account := CopyOrMoveRef(__t17);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, imm_sender_account));
    assume $DebugTrackLocal(5, 31, 6, 30545, Dereference(__m, imm_sender_account));

    call __t18 := CopyOrMoveRef(imm_sender_account);

    call __t19 := LibraAccount_balance_for_account(__t18);
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 31, 30624);
      goto Label_Abort;
    }
    assume IsValidU64(__t19);

    __m := UpdateLocal(__m, __frame + 19, __t19);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := Ge(GetLocal(__m, __frame + 19), GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 21));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    __tmp := GetLocal(__m, __frame + 22);
    if (!b#Boolean(__tmp)) { goto Label_20; }

    call __tmp := LdConst(6);
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    if (true) { assume $DebugTrackAbort(5, 31, 30720); }
    goto Label_Abort;

Label_20:
    call __t24 := CopyOrMoveRef(sender_account);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 25, __tmp);

    call __t26 := LibraAccount_withdraw_from_account(__t24, GetLocal(__m, __frame + 25));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 31, 30771);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t26);

    __m := UpdateLocal(__m, __frame + 26, __t26);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 26));
    __m := UpdateLocal(__m, __frame + 8, __tmp);
    assume $DebugTrackLocal(5, 31, 8, 30741, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 27, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 28, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 27), GetLocal(__m, __frame + 28));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 31, 30989);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 29, __tmp);

    call __t30 := CopyOrMoveRef(sender_account);

    call __t31 := BorrowField(__t30, LibraAccount_T_sequence_number);

    call WriteRef(__t31, GetLocal(__m, __frame + 29));


    call __tmp := LdAddr(4078);
    __m := UpdateLocal(__m, __frame + 32, __tmp);

    call __t33 := BorrowGlobal(GetLocal(__m, __frame + 32), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 31, 31118);
      goto Label_Abort;
    }

    call transaction_fee_account := CopyOrMoveRef(__t33);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, transaction_fee_account));
    assume $DebugTrackLocal(5, 31, 5, 31092, Dereference(__m, transaction_fee_account));

    call __t34 := CopyOrMoveRef(transaction_fee_account);

    call __t35 := BorrowField(__t34, LibraAccount_T_balance);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 36, __tmp);

    call LibraCoin_deposit(__t35, GetLocal(__m, __frame + 36));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 31, 31155);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_epilogue_verify (txn_sequence_number: Value, txn_gas_price: Value, txn_max_gas_units: Value, gas_units_remaining: Value) returns ()
{
    call InitVerification();
    call LibraAccount_epilogue(txn_sequence_number, txn_gas_price, txn_max_gas_units, gas_units_remaining);
}

procedure {:inline 1} LibraAccount_fresh_guid (counter: Reference, sender: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(__m, counter), LibraAccount_EventHandleGenerator_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(__m, counter), LibraAccount_EventHandleGenerator_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64())))) ==> __abort_flag;

{
    // declare local variables
    var count: Reference; // ReferenceType(IntegerType())
    var count_bytes: Value; // ByteArrayType()
    var preimage: Value; // ByteArrayType()
    var sender_bytes: Value; // ByteArrayType()
    var __t6: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var __t7: Reference; // ReferenceType(IntegerType())
    var __t8: Value; // AddressType()
    var __t9: Value; // ByteArrayType()
    var __t10: Reference; // ReferenceType(IntegerType())
    var __t11: Value; // IntegerType()
    var __t12: Value; // ByteArrayType()
    var __t13: Reference; // ReferenceType(IntegerType())
    var __t14: Value; // IntegerType()
    var __t15: Value; // IntegerType()
    var __t16: Value; // IntegerType()
    var __t17: Reference; // ReferenceType(IntegerType())
    var __t18: Value; // ByteArrayType()
    var __t19: Value; // ByteArrayType()
    var __t20: Value; // ByteArrayType()
    var __t21: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_EventHandleGenerator_is_well_formed(Dereference(__m, counter)) && IsValidReferenceParameter(__m, __local_counter, counter);
    assume $LibraAccount_EventHandleGenerator_is_well_formed(Dereference(__m, counter));
    assume $DebugTrackLocal(5, 32, 0, 31860, Dereference(__m, counter));
    assume is#Address(sender);
    __m := UpdateLocal(__m, __frame + 1, sender);
    assume $DebugTrackLocal(5, 32, 1, 31860, sender);

    // increase the local counter
    __local_counter := __local_counter + 22;

    // bytecode translation starts here
    call __t6 := CopyOrMoveRef(counter);

    call __t7 := BorrowField(__t6, LibraAccount_EventHandleGenerator_counter);

    call count := CopyOrMoveRef(__t7);
    assume IsValidU64(Dereference(__m, count));
    assume $DebugTrackLocal(5, 32, 2, 32140, Dereference(__m, count));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := AddressUtil_address_to_bytes(GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 32, 32199);
      goto Label_Abort;
    }
    assume is#ByteArray(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 5, __tmp);
    assume $DebugTrackLocal(5, 32, 5, 32184, __tmp);

    call __t10 := CopyOrMoveRef(count);

    call __tmp := ReadRef(__t10);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := U64Util_u64_to_bytes(GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 32, 32266);
      goto Label_Abort;
    }
    assume is#ByteArray(__t12);

    __m := UpdateLocal(__m, __frame + 12, __t12);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(5, 32, 3, 32252, __tmp);

    call __t13 := CopyOrMoveRef(count);

    call __tmp := ReadRef(__t13);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 32, 32325);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __t17 := CopyOrMoveRef(count);

    call WriteRef(__t17, GetLocal(__m, __frame + 16));
    assume $LibraAccount_EventHandleGenerator_is_well_formed(Dereference(__m, counter));
    assume $DebugTrackLocal(5, 32, 0, 32310, Dereference(__m, counter));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __t20 := BytearrayUtil_bytearray_concat(GetLocal(__m, __frame + 18), GetLocal(__m, __frame + 19));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 32, 32460);
      goto Label_Abort;
    }
    assume is#ByteArray(__t20);

    __m := UpdateLocal(__m, __frame + 20, __t20);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 4, __tmp);
    assume $DebugTrackLocal(5, 32, 4, 32449, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    __ret0 := GetLocal(__m, __frame + 21);
    assume $DebugTrackLocal(5, 32, 6, 32540, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_fresh_guid_verify (counter: Reference, sender: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_fresh_guid(counter, sender);
}

procedure {:inline 1} LibraAccount_new_event_handle_impl (tv0: TypeValue, counter: Reference, sender: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(__m, counter), LibraAccount_EventHandleGenerator_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(__m, counter), LibraAccount_EventHandleGenerator_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64())))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var __t4: Value; // AddressType()
    var __t5: Value; // ByteArrayType()
    var __t6: Value; // LibraAccount_EventHandle_type_value(tv0)
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_EventHandleGenerator_is_well_formed(Dereference(__m, counter)) && IsValidReferenceParameter(__m, __local_counter, counter);
    assume $LibraAccount_EventHandleGenerator_is_well_formed(Dereference(__m, counter));
    assume $DebugTrackLocal(5, 33, 0, 32671, Dereference(__m, counter));
    assume is#Address(sender);
    __m := UpdateLocal(__m, __frame + 1, sender);
    assume $DebugTrackLocal(5, 33, 1, 32671, sender);

    // increase the local counter
    __local_counter := __local_counter + 7;

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := CopyOrMoveRef(counter);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __t5 := LibraAccount_fresh_guid(__t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 33, 32894);
      goto Label_Abort;
    }
    assume is#ByteArray(__t5);

    __m := UpdateLocal(__m, __frame + 5, __t5);
    assume $LibraAccount_EventHandleGenerator_is_well_formed(Dereference(__m, counter));
    assume $DebugTrackLocal(5, 33, 0, 32894, Dereference(__m, counter));

    call __tmp := Pack_LibraAccount_EventHandle(0, 0, 0, 0, tv0, GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __ret0 := GetLocal(__m, __frame + 6);
    assume $DebugTrackLocal(5, 33, 2, 32853, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_new_event_handle_impl_verify (tv0: TypeValue, counter: Reference, sender: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_new_event_handle_impl(tv0, counter, sender);
}

procedure {:inline 1} LibraAccount_new_event_handle (tv0: TypeValue) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_event_generator), LibraAccount_EventHandleGenerator_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraAccount_T_event_generator), LibraAccount_EventHandleGenerator_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64())))) ==> __abort_flag;

{
    // declare local variables
    var sender_account_ref: Reference; // ReferenceType(LibraAccount_T_type_value())
    var sender_bytes: Value; // ByteArrayType()
    var __t2: Value; // AddressType()
    var __t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t5: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var __t6: Value; // AddressType()
    var __t7: Value; // LibraAccount_EventHandle_type_value(tv0)
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 34, 33335);
      goto Label_Abort;
    }

    call sender_account_ref := CopyOrMoveRef(__t3);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account_ref));
    assume $DebugTrackLocal(5, 34, 0, 33314, Dereference(__m, sender_account_ref));

    call __t4 := CopyOrMoveRef(sender_account_ref);

    call __t5 := BorrowField(__t4, LibraAccount_T_event_generator);

    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := LibraAccount_new_event_handle_impl(tv0, __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 34, 33390);
      goto Label_Abort;
    }
    assume $LibraAccount_EventHandle_is_well_formed(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    __ret0 := GetLocal(__m, __frame + 7);
    assume $DebugTrackLocal(5, 34, 2, 33383, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_new_event_handle_verify (tv0: TypeValue) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraAccount_new_event_handle(tv0);
}

procedure {:inline 1} LibraAccount_emit_event (tv0: TypeValue, handle_ref: Reference, msg: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(__m, handle_ref), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64()))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(__m, handle_ref), LibraAccount_EventHandle_counter)) + i#Integer(Integer(1)))) > i#Integer(max_u64())))) ==> __abort_flag;

{
    // declare local variables
    var count: Reference; // ReferenceType(IntegerType())
    var guid: Value; // ByteArrayType()
    var __t4: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(tv0))
    var __t5: Reference; // ReferenceType(ByteArrayType())
    var __t6: Value; // ByteArrayType()
    var __t7: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(tv0))
    var __t8: Reference; // ReferenceType(IntegerType())
    var __t9: Value; // ByteArrayType()
    var __t10: Reference; // ReferenceType(IntegerType())
    var __t11: Value; // IntegerType()
    var __t12: Value; // tv0
    var __t13: Reference; // ReferenceType(IntegerType())
    var __t14: Value; // IntegerType()
    var __t15: Value; // IntegerType()
    var __t16: Value; // IntegerType()
    var __t17: Reference; // ReferenceType(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_EventHandle_is_well_formed(Dereference(__m, handle_ref)) && IsValidReferenceParameter(__m, __local_counter, handle_ref);
    assume $LibraAccount_EventHandle_is_well_formed(Dereference(__m, handle_ref));
    assume $DebugTrackLocal(5, 35, 0, 33671, Dereference(__m, handle_ref));
    __m := UpdateLocal(__m, __frame + 1, msg);
    assume $DebugTrackLocal(5, 35, 1, 33671, msg);

    // increase the local counter
    __local_counter := __local_counter + 18;

    // bytecode translation starts here
    call __t4 := CopyOrMoveRef(handle_ref);

    call __t5 := BorrowField(__t4, LibraAccount_EventHandle_guid);

    call __tmp := ReadRef(__t5);
    assume is#ByteArray(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(5, 35, 3, 33878, __tmp);

    call __t7 := CopyOrMoveRef(handle_ref);

    call __t8 := BorrowField(__t7, LibraAccount_EventHandle_counter);

    call count := CopyOrMoveRef(__t8);
    assume IsValidU64(Dereference(__m, count));
    assume $DebugTrackLocal(5, 35, 2, 33918, Dereference(__m, count));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := CopyOrMoveRef(count);

    call __tmp := ReadRef(__t10);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call LibraAccount_write_to_event_store(tv0, GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 12));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 35, 33966);
      goto Label_Abort;
    }

    call __t13 := CopyOrMoveRef(count);

    call __tmp := ReadRef(__t13);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 35, 34056);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __t17 := CopyOrMoveRef(count);

    call WriteRef(__t17, GetLocal(__m, __frame + 16));
    assume $LibraAccount_EventHandle_is_well_formed(Dereference(__m, handle_ref));
    assume $DebugTrackLocal(5, 35, 0, 34041, Dereference(__m, handle_ref));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_emit_event_verify (tv0: TypeValue, handle_ref: Reference, msg: Value) returns ()
{
    call InitVerification();
    call LibraAccount_emit_event(tv0, handle_ref, msg);
}

procedure {:inline 1} LibraAccount_destroy_handle (tv0: TypeValue, handle: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var guid: Value; // ByteArrayType()
    var count: Value; // IntegerType()
    var __t3: Value; // LibraAccount_EventHandle_type_value(tv0)
    var __t4: Value; // IntegerType()
    var __t5: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $LibraAccount_EventHandle_is_well_formed(handle);
    __m := UpdateLocal(__m, __frame + 0, handle);
    assume $DebugTrackLocal(5, 37, 0, 34391, handle);

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4, __t5 := Unpack_LibraAccount_EventHandle(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __t4);
    __m := UpdateLocal(__m, __frame + 5, __t5);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(5, 37, 1, 34574, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(5, 37, 2, 34567, __tmp);

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_destroy_handle_verify (tv0: TypeValue, handle: Value) returns ()
{
    call InitVerification();
    call LibraAccount_destroy_handle(tv0, handle);
}
