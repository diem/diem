

// ** synthetics of module U64Util



// ** structs of module U64Util



// ** functions of module U64Util

procedure {:inline 1} U64Util_u64_to_bytes (i: Value) returns (__ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);



// ** synthetics of module AddressUtil



// ** structs of module AddressUtil



// ** functions of module AddressUtil

procedure {:inline 1} AddressUtil_address_to_bytes (addr: Value) returns (__ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);



// ** synthetics of module BytearrayUtil



// ** structs of module BytearrayUtil



// ** functions of module BytearrayUtil

procedure {:inline 1} BytearrayUtil_bytearray_concat (data1: Value, data2: Value) returns (__ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);



// ** synthetics of module Hash



// ** structs of module Hash



// ** functions of module Hash

procedure {:inline 1} Hash_sha2_256 (data: Value) returns (__ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);

procedure {:inline 1} Hash_sha3_256 (data: Value) returns (__ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);



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
        && IsValidU128(SelectField(__this, LibraCoin_MarketCap_total_value))
}

procedure {:inline 1} Pack_LibraCoin_MarketCap(module_idx: int, func_idx: int, var_idx: int, code_idx: int, total_value: Value) returns (_struct: Value)
{
    assume IsValidU128(total_value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, total_value));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraCoin_MarketCap(_struct: Value) returns (total_value: Value)
{
    assume is#Vector(_struct);
    total_value := SelectField(_struct, LibraCoin_MarketCap_total_value);
    assume IsValidU128(total_value);
}



// ** functions of module LibraCoin

procedure {:inline 1} LibraCoin_mint_with_default_capability (amount: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
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
    assume $DebugTrackLocal(5, 0, 0, 807, amount);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraCoin_MintCapability_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 0, 940);
      goto Label_Abort;
    }

    call __t4 := LibraCoin_mint(GetLocal(__m, __frame + 1), __t3);
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 0, 916);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(5, 0, 1, 909, __ret0);
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
{
    // declare local variables
    var total_value_ref: Reference; // ReferenceType(IntegerType())
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // BooleanType()
    var __t8: Value; // BooleanType()
    var __t9: Value; // IntegerType()
    var __t10: Value; // AddressType()
    var __t11: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var __t12: Reference; // ReferenceType(IntegerType())
    var __t13: Reference; // ReferenceType(IntegerType())
    var __t14: Value; // IntegerType()
    var __t15: Value; // IntegerType()
    var __t16: Value; // IntegerType()
    var __t17: Value; // IntegerType()
    var __t18: Reference; // ReferenceType(IntegerType())
    var __t19: Value; // IntegerType()
    var __t20: Value; // LibraCoin_T_type_value()
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
    assume $DebugTrackLocal(5, 1, 0, 1231, value);
    assume $LibraCoin_MintCapability_is_well_formed(Dereference(__m, capability)) && IsValidReferenceParameter(__m, __local_counter, capability);
    assume $LibraCoin_MintCapability_is_well_formed(Dereference(__m, capability));
    assume $DebugTrackLocal(5, 1, 1, 1231, Dereference(__m, capability));

    // increase the local counter
    __local_counter := __local_counter + 21;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(1000000000);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := LdConst(1000000);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 1, 1766);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Le(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    __tmp := GetLocal(__m, __frame + 8);
    if (!b#Boolean(__tmp)) { goto Label_9; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    if (true) { assume $DebugTrackAbort(5, 1, 1788); }
    goto Label_Abort;

Label_9:
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __t11 := BorrowGlobal(GetLocal(__m, __frame + 10), LibraCoin_MarketCap_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 1, 1925);
      goto Label_Abort;
    }

    call __t12 := BorrowField(__t11, LibraCoin_MarketCap_total_value);

    call total_value_ref := CopyOrMoveRef(__t12);
    assume IsValidU128(Dereference(__m, total_value_ref));
    assume $DebugTrackLocal(5, 1, 2, 1902, Dereference(__m, total_value_ref));

    call __t13 := CopyOrMoveRef(total_value_ref);

    call __tmp := ReadRef(__t13);
    assume IsValidU128(__tmp);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := CastU128(GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 1, 2036);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := AddU128(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 16));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 1, 2011);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __t18 := CopyOrMoveRef(total_value_ref);

    call WriteRef(__t18, GetLocal(__m, __frame + 17));


    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __tmp := Pack_LibraCoin_T(0, 0, 0, 0, GetLocal(__m, __frame + 19));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    __ret0 := GetLocal(__m, __frame + 20);
    assume $DebugTrackLocal(5, 1, 3, 2067, __ret0);
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

    if (true) { assume $DebugTrackAbort(5, 2, 2371); }
    goto Label_Abort;

Label_7:
    call __tmp := LdTrue();
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Pack_LibraCoin_MintCapability(0, 0, 0, 0, GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call MoveToSender(LibraCoin_MintCapability_type_value(), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 2, 2384);
      goto Label_Abort;
    }

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := Pack_LibraCoin_MarketCap(0, 0, 0, 0, GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call MoveToSender(LibraCoin_MarketCap_type_value(), GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 2, 2456);
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
      assume $DebugTrackAbort(5, 3, 2667);
      goto Label_Abort;
    }

    call __t2 := BorrowField(__t1, LibraCoin_MarketCap_total_value);

    call __tmp := ReadRef(__t2);
    assume IsValidU128(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(5, 3, 0, 2657, __ret0);
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
    assume $DebugTrackLocal(5, 4, 0, 2810, __ret0);
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
    assume $DebugTrackLocal(5, 5, 0, 2888, Dereference(__m, coin_ref));

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(coin_ref);

    call __t2 := BorrowField(__t1, LibraCoin_T_value);

    call __tmp := ReadRef(__t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(5, 5, 1, 2935, __ret0);
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
    assume $DebugTrackLocal(5, 6, 0, 3109, coin);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(5, 6, 1, 3109, amount);

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __t3 := BorrowLoc(__frame + 0, LibraCoin_T_type_value());

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __t5 := LibraCoin_withdraw(__t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 6, 3211);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t5);

    __m := UpdateLocal(__m, __frame + 5, __t5);
    assume $DebugTrackLocal(5, 6, 0, 3211, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(5, 6, 2, 3203, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __ret0 := GetLocal(__m, __frame + 6);
    assume $DebugTrackLocal(5, 6, 3, 3259, __ret0);
    __ret1 := GetLocal(__m, __frame + 7);
    assume $DebugTrackLocal(5, 6, 4, 3259, __ret1);
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
    assume $DebugTrackLocal(5, 7, 0, 3557, Dereference(__m, coin_ref));
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(5, 7, 1, 3557, amount);

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
    assume $DebugTrackLocal(5, 7, 2, 3713, __tmp);

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

    if (true) { assume $DebugTrackAbort(5, 7, 3795); }
    goto Label_Abort;

Label_11:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 12));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 7, 3866);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __t14 := CopyOrMoveRef(coin_ref);

    call __t15 := BorrowField(__t14, LibraCoin_T_value);

    call WriteRef(__t15, GetLocal(__m, __frame + 13));
    assume $LibraCoin_T_is_well_formed(Dereference(__m, coin_ref));
    assume $DebugTrackLocal(5, 7, 0, 3835, Dereference(__m, coin_ref));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := Pack_LibraCoin_T(0, 0, 0, 0, GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    assume $DebugTrackLocal(5, 7, 3, 3902, __ret0);
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
    assume $DebugTrackLocal(5, 8, 0, 4041, coin1);
    assume $LibraCoin_T_is_well_formed(coin2);
    __m := UpdateLocal(__m, __frame + 1, coin2);
    assume $DebugTrackLocal(5, 8, 1, 4041, coin2);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __t2 := BorrowLoc(__frame + 0, LibraCoin_T_type_value());

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call LibraCoin_deposit(__t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 8, 4102);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(5, 8, 0, 4102, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(5, 8, 2, 4149, __ret0);
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
    assume $DebugTrackLocal(5, 9, 0, 4352, Dereference(__m, coin_ref));
    assume $LibraCoin_T_is_well_formed(check);
    __m := UpdateLocal(__m, __frame + 1, check);
    assume $DebugTrackLocal(5, 9, 1, 4352, check);

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
    assume $DebugTrackLocal(5, 9, 2, 4470, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := Unpack_LibraCoin_T(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(5, 9, 3, 4520, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    if (__abort_flag) {
      assume $DebugTrackAbort(5, 9, 4594);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := CopyOrMoveRef(coin_ref);

    call __t13 := BorrowField(__t12, LibraCoin_T_value);

    call WriteRef(__t13, GetLocal(__m, __frame + 11));
    assume $LibraCoin_T_is_well_formed(Dereference(__m, coin_ref));
    assume $DebugTrackLocal(5, 9, 0, 4564, Dereference(__m, coin_ref));

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
    assume $DebugTrackLocal(5, 10, 0, 4858, coin);

    // increase the local counter
    __local_counter := __local_counter + 9;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := Unpack_LibraCoin_T(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(5, 10, 1, 4930, __tmp);

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

    if (true) { assume $DebugTrackAbort(5, 10, 4985); }
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



// ** synthetics of module LibraTimestamp



// ** structs of module LibraTimestamp

const unique LibraTimestamp_CurrentTimeMicroseconds: TypeName;
const LibraTimestamp_CurrentTimeMicroseconds_microseconds: FieldName;
axiom LibraTimestamp_CurrentTimeMicroseconds_microseconds == 0;
function LibraTimestamp_CurrentTimeMicroseconds_type_value(): TypeValue {
    StructType(LibraTimestamp_CurrentTimeMicroseconds, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
function {:inline 1} $LibraTimestamp_CurrentTimeMicroseconds_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, LibraTimestamp_CurrentTimeMicroseconds_microseconds))
}

procedure {:inline 1} Pack_LibraTimestamp_CurrentTimeMicroseconds(module_idx: int, func_idx: int, var_idx: int, code_idx: int, microseconds: Value) returns (_struct: Value)
{
    assume IsValidU64(microseconds);
    _struct := Vector(ExtendValueArray(EmptyValueArray, microseconds));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraTimestamp_CurrentTimeMicroseconds(_struct: Value) returns (microseconds: Value)
{
    assume is#Vector(_struct);
    microseconds := SelectField(_struct, LibraTimestamp_CurrentTimeMicroseconds_microseconds);
    assume IsValidU64(microseconds);
}



// ** functions of module LibraTimestamp

procedure {:inline 1} LibraTimestamp_initialize_timer () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var timer: Value; // LibraTimestamp_CurrentTimeMicroseconds_type_value()
    var __t1: Value; // AddressType()
    var __t2: Value; // AddressType()
    var __t3: Value; // BooleanType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // LibraTimestamp_CurrentTimeMicroseconds_type_value()
    var __t8: Value; // LibraTimestamp_CurrentTimeMicroseconds_type_value()
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
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 1), GetLocal(__m, __frame + 2)));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __tmp := GetLocal(__m, __frame + 4);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    if (true) { assume $DebugTrackAbort(6, 0, 406); }
    goto Label_Abort;

Label_7:
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Pack_LibraTimestamp_CurrentTimeMicroseconds(6, 0, 0, 498, GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(6, 0, 0, 490, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call MoveToSender(LibraTimestamp_CurrentTimeMicroseconds_type_value(), GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(6, 0, 549);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraTimestamp_initialize_timer_verify () returns ()
{
    call InitVerification();
    call LibraTimestamp_initialize_timer();
}

procedure {:inline 1} LibraTimestamp_update_global_time (proposer: Value, timestamp: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var global_timer: Reference; // ReferenceType(LibraTimestamp_CurrentTimeMicroseconds_type_value())
    var __t3: Value; // AddressType()
    var __t4: Reference; // ReferenceType(LibraTimestamp_CurrentTimeMicroseconds_type_value())
    var __t5: Value; // AddressType()
    var __t6: Value; // AddressType()
    var __t7: Value; // BooleanType()
    var __t8: Value; // IntegerType()
    var __t9: Reference; // ReferenceType(LibraTimestamp_CurrentTimeMicroseconds_type_value())
    var __t10: Reference; // ReferenceType(IntegerType())
    var __t11: Value; // IntegerType()
    var __t12: Value; // BooleanType()
    var __t13: Value; // BooleanType()
    var __t14: Value; // IntegerType()
    var __t15: Reference; // ReferenceType(LibraTimestamp_CurrentTimeMicroseconds_type_value())
    var __t16: Reference; // ReferenceType(IntegerType())
    var __t17: Value; // IntegerType()
    var __t18: Value; // IntegerType()
    var __t19: Value; // BooleanType()
    var __t20: Value; // BooleanType()
    var __t21: Value; // IntegerType()
    var __t22: Value; // IntegerType()
    var __t23: Reference; // ReferenceType(LibraTimestamp_CurrentTimeMicroseconds_type_value())
    var __t24: Reference; // ReferenceType(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(proposer);
    __m := UpdateLocal(__m, __frame + 0, proposer);
    assume $DebugTrackLocal(6, 1, 0, 743, proposer);
    assume IsValidU64(timestamp);
    __m := UpdateLocal(__m, __frame + 1, timestamp);
    assume $DebugTrackLocal(6, 1, 1, 743, timestamp);

    // increase the local counter
    __local_counter := __local_counter + 25;

    // bytecode translation starts here
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := BorrowGlobal(GetLocal(__m, __frame + 3), LibraTimestamp_CurrentTimeMicroseconds_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(6, 1, 1099);
      goto Label_Abort;
    }

    call global_timer := CopyOrMoveRef(__t4);
    assume $LibraTimestamp_CurrentTimeMicroseconds_is_well_formed(Dereference(__m, global_timer));
    assume $DebugTrackLocal(6, 1, 2, 1084, Dereference(__m, global_timer));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdAddr(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6)));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __tmp := GetLocal(__m, __frame + 7);
    if (!b#Boolean(__tmp)) { goto Label_17; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := CopyOrMoveRef(global_timer);

    call __t10 := BorrowField(__t9, LibraTimestamp_CurrentTimeMicroseconds_microseconds);

    call __tmp := ReadRef(__t10);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 8), GetLocal(__m, __frame + 11)));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    __tmp := GetLocal(__m, __frame + 13);
    if (!b#Boolean(__tmp)) { goto Label_16; }

    call __tmp := LdConst(5001);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    if (true) { assume $DebugTrackAbort(6, 1, 1347); }
    goto Label_Abort;

Label_16:
    goto Label_26;

Label_17:
    call __t15 := CopyOrMoveRef(global_timer);

    call __t16 := BorrowField(__t15, LibraTimestamp_CurrentTimeMicroseconds_microseconds);

    call __tmp := ReadRef(__t16);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := Lt(GetLocal(__m, __frame + 17), GetLocal(__m, __frame + 18));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 19));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    __tmp := GetLocal(__m, __frame + 20);
    if (!b#Boolean(__tmp)) { goto Label_26; }

    call __tmp := LdConst(5001);
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    if (true) { assume $DebugTrackAbort(6, 1, 1492); }
    goto Label_Abort;

Label_26:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    call __t23 := CopyOrMoveRef(global_timer);

    call __t24 := BorrowField(__t23, LibraTimestamp_CurrentTimeMicroseconds_microseconds);

    call WriteRef(__t24, GetLocal(__m, __frame + 22));


    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraTimestamp_update_global_time_verify (proposer: Value, timestamp: Value) returns ()
{
    call InitVerification();
    call LibraTimestamp_update_global_time(proposer, timestamp);
}

procedure {:inline 1} LibraTimestamp_now_microseconds () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var __t0: Value; // AddressType()
    var __t1: Reference; // ReferenceType(LibraTimestamp_CurrentTimeMicroseconds_type_value())
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

    call __t1 := BorrowGlobal(GetLocal(__m, __frame + 0), LibraTimestamp_CurrentTimeMicroseconds_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(6, 2, 1746);
      goto Label_Abort;
    }

    call __t2 := BorrowField(__t1, LibraTimestamp_CurrentTimeMicroseconds_microseconds);

    call __tmp := ReadRef(__t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(6, 2, 0, 1736, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraTimestamp_now_microseconds_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraTimestamp_now_microseconds();
}



// ** synthetics of module LibraTransactionTimeout



// ** structs of module LibraTransactionTimeout

const unique LibraTransactionTimeout_TTL: TypeName;
const LibraTransactionTimeout_TTL_duration_microseconds: FieldName;
axiom LibraTransactionTimeout_TTL_duration_microseconds == 0;
function LibraTransactionTimeout_TTL_type_value(): TypeValue {
    StructType(LibraTransactionTimeout_TTL, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
function {:inline 1} $LibraTransactionTimeout_TTL_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, LibraTransactionTimeout_TTL_duration_microseconds))
}

procedure {:inline 1} Pack_LibraTransactionTimeout_TTL(module_idx: int, func_idx: int, var_idx: int, code_idx: int, duration_microseconds: Value) returns (_struct: Value)
{
    assume IsValidU64(duration_microseconds);
    _struct := Vector(ExtendValueArray(EmptyValueArray, duration_microseconds));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraTransactionTimeout_TTL(_struct: Value) returns (duration_microseconds: Value)
{
    assume is#Vector(_struct);
    duration_microseconds := SelectField(_struct, LibraTransactionTimeout_TTL_duration_microseconds);
    assume IsValidU64(duration_microseconds);
}



// ** functions of module LibraTransactionTimeout

procedure {:inline 1} LibraTransactionTimeout_initialize () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var timeout: Value; // LibraTransactionTimeout_TTL_type_value()
    var __t1: Value; // AddressType()
    var __t2: Value; // AddressType()
    var __t3: Value; // BooleanType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // LibraTransactionTimeout_TTL_type_value()
    var __t8: Value; // LibraTransactionTimeout_TTL_type_value()
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
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 1), GetLocal(__m, __frame + 2)));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __tmp := GetLocal(__m, __frame + 4);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    if (true) { assume $DebugTrackAbort(7, 0, 394); }
    goto Label_Abort;

Label_7:
    call __tmp := LdConst(86400000000);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Pack_LibraTransactionTimeout_TTL(7, 0, 0, 451, GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(7, 0, 0, 441, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call MoveToSender(LibraTransactionTimeout_TTL_type_value(), GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(7, 0, 501);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraTransactionTimeout_initialize_verify () returns ()
{
    call InitVerification();
    call LibraTransactionTimeout_initialize();
}

procedure {:inline 1} LibraTransactionTimeout_set_timeout (new_duration: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var timeout: Reference; // ReferenceType(LibraTransactionTimeout_TTL_type_value())
    var __t2: Value; // AddressType()
    var __t3: Value; // AddressType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // AddressType()
    var __t8: Reference; // ReferenceType(LibraTransactionTimeout_TTL_type_value())
    var __t9: Value; // IntegerType()
    var __t10: Reference; // ReferenceType(LibraTransactionTimeout_TTL_type_value())
    var __t11: Reference; // ReferenceType(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU64(new_duration);
    __m := UpdateLocal(__m, __frame + 0, new_duration);
    assume $DebugTrackLocal(7, 1, 0, 564, new_duration);

    // increase the local counter
    __local_counter := __local_counter + 12;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3)));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    if (true) { assume $DebugTrackAbort(7, 1, 752); }
    goto Label_Abort;

Label_7:
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := BorrowGlobal(GetLocal(__m, __frame + 7), LibraTransactionTimeout_TTL_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(7, 1, 775);
      goto Label_Abort;
    }

    call timeout := CopyOrMoveRef(__t8);
    assume $LibraTransactionTimeout_TTL_is_well_formed(Dereference(__m, timeout));
    assume $DebugTrackLocal(7, 1, 1, 765, Dereference(__m, timeout));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := CopyOrMoveRef(timeout);

    call __t11 := BorrowField(__t10, LibraTransactionTimeout_TTL_duration_microseconds);

    call WriteRef(__t11, GetLocal(__m, __frame + 9));


    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraTransactionTimeout_set_timeout_verify (new_duration: Value) returns ()
{
    call InitVerification();
    call LibraTransactionTimeout_set_timeout(new_duration);
}

procedure {:inline 1} LibraTransactionTimeout_is_valid_transaction_timestamp (timestamp: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var current_block_time: Value; // IntegerType()
    var max_txn_time: Value; // IntegerType()
    var timeout: Value; // IntegerType()
    var txn_time_microseconds: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // BooleanType()
    var __t8: Value; // BooleanType()
    var __t9: Value; // IntegerType()
    var __t10: Value; // AddressType()
    var __t11: Reference; // ReferenceType(LibraTransactionTimeout_TTL_type_value())
    var __t12: Reference; // ReferenceType(IntegerType())
    var __t13: Value; // IntegerType()
    var __t14: Value; // IntegerType()
    var __t15: Value; // IntegerType()
    var __t16: Value; // IntegerType()
    var __t17: Value; // IntegerType()
    var __t18: Value; // IntegerType()
    var __t19: Value; // IntegerType()
    var __t20: Value; // IntegerType()
    var __t21: Value; // IntegerType()
    var __t22: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU64(timestamp);
    __m := UpdateLocal(__m, __frame + 0, timestamp);
    assume $DebugTrackLocal(7, 2, 0, 911, timestamp);

    // increase the local counter
    __local_counter := __local_counter + 23;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(9223372036854);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __tmp := GetLocal(__m, __frame + 7);
    if (!b#Boolean(__tmp)) { goto Label_6; }

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(7, 2, 5, 1242, __ret0);
    return;

Label_6:
    call __t9 := LibraTimestamp_now_microseconds();
    if (__abort_flag) {
      assume $DebugTrackAbort(7, 2, 1296);
      goto Label_Abort;
    }
    assume IsValidU64(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(7, 2, 1, 1275, __tmp);

    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __t11 := BorrowGlobal(GetLocal(__m, __frame + 10), LibraTransactionTimeout_TTL_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(7, 2, 1351);
      goto Label_Abort;
    }

    call __t12 := BorrowField(__t11, LibraTransactionTimeout_TTL_duration_microseconds);

    call __tmp := ReadRef(__t12);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 13));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(7, 2, 3, 1339, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(7, 2, 1427);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(7, 2, 2, 1412, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := LdConst(1000000);
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 17), GetLocal(__m, __frame + 18));
    if (__abort_flag) {
      assume $DebugTrackAbort(7, 2, 1502);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 19));
    __m := UpdateLocal(__m, __frame + 4, __tmp);
    assume $DebugTrackLocal(7, 2, 4, 1478, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    call __tmp := Lt(GetLocal(__m, __frame + 20), GetLocal(__m, __frame + 21));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    __ret0 := GetLocal(__m, __frame + 22);
    assume $DebugTrackLocal(7, 2, 5, 1893, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraTransactionTimeout_is_valid_transaction_timestamp_verify (timestamp: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraTransactionTimeout_is_valid_transaction_timestamp(timestamp);
}



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
axiom LibraAccount_T_type_value() == StructType(LibraAccount_T, ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, Vector_T_type_value(IntegerType())), LibraCoin_T_type_value()), BooleanType()), BooleanType()), LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())), LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())), IntegerType()), LibraAccount_EventHandleGenerator_type_value()));
function {:inline 1} $LibraAccount_T_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && $Vector_T_is_well_formed(SelectField(__this, LibraAccount_T_authentication_key))
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
    assume $Vector_T_is_well_formed(authentication_key);
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
    assume $Vector_T_is_well_formed(authentication_key);
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
    StructType(LibraAccount_SentPaymentEvent, ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), AddressType()), Vector_T_type_value(IntegerType())))
}
function {:inline 1} $LibraAccount_SentPaymentEvent_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, LibraAccount_SentPaymentEvent_amount))
        && is#Address(SelectField(__this, LibraAccount_SentPaymentEvent_payee))
        && $Vector_T_is_well_formed(SelectField(__this, LibraAccount_SentPaymentEvent_metadata))
}

procedure {:inline 1} Pack_LibraAccount_SentPaymentEvent(module_idx: int, func_idx: int, var_idx: int, code_idx: int, amount: Value, payee: Value, metadata: Value) returns (_struct: Value)
{
    assume IsValidU64(amount);
    assume is#Address(payee);
    assume $Vector_T_is_well_formed(metadata);
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
    assume $Vector_T_is_well_formed(metadata);
}

const unique LibraAccount_ReceivedPaymentEvent: TypeName;
const LibraAccount_ReceivedPaymentEvent_amount: FieldName;
axiom LibraAccount_ReceivedPaymentEvent_amount == 0;
const LibraAccount_ReceivedPaymentEvent_payer: FieldName;
axiom LibraAccount_ReceivedPaymentEvent_payer == 1;
const LibraAccount_ReceivedPaymentEvent_metadata: FieldName;
axiom LibraAccount_ReceivedPaymentEvent_metadata == 2;
function LibraAccount_ReceivedPaymentEvent_type_value(): TypeValue {
    StructType(LibraAccount_ReceivedPaymentEvent, ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), AddressType()), Vector_T_type_value(IntegerType())))
}
function {:inline 1} $LibraAccount_ReceivedPaymentEvent_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, LibraAccount_ReceivedPaymentEvent_amount))
        && is#Address(SelectField(__this, LibraAccount_ReceivedPaymentEvent_payer))
        && $Vector_T_is_well_formed(SelectField(__this, LibraAccount_ReceivedPaymentEvent_metadata))
}

procedure {:inline 1} Pack_LibraAccount_ReceivedPaymentEvent(module_idx: int, func_idx: int, var_idx: int, code_idx: int, amount: Value, payer: Value, metadata: Value) returns (_struct: Value)
{
    assume IsValidU64(amount);
    assume is#Address(payer);
    assume $Vector_T_is_well_formed(metadata);
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
    assume $Vector_T_is_well_formed(metadata);
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
    StructType(LibraAccount_EventHandle, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), Vector_T_type_value(IntegerType())))
}
function {:inline 1} $LibraAccount_EventHandle_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, LibraAccount_EventHandle_counter))
        && $Vector_T_is_well_formed(SelectField(__this, LibraAccount_EventHandle_guid))
}

procedure {:inline 1} Pack_LibraAccount_EventHandle(module_idx: int, func_idx: int, var_idx: int, code_idx: int, tv0: TypeValue, counter: Value, guid: Value) returns (_struct: Value)
{
    assume IsValidU64(counter);
    assume $Vector_T_is_well_formed(guid);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, counter), guid));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_LibraAccount_EventHandle(_struct: Value) returns (counter: Value, guid: Value)
{
    assume is#Vector(_struct);
    counter := SelectField(_struct, LibraAccount_EventHandle_counter);
    assume IsValidU64(counter);
    guid := SelectField(_struct, LibraAccount_EventHandle_guid);
    assume $Vector_T_is_well_formed(guid);
}



// ** functions of module LibraAccount

procedure {:inline 1} LibraAccount_deposit (payee: Value, to_deposit: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
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
    assume $DebugTrackLocal(8, 0, 0, 3302, payee);
    assume $LibraCoin_T_is_well_formed(to_deposit);
    __m := UpdateLocal(__m, __frame + 1, to_deposit);
    assume $DebugTrackLocal(8, 0, 1, 3302, to_deposit);

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
      assume $DebugTrackAbort(8, 0, 3379);
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
{
    // declare local variables
    var __t3: Value; // AddressType()
    var __t4: Value; // AddressType()
    var __t5: Value; // LibraCoin_T_type_value()
    var __t6: Value; // Vector_T_type_value(IntegerType())
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
    assume $DebugTrackLocal(8, 1, 0, 3564, payee);
    assume $LibraCoin_T_is_well_formed(to_deposit);
    __m := UpdateLocal(__m, __frame + 1, to_deposit);
    assume $DebugTrackLocal(8, 1, 1, 3564, to_deposit);
    assume $Vector_T_is_well_formed(metadata);
    __m := UpdateLocal(__m, __frame + 2, metadata);
    assume $DebugTrackLocal(8, 1, 2, 3564, metadata);

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
      assume $DebugTrackAbort(8, 1, 3707);
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
    var __t20: Value; // Vector_T_type_value(IntegerType())
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
    var __t31: Value; // Vector_T_type_value(IntegerType())
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
    assume $DebugTrackLocal(8, 2, 0, 4016, payee);
    assume is#Address(sender);
    __m := UpdateLocal(__m, __frame + 1, sender);
    assume $DebugTrackLocal(8, 2, 1, 4016, sender);
    assume $LibraCoin_T_is_well_formed(to_deposit);
    __m := UpdateLocal(__m, __frame + 2, to_deposit);
    assume $DebugTrackLocal(8, 2, 2, 4016, to_deposit);
    assume $Vector_T_is_well_formed(metadata);
    __m := UpdateLocal(__m, __frame + 3, metadata);
    assume $DebugTrackLocal(8, 2, 3, 4016, metadata);

    // increase the local counter
    __local_counter := __local_counter + 33;

    // bytecode translation starts here
    call __t7 := BorrowLoc(__frame + 2, LibraCoin_T_type_value());

    call __t8 := LibraCoin_value(__t7);
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 2, 4382);
      goto Label_Abort;
    }
    assume IsValidU64(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 4, __tmp);
    assume $DebugTrackLocal(8, 2, 4, 4366, __tmp);

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

    if (true) { assume $DebugTrackAbort(8, 2, 4452); }
    goto Label_Abort;

Label_10:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __t15 := BorrowGlobal(GetLocal(__m, __frame + 14), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 2, 4523);
      goto Label_Abort;
    }

    call sender_account_ref := CopyOrMoveRef(__t15);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account_ref));
    assume $DebugTrackLocal(8, 2, 6, 4502, Dereference(__m, sender_account_ref));

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
      assume $DebugTrackAbort(8, 2, 4595);
      goto Label_Abort;
    }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    call __t23 := BorrowGlobal(GetLocal(__m, __frame + 22), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 2, 4934);
      goto Label_Abort;
    }

    call payee_account_ref := CopyOrMoveRef(__t23);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, payee_account_ref));
    assume $DebugTrackLocal(8, 2, 5, 4914, Dereference(__m, payee_account_ref));

    call __t24 := CopyOrMoveRef(payee_account_ref);

    call __t25 := BorrowField(__t24, LibraAccount_T_balance);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 26, __tmp);

    call LibraCoin_deposit(__t25, GetLocal(__m, __frame + 26));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 2, 5018);
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
      assume $DebugTrackAbort(8, 2, 5133);
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
    assume $DebugTrackLocal(8, 3, 0, 5775, payee);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(8, 3, 1, 5775, amount);

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
      assume $DebugTrackAbort(8, 3, 5941);
      goto Label_Abort;
    }

Label_6:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := LibraCoin_mint_with_default_capability(GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 3, 6057);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);

    call LibraAccount_deposit(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 3, 6031);
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
    assume $DebugTrackLocal(8, 4, 0, 6236, Dereference(__m, account));
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(8, 4, 1, 6236, amount);

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __t3 := CopyOrMoveRef(account);

    call __t4 := BorrowField(__t3, LibraAccount_T_balance);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := LibraCoin_withdraw(__t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 4, 6369);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t6);

    __m := UpdateLocal(__m, __frame + 6, __t6);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(8, 4, 0, 6369, Dereference(__m, account));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(8, 4, 2, 6355, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __ret0 := GetLocal(__m, __frame + 7);
    assume $DebugTrackLocal(8, 4, 3, 6439, __ret0);
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
    assume $DebugTrackLocal(8, 5, 0, 6551, amount);

    // increase the local counter
    __local_counter := __local_counter + 11;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 5, 6685);
      goto Label_Abort;
    }

    call sender_account := CopyOrMoveRef(__t3);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account));
    assume $DebugTrackLocal(8, 5, 1, 6668, Dereference(__m, sender_account));

    call __t4 := CopyOrMoveRef(sender_account);

    call __t5 := BorrowField(__t4, LibraAccount_T_delegated_withdrawal_capability);

    call __tmp := ReadRef(__t5);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_9; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    if (true) { assume $DebugTrackAbort(8, 5, 6908); }
    goto Label_Abort;

Label_9:
    call __t8 := CopyOrMoveRef(sender_account);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := LibraAccount_withdraw_from_account(__t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 5, 7030);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t10);

    __m := UpdateLocal(__m, __frame + 10, __t10);

    __ret0 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(8, 5, 2, 7023, __ret0);
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
    assume $DebugTrackLocal(8, 6, 0, 7191, Dereference(__m, cap));
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(8, 6, 1, 7191, amount);

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
      assume $DebugTrackAbort(8, 6, 7363);
      goto Label_Abort;
    }

    call account := CopyOrMoveRef(__t6);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(8, 6, 2, 7353, Dereference(__m, account));

    call __t7 := CopyOrMoveRef(account);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := LibraAccount_withdraw_from_account(__t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 6, 7429);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    __ret0 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(8, 6, 3, 7422, __ret0);
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
    assume $DebugTrackLocal(8, 7, 0, 7801, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __t5 := BorrowGlobal(GetLocal(__m, __frame + 4), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 7, 7853);
      goto Label_Abort;
    }

    call sender_account := CopyOrMoveRef(__t5);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account));
    assume $DebugTrackLocal(8, 7, 1, 7836, Dereference(__m, sender_account));

    call __t6 := CopyOrMoveRef(sender_account);

    call __t7 := BorrowField(__t6, LibraAccount_T_delegated_withdrawal_capability);

    call delegated_ref := CopyOrMoveRef(__t7);
    assume is#Boolean(Dereference(__m, delegated_ref));
    assume $DebugTrackLocal(8, 7, 2, 7897, Dereference(__m, delegated_ref));

    call __t8 := CopyOrMoveRef(delegated_ref);

    call __tmp := ReadRef(__t8);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __tmp := GetLocal(__m, __frame + 9);
    if (!b#Boolean(__tmp)) { goto Label_13; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    if (true) { assume $DebugTrackAbort(8, 7, 8107); }
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
    assume $DebugTrackLocal(8, 7, 3, 8227, __ret0);
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
    assume $DebugTrackLocal(8, 8, 0, 8390, cap);

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Unpack_LibraAccount_WithdrawalCapability(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(8, 8, 1, 8610, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := BorrowGlobal(GetLocal(__m, __frame + 5), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 8, 8659);
      goto Label_Abort;
    }

    call account := CopyOrMoveRef(__t6);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(8, 8, 2, 8649, Dereference(__m, account));

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
    var __t15: Value; // Vector_T_type_value(IntegerType())
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
    assume $DebugTrackLocal(8, 9, 0, 9235, payee);
    assume $LibraAccount_WithdrawalCapability_is_well_formed(Dereference(__m, cap)) && IsValidReferenceParameter(__m, __local_counter, cap);
    assume $LibraAccount_WithdrawalCapability_is_well_formed(Dereference(__m, cap));
    assume $DebugTrackLocal(8, 9, 1, 9235, Dereference(__m, cap));
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 2, amount);
    assume $DebugTrackLocal(8, 9, 2, 9235, amount);
    assume $Vector_T_is_well_formed(metadata);
    __m := UpdateLocal(__m, __frame + 3, metadata);
    assume $DebugTrackLocal(8, 9, 3, 9235, metadata);

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
      assume $DebugTrackAbort(8, 9, 9448);
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
      assume $DebugTrackAbort(8, 9, 9617);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t14);

    __m := UpdateLocal(__m, __frame + 14, __t14);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call LibraAccount_deposit_with_sender_and_metadata(GetLocal(__m, __frame + 8), GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 9, 9500);
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
{
    // declare local variables
    var __t3: Value; // AddressType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // AddressType()
    var __t7: Value; // AddressType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // LibraCoin_T_type_value()
    var __t10: Value; // Vector_T_type_value(IntegerType())
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
    assume $DebugTrackLocal(8, 10, 0, 9947, payee);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(8, 10, 1, 9947, amount);
    assume $Vector_T_is_well_formed(metadata);
    __m := UpdateLocal(__m, __frame + 2, metadata);
    assume $DebugTrackLocal(8, 10, 2, 9947, metadata);

    // increase the local counter
    __local_counter := __local_counter + 11;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 3), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_6; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call LibraAccount_create_account(GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 10, 10129);
      goto Label_Abort;
    }

Label_6:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := LibraAccount_withdraw_from_sender(GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 10, 10246);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call LibraAccount_deposit_with_metadata(GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 10, 10181);
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
{
    // declare local variables
    var __t2: Value; // AddressType()
    var __t3: Value; // IntegerType()
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
    assume $DebugTrackLocal(8, 11, 0, 10531, payee);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume $DebugTrackLocal(8, 11, 1, 10531, amount);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    // unimplemented instruction: LdByteArray(4, ByteArrayPoolIndex(0))

    call LibraAccount_pay_from_sender_with_metadata(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 11, 10604);
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
{
    // declare local variables
    var __t2: Value; // Vector_T_type_value(IntegerType())
    var __t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t4: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
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
    assume $DebugTrackLocal(8, 12, 0, 10699, Dereference(__m, account));
    assume $Vector_T_is_well_formed(new_authentication_key);
    __m := UpdateLocal(__m, __frame + 1, new_authentication_key);
    assume $DebugTrackLocal(8, 12, 1, 10699, new_authentication_key);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := CopyOrMoveRef(account);

    call __t4 := BorrowField(__t3, LibraAccount_T_authentication_key);

    call WriteRef(__t4, GetLocal(__m, __frame + 2));
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(8, 12, 0, 10805, Dereference(__m, account));

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
    var __t9: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(new_authentication_key);
    __m := UpdateLocal(__m, __frame + 0, new_authentication_key);
    assume $DebugTrackLocal(8, 13, 0, 11027, new_authentication_key);

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 13, 11176);
      goto Label_Abort;
    }

    call sender_account := CopyOrMoveRef(__t3);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account));
    assume $DebugTrackLocal(8, 13, 1, 11159, Dereference(__m, sender_account));

    call __t4 := CopyOrMoveRef(sender_account);

    call __t5 := BorrowField(__t4, LibraAccount_T_delegated_key_rotation_capability);

    call __tmp := ReadRef(__t5);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_9; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    if (true) { assume $DebugTrackAbort(8, 13, 11389); }
    goto Label_Abort;

Label_9:
    call __t8 := CopyOrMoveRef(sender_account);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call LibraAccount_rotate_authentication_key_for_account(__t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 13, 11506);
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
{
    // declare local variables
    var __t2: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var __t3: Reference; // ReferenceType(AddressType())
    var __t4: Value; // AddressType()
    var __t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t6: Value; // Vector_T_type_value(IntegerType())
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
    assume $DebugTrackLocal(8, 14, 0, 11768, Dereference(__m, cap));
    assume $Vector_T_is_well_formed(new_authentication_key);
    __m := UpdateLocal(__m, __frame + 1, new_authentication_key);
    assume $DebugTrackLocal(8, 14, 1, 11768, new_authentication_key);

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
      assume $DebugTrackAbort(8, 14, 11988);
      goto Label_Abort;
    }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call LibraAccount_rotate_authentication_key_for_account(__t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 14, 11932);
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
    assume $DebugTrackLocal(8, 15, 0, 12379, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := BorrowGlobal(GetLocal(__m, __frame + 3), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 15, 12435);
      goto Label_Abort;
    }

    call __t5 := BorrowField(__t4, LibraAccount_T_delegated_key_rotation_capability);

    call delegated_ref := CopyOrMoveRef(__t5);
    assume is#Boolean(Dereference(__m, delegated_ref));
    assume $DebugTrackLocal(8, 15, 1, 12414, Dereference(__m, delegated_ref));

    call __t6 := CopyOrMoveRef(delegated_ref);

    call __tmp := ReadRef(__t6);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __tmp := GetLocal(__m, __frame + 7);
    if (!b#Boolean(__tmp)) { goto Label_11; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    if (true) { assume $DebugTrackAbort(8, 15, 12642); }
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
    assume $DebugTrackLocal(8, 15, 2, 12762, __ret0);
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
    assume $DebugTrackLocal(8, 16, 0, 12928, cap);

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Unpack_LibraAccount_KeyRotationCapability(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(8, 16, 1, 13152, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := BorrowGlobal(GetLocal(__m, __frame + 5), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 16, 13201);
      goto Label_Abort;
    }

    call account := CopyOrMoveRef(__t6);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, account));
    assume $DebugTrackLocal(8, 16, 2, 13191, Dereference(__m, account));

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
{
    // declare local variables
    var generator: Value; // LibraAccount_EventHandleGenerator_type_value()
    var __t2: Value; // IntegerType()
    var __t3: Value; // LibraAccount_EventHandleGenerator_type_value()
    var __t4: Value; // AddressType()
    var __t5: Value; // AddressType()
    var __t6: Value; // Vector_T_type_value(IntegerType())
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
    assume $DebugTrackLocal(8, 17, 0, 13765, fresh_address);

    // increase the local counter
    __local_counter := __local_counter + 19;

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := Pack_LibraAccount_EventHandleGenerator(8, 17, 1, 13884, GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(8, 17, 1, 13872, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := AddressUtil_address_to_bytes(GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 17, 14031);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t6);

    __m := UpdateLocal(__m, __frame + 6, __t6);

    call __t7 := LibraCoin_zero();
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 17, 14107);
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
      assume $DebugTrackAbort(8, 17, 14272);
      goto Label_Abort;
    }
    assume $LibraAccount_EventHandle_is_well_formed(__t12);

    __m := UpdateLocal(__m, __frame + 12, __t12);
    assume $DebugTrackLocal(8, 17, 1, 14272, GetLocal(__m, __frame + 1));

    call __t13 := BorrowLoc(__frame + 1, LibraAccount_EventHandleGenerator_type_value());

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __t15 := LibraAccount_new_event_handle_impl(LibraAccount_SentPaymentEvent_type_value(), __t13, GetLocal(__m, __frame + 14));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 17, 14393);
      goto Label_Abort;
    }
    assume $LibraAccount_EventHandle_is_well_formed(__t15);

    __m := UpdateLocal(__m, __frame + 15, __t15);
    assume $DebugTrackLocal(8, 17, 1, 14393, GetLocal(__m, __frame + 1));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := Pack_LibraAccount_T(0, 0, 0, 0, GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8), GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 12), GetLocal(__m, __frame + 15), GetLocal(__m, __frame + 16), GetLocal(__m, __frame + 17));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call LibraAccount_save_account(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 18));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 17, 13927);
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
    assume $DebugTrackLocal(8, 18, 0, 14748, fresh_address);
    assume IsValidU64(initial_balance);
    __m := UpdateLocal(__m, __frame + 1, initial_balance);
    assume $DebugTrackLocal(8, 18, 1, 14748, initial_balance);

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call LibraAccount_create_account(GetLocal(__m, __frame + 2));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 18, 14841);
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
      assume $DebugTrackAbort(8, 18, 14936);
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

procedure {:inline 1} LibraAccount_save_account (addr: Value, account: Value) returns ();
requires ExistsTxnSenderAccount(__m, __txn);

procedure {:inline 1} LibraAccount_balance_for_account (account: Reference) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
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
    assume $DebugTrackLocal(8, 20, 0, 15269, Dereference(__m, account));

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(account);

    call __t3 := BorrowField(__t2, LibraAccount_T_balance);

    call __t4 := LibraCoin_value(__t3);
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 20, 15370);
      goto Label_Abort;
    }
    assume IsValidU64(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(8, 20, 1, 15354, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(8, 20, 2, 15419, __ret0);
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
    assume $DebugTrackLocal(8, 21, 0, 15539, addr);

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 21, 15627);
      goto Label_Abort;
    }

    call __t3 := LibraAccount_balance_for_account(__t2);
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 21, 15602);
      goto Label_Abort;
    }
    assume IsValidU64(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(8, 21, 1, 15595, __ret0);
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
    assume $DebugTrackLocal(8, 22, 0, 15739, Dereference(__m, account));

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(account);

    call __t2 := BorrowField(__t1, LibraAccount_T_sequence_number);

    call __tmp := ReadRef(__t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(8, 22, 1, 15800, __ret0);
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
    assume $DebugTrackLocal(8, 23, 0, 15905, addr);

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 23, 16009);
      goto Label_Abort;
    }

    call __t3 := LibraAccount_sequence_number_for_account(__t2);
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 23, 15976);
      goto Label_Abort;
    }
    assume IsValidU64(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(8, 23, 1, 15969, __ret0);
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
    assume $DebugTrackLocal(8, 24, 0, 16136, addr);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 24, 16229);
      goto Label_Abort;
    }

    call __t3 := BorrowField(__t2, LibraAccount_T_delegated_key_rotation_capability);

    call __tmp := ReadRef(__t3);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(8, 24, 1, 16219, __ret0);
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
    assume $DebugTrackLocal(8, 25, 0, 16389, addr);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 25, 16480);
      goto Label_Abort;
    }

    call __t3 := BorrowField(__t2, LibraAccount_T_delegated_withdrawal_capability);

    call __tmp := ReadRef(__t3);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(8, 25, 1, 16470, __ret0);
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
    assume $DebugTrackLocal(8, 26, 0, 16644, Dereference(__m, cap));

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(cap);

    call __t2 := BorrowField(__t1, LibraAccount_WithdrawalCapability_account_address);

    __ret0 := __t2;
    assume is#Address(Dereference(__m, __ret0));
    assume $DebugTrackLocal(8, 26, 1, 16734, Dereference(__m, __ret0));
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
    assume $DebugTrackLocal(8, 27, 0, 16871, Dereference(__m, cap));

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(cap);

    call __t2 := BorrowField(__t1, LibraAccount_KeyRotationCapability_account_address);

    __ret0 := __t2;
    assume is#Address(Dereference(__m, __ret0));
    assume $DebugTrackLocal(8, 27, 1, 16964, Dereference(__m, __ret0));
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
    assume $DebugTrackLocal(8, 28, 0, 17061, check_addr);

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(8, 28, 1, 17112, __ret0);
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

procedure {:inline 1} LibraAccount_prologue (txn_sequence_number: Value, txn_public_key: Value, txn_gas_price: Value, txn_max_gas_units: Value, txn_expiration_time: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var transaction_sender: Value; // AddressType()
    var sender_account: Reference; // ReferenceType(LibraAccount_T_type_value())
    var imm_sender_account: Reference; // ReferenceType(LibraAccount_T_type_value())
    var max_transaction_fee: Value; // IntegerType()
    var balance_amount: Value; // IntegerType()
    var sequence_number_value: Value; // IntegerType()
    var __t11: Value; // AddressType()
    var __t12: Value; // AddressType()
    var __t13: Value; // BooleanType()
    var __t14: Value; // BooleanType()
    var __t15: Value; // IntegerType()
    var __t16: Value; // AddressType()
    var __t17: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t18: Value; // Vector_T_type_value(IntegerType())
    var __t19: Value; // Vector_T_type_value(IntegerType())
    var __t20: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t21: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t22: Value; // Vector_T_type_value(IntegerType())
    var __t23: Value; // BooleanType()
    var __t24: Value; // BooleanType()
    var __t25: Value; // IntegerType()
    var __t26: Value; // IntegerType()
    var __t27: Value; // IntegerType()
    var __t28: Value; // IntegerType()
    var __t29: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t30: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t31: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t32: Value; // IntegerType()
    var __t33: Value; // IntegerType()
    var __t34: Value; // IntegerType()
    var __t35: Value; // BooleanType()
    var __t36: Value; // BooleanType()
    var __t37: Value; // IntegerType()
    var __t38: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t39: Reference; // ReferenceType(IntegerType())
    var __t40: Value; // IntegerType()
    var __t41: Value; // IntegerType()
    var __t42: Value; // IntegerType()
    var __t43: Value; // BooleanType()
    var __t44: Value; // BooleanType()
    var __t45: Value; // IntegerType()
    var __t46: Value; // IntegerType()
    var __t47: Value; // IntegerType()
    var __t48: Value; // BooleanType()
    var __t49: Value; // BooleanType()
    var __t50: Value; // IntegerType()
    var __t51: Value; // IntegerType()
    var __t52: Value; // BooleanType()
    var __t53: Value; // BooleanType()
    var __t54: Value; // IntegerType()
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
    assume $DebugTrackLocal(8, 29, 0, 17461, txn_sequence_number);
    assume $Vector_T_is_well_formed(txn_public_key);
    __m := UpdateLocal(__m, __frame + 1, txn_public_key);
    assume $DebugTrackLocal(8, 29, 1, 17461, txn_public_key);
    assume IsValidU64(txn_gas_price);
    __m := UpdateLocal(__m, __frame + 2, txn_gas_price);
    assume $DebugTrackLocal(8, 29, 2, 17461, txn_gas_price);
    assume IsValidU64(txn_max_gas_units);
    __m := UpdateLocal(__m, __frame + 3, txn_max_gas_units);
    assume $DebugTrackLocal(8, 29, 3, 17461, txn_max_gas_units);
    assume IsValidU64(txn_expiration_time);
    __m := UpdateLocal(__m, __frame + 4, txn_expiration_time);
    assume $DebugTrackLocal(8, 29, 4, 17461, txn_expiration_time);

    // increase the local counter
    __local_counter := __local_counter + 55;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 5, __tmp);
    assume $DebugTrackLocal(8, 29, 5, 17897, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 12), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 13));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __tmp := GetLocal(__m, __frame + 14);
    if (!b#Boolean(__tmp)) { goto Label_8; }

    call __tmp := LdConst(5);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    if (true) { assume $DebugTrackAbort(8, 29, 18105); }
    goto Label_Abort;

Label_8:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __t17 := BorrowGlobal(GetLocal(__m, __frame + 16), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 29, 18184);
      goto Label_Abort;
    }

    call sender_account := CopyOrMoveRef(__t17);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account));
    assume $DebugTrackLocal(8, 29, 6, 18167, Dereference(__m, sender_account));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __t19 := Hash_sha3_256(GetLocal(__m, __frame + 18));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 29, 18355);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t19);

    __m := UpdateLocal(__m, __frame + 19, __t19);

    call __t20 := CopyOrMoveRef(sender_account);

    call __t21 := BorrowField(__t20, LibraAccount_T_authentication_key);

    call __tmp := ReadRef(__t21);
    assume $Vector_T_is_well_formed(__tmp);
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 19), GetLocal(__m, __frame + 22)));
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 23));
    __m := UpdateLocal(__m, __frame + 24, __tmp);

    __tmp := GetLocal(__m, __frame + 24);
    if (!b#Boolean(__tmp)) { goto Label_21; }

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 25, __tmp);

    if (true) { assume $DebugTrackAbort(8, 29, 18451); }
    goto Label_Abort;

Label_21:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 26, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 27, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 26), GetLocal(__m, __frame + 27));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 29, 18567);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 28, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 28));
    __m := UpdateLocal(__m, __frame + 8, __tmp);
    assume $DebugTrackLocal(8, 29, 8, 18545, __tmp);

    call __t29 := CopyOrMoveRef(sender_account);

    call __t30 := FreezeRef(__t29);

    call imm_sender_account := CopyOrMoveRef(__t30);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, imm_sender_account));
    assume $DebugTrackLocal(8, 29, 7, 18622, Dereference(__m, imm_sender_account));

    call __t31 := CopyOrMoveRef(imm_sender_account);

    call __t32 := LibraAccount_balance_for_account(__t31);
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 29, 18698);
      goto Label_Abort;
    }
    assume IsValidU64(__t32);

    __m := UpdateLocal(__m, __frame + 32, __t32);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 32));
    __m := UpdateLocal(__m, __frame + 9, __tmp);
    assume $DebugTrackLocal(8, 29, 9, 18681, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 33, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 34, __tmp);

    call __tmp := Ge(GetLocal(__m, __frame + 33), GetLocal(__m, __frame + 34));
    __m := UpdateLocal(__m, __frame + 35, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 35));
    __m := UpdateLocal(__m, __frame + 36, __tmp);

    __tmp := GetLocal(__m, __frame + 36);
    if (!b#Boolean(__tmp)) { goto Label_38; }

    call __tmp := LdConst(6);
    __m := UpdateLocal(__m, __frame + 37, __tmp);

    if (true) { assume $DebugTrackAbort(8, 29, 18816); }
    goto Label_Abort;

Label_38:
    call __t38 := CopyOrMoveRef(sender_account);

    call __t39 := BorrowField(__t38, LibraAccount_T_sequence_number);

    call __tmp := ReadRef(__t39);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 40, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 40));
    __m := UpdateLocal(__m, __frame + 10, __tmp);
    assume $DebugTrackLocal(8, 29, 10, 18926, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 41, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 10));
    __m := UpdateLocal(__m, __frame + 42, __tmp);

    call __tmp := Ge(GetLocal(__m, __frame + 41), GetLocal(__m, __frame + 42));
    __m := UpdateLocal(__m, __frame + 43, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 43));
    __m := UpdateLocal(__m, __frame + 44, __tmp);

    __tmp := GetLocal(__m, __frame + 44);
    if (!b#Boolean(__tmp)) { goto Label_49; }

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 45, __tmp);

    if (true) { assume $DebugTrackAbort(8, 29, 19069); }
    goto Label_Abort;

Label_49:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 46, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 10));
    __m := UpdateLocal(__m, __frame + 47, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 46), GetLocal(__m, __frame + 47)));
    __m := UpdateLocal(__m, __frame + 48, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 48));
    __m := UpdateLocal(__m, __frame + 49, __tmp);

    __tmp := GetLocal(__m, __frame + 49);
    if (!b#Boolean(__tmp)) { goto Label_56; }

    call __tmp := LdConst(4);
    __m := UpdateLocal(__m, __frame + 50, __tmp);

    if (true) { assume $DebugTrackAbort(8, 29, 19146); }
    goto Label_Abort;

Label_56:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 51, __tmp);

    call __t52 := LibraTransactionTimeout_is_valid_transaction_timestamp(GetLocal(__m, __frame + 51));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 29, 19165);
      goto Label_Abort;
    }
    assume is#Boolean(__t52);

    __m := UpdateLocal(__m, __frame + 52, __t52);

    call __tmp := Not(GetLocal(__m, __frame + 52));
    __m := UpdateLocal(__m, __frame + 53, __tmp);

    __tmp := GetLocal(__m, __frame + 53);
    if (!b#Boolean(__tmp)) { goto Label_62; }

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 54, __tmp);

    if (true) { assume $DebugTrackAbort(8, 29, 19248); }
    goto Label_Abort;

Label_62:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_prologue_verify (txn_sequence_number: Value, txn_public_key: Value, txn_gas_price: Value, txn_max_gas_units: Value, txn_expiration_time: Value) returns ()
{
    call InitVerification();
    call LibraAccount_prologue(txn_sequence_number, txn_public_key, txn_gas_price, txn_max_gas_units, txn_expiration_time);
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
    assume $DebugTrackLocal(8, 30, 0, 19391, txn_sequence_number);
    assume IsValidU64(txn_gas_price);
    __m := UpdateLocal(__m, __frame + 1, txn_gas_price);
    assume $DebugTrackLocal(8, 30, 1, 19391, txn_gas_price);
    assume IsValidU64(txn_max_gas_units);
    __m := UpdateLocal(__m, __frame + 2, txn_max_gas_units);
    assume $DebugTrackLocal(8, 30, 2, 19391, txn_max_gas_units);
    assume IsValidU64(gas_units_remaining);
    __m := UpdateLocal(__m, __frame + 3, gas_units_remaining);
    assume $DebugTrackLocal(8, 30, 3, 19391, gas_units_remaining);

    // increase the local counter
    __local_counter := __local_counter + 37;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := BorrowGlobal(GetLocal(__m, __frame + 9), LibraAccount_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 30, 19837);
      goto Label_Abort;
    }

    call sender_account := CopyOrMoveRef(__t10);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account));
    assume $DebugTrackLocal(8, 30, 4, 19820, Dereference(__m, sender_account));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 12), GetLocal(__m, __frame + 13));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 30, 19972);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 14));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 30, 19949);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 15));
    __m := UpdateLocal(__m, __frame + 7, __tmp);
    assume $DebugTrackLocal(8, 30, 7, 19912, __tmp);

    call __t16 := CopyOrMoveRef(sender_account);

    call __t17 := FreezeRef(__t16);

    call imm_sender_account := CopyOrMoveRef(__t17);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, imm_sender_account));
    assume $DebugTrackLocal(8, 30, 6, 20034, Dereference(__m, imm_sender_account));

    call __t18 := CopyOrMoveRef(imm_sender_account);

    call __t19 := LibraAccount_balance_for_account(__t18);
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 30, 20113);
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

    if (true) { assume $DebugTrackAbort(8, 30, 20209); }
    goto Label_Abort;

Label_20:
    call __t24 := CopyOrMoveRef(sender_account);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 25, __tmp);

    call __t26 := LibraAccount_withdraw_from_account(__t24, GetLocal(__m, __frame + 25));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 30, 20260);
      goto Label_Abort;
    }
    assume $LibraCoin_T_is_well_formed(__t26);

    __m := UpdateLocal(__m, __frame + 26, __t26);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 26));
    __m := UpdateLocal(__m, __frame + 8, __tmp);
    assume $DebugTrackLocal(8, 30, 8, 20230, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 27, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 28, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 27), GetLocal(__m, __frame + 28));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 30, 20478);
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
      assume $DebugTrackAbort(8, 30, 20607);
      goto Label_Abort;
    }

    call transaction_fee_account := CopyOrMoveRef(__t33);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, transaction_fee_account));
    assume $DebugTrackLocal(8, 30, 5, 20581, Dereference(__m, transaction_fee_account));

    call __t34 := CopyOrMoveRef(transaction_fee_account);

    call __t35 := BorrowField(__t34, LibraAccount_T_balance);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 36, __tmp);

    call LibraCoin_deposit(__t35, GetLocal(__m, __frame + 36));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 30, 20644);
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
{
    // declare local variables
    var count: Reference; // ReferenceType(IntegerType())
    var count_bytes: Value; // Vector_T_type_value(IntegerType())
    var preimage: Value; // Vector_T_type_value(IntegerType())
    var sender_bytes: Value; // Vector_T_type_value(IntegerType())
    var __t6: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var __t7: Reference; // ReferenceType(IntegerType())
    var __t8: Value; // AddressType()
    var __t9: Value; // Vector_T_type_value(IntegerType())
    var __t10: Reference; // ReferenceType(IntegerType())
    var __t11: Value; // IntegerType()
    var __t12: Value; // Vector_T_type_value(IntegerType())
    var __t13: Reference; // ReferenceType(IntegerType())
    var __t14: Value; // IntegerType()
    var __t15: Value; // IntegerType()
    var __t16: Value; // IntegerType()
    var __t17: Reference; // ReferenceType(IntegerType())
    var __t18: Value; // Vector_T_type_value(IntegerType())
    var __t19: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t20: Value; // Vector_T_type_value(IntegerType())
    var __t21: Value; // Vector_T_type_value(IntegerType())
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
    assume $DebugTrackLocal(8, 31, 0, 21350, Dereference(__m, counter));
    assume is#Address(sender);
    __m := UpdateLocal(__m, __frame + 1, sender);
    assume $DebugTrackLocal(8, 31, 1, 21350, sender);

    // increase the local counter
    __local_counter := __local_counter + 22;

    // bytecode translation starts here
    call __t6 := CopyOrMoveRef(counter);

    call __t7 := BorrowField(__t6, LibraAccount_EventHandleGenerator_counter);

    call count := CopyOrMoveRef(__t7);
    assume IsValidU64(Dereference(__m, count));
    assume $DebugTrackLocal(8, 31, 2, 21580, Dereference(__m, count));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := AddressUtil_address_to_bytes(GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 31, 21639);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 5, __tmp);
    assume $DebugTrackLocal(8, 31, 5, 21624, __tmp);

    call __t10 := CopyOrMoveRef(count);

    call __tmp := ReadRef(__t10);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := U64Util_u64_to_bytes(GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 31, 21706);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t12);

    __m := UpdateLocal(__m, __frame + 12, __t12);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(8, 31, 3, 21692, __tmp);

    call __t13 := CopyOrMoveRef(count);

    call __tmp := ReadRef(__t13);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 31, 21765);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __t17 := CopyOrMoveRef(count);

    call WriteRef(__t17, GetLocal(__m, __frame + 16));
    assume $DebugTrackLocal(8, 31, 4, 21750, GetLocal(__m, __frame + 4));
    assume $LibraAccount_EventHandleGenerator_is_well_formed(Dereference(__m, counter));
    assume $DebugTrackLocal(8, 31, 0, 21750, Dereference(__m, counter));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 18));
    __m := UpdateLocal(__m, __frame + 4, __tmp);
    assume $DebugTrackLocal(8, 31, 4, 21889, __tmp);

    call __t19 := BorrowLoc(__frame + 4, Vector_T_type_value(IntegerType()));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call Vector_append(IntegerType(), __t19, GetLocal(__m, __frame + 20));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 31, 21927);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(8, 31, 4, 21927, GetLocal(__m, __frame + 4));
    assume $LibraAccount_EventHandleGenerator_is_well_formed(Dereference(__m, counter));
    assume $DebugTrackLocal(8, 31, 0, 21927, Dereference(__m, counter));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    __ret0 := GetLocal(__m, __frame + 21);
    assume $DebugTrackLocal(8, 31, 6, 21989, __ret0);
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
{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var __t4: Value; // AddressType()
    var __t5: Value; // Vector_T_type_value(IntegerType())
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
    assume $DebugTrackLocal(8, 32, 0, 22120, Dereference(__m, counter));
    assume is#Address(sender);
    __m := UpdateLocal(__m, __frame + 1, sender);
    assume $DebugTrackLocal(8, 32, 1, 22120, sender);

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
      assume $DebugTrackAbort(8, 32, 22289);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t5);

    __m := UpdateLocal(__m, __frame + 5, __t5);
    assume $LibraAccount_EventHandleGenerator_is_well_formed(Dereference(__m, counter));
    assume $DebugTrackLocal(8, 32, 0, 22289, Dereference(__m, counter));

    call __tmp := Pack_LibraAccount_EventHandle(0, 0, 0, 0, tv0, GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __ret0 := GetLocal(__m, __frame + 6);
    assume $DebugTrackLocal(8, 32, 2, 22248, __ret0);
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
{
    // declare local variables
    var sender_account_ref: Reference; // ReferenceType(LibraAccount_T_type_value())
    var sender_bytes: Value; // Vector_T_type_value(IntegerType())
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
      assume $DebugTrackAbort(8, 33, 22642);
      goto Label_Abort;
    }

    call sender_account_ref := CopyOrMoveRef(__t3);
    assume $LibraAccount_T_is_well_formed(Dereference(__m, sender_account_ref));
    assume $DebugTrackLocal(8, 33, 0, 22621, Dereference(__m, sender_account_ref));

    call __t4 := CopyOrMoveRef(sender_account_ref);

    call __t5 := BorrowField(__t4, LibraAccount_T_event_generator);

    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := LibraAccount_new_event_handle_impl(tv0, __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(8, 33, 22697);
      goto Label_Abort;
    }
    assume $LibraAccount_EventHandle_is_well_formed(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    __ret0 := GetLocal(__m, __frame + 7);
    assume $DebugTrackLocal(8, 33, 2, 22690, __ret0);
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
{
    // declare local variables
    var count: Reference; // ReferenceType(IntegerType())
    var guid: Value; // Vector_T_type_value(IntegerType())
    var __t4: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(tv0))
    var __t5: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t6: Value; // Vector_T_type_value(IntegerType())
    var __t7: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(tv0))
    var __t8: Reference; // ReferenceType(IntegerType())
    var __t9: Value; // Vector_T_type_value(IntegerType())
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
    assume $DebugTrackLocal(8, 34, 0, 22979, Dereference(__m, handle_ref));
    __m := UpdateLocal(__m, __frame + 1, msg);
    assume $DebugTrackLocal(8, 34, 1, 22979, msg);

    // increase the local counter
    __local_counter := __local_counter + 18;

    // bytecode translation starts here
    call __t4 := CopyOrMoveRef(handle_ref);

    call __t5 := BorrowField(__t4, LibraAccount_EventHandle_guid);

    call __tmp := ReadRef(__t5);
    assume $Vector_T_is_well_formed(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(8, 34, 3, 23130, __tmp);

    call __t7 := CopyOrMoveRef(handle_ref);

    call __t8 := BorrowField(__t7, LibraAccount_EventHandle_counter);

    call count := CopyOrMoveRef(__t8);
    assume IsValidU64(Dereference(__m, count));
    assume $DebugTrackLocal(8, 34, 2, 23170, Dereference(__m, count));

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
      assume $DebugTrackAbort(8, 34, 23218);
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
      assume $DebugTrackAbort(8, 34, 23308);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __t17 := CopyOrMoveRef(count);

    call WriteRef(__t17, GetLocal(__m, __frame + 16));
    assume $LibraAccount_EventHandle_is_well_formed(Dereference(__m, handle_ref));
    assume $DebugTrackLocal(8, 34, 0, 23293, Dereference(__m, handle_ref));

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

procedure {:inline 1} LibraAccount_write_to_event_store (tv0: TypeValue, guid: Value, count: Value, msg: Value) returns ();
requires ExistsTxnSenderAccount(__m, __txn);

procedure {:inline 1} LibraAccount_destroy_handle (tv0: TypeValue, handle: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var guid: Value; // Vector_T_type_value(IntegerType())
    var count: Value; // IntegerType()
    var __t3: Value; // LibraAccount_EventHandle_type_value(tv0)
    var __t4: Value; // IntegerType()
    var __t5: Value; // Vector_T_type_value(IntegerType())
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
    assume $DebugTrackLocal(8, 36, 0, 23620, handle);

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
    assume $DebugTrackLocal(8, 36, 1, 23776, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(8, 36, 2, 23769, __tmp);

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



// ** synthetics of module AccessPathTest



// ** structs of module AccessPathTest

const unique AccessPathTest_ProverGhostTypes: TypeName;
const AccessPathTest_ProverGhostTypes_a: FieldName;
axiom AccessPathTest_ProverGhostTypes_a == 0;
const AccessPathTest_ProverGhostTypes_b: FieldName;
axiom AccessPathTest_ProverGhostTypes_b == 1;
function AccessPathTest_ProverGhostTypes_type_value(): TypeValue {
    StructType(AccessPathTest_ProverGhostTypes, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, LibraAccount_T_type_value()), LibraCoin_T_type_value()))
}
function {:inline 1} $AccessPathTest_ProverGhostTypes_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && $LibraAccount_T_is_well_formed(SelectField(__this, AccessPathTest_ProverGhostTypes_a))
        && $LibraCoin_T_is_well_formed(SelectField(__this, AccessPathTest_ProverGhostTypes_b))
}

procedure {:inline 1} Pack_AccessPathTest_ProverGhostTypes(module_idx: int, func_idx: int, var_idx: int, code_idx: int, a: Value, b: Value) returns (_struct: Value)
{
    assume $LibraAccount_T_is_well_formed(a);
    assume $LibraCoin_T_is_well_formed(b);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, a), b));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_AccessPathTest_ProverGhostTypes(_struct: Value) returns (a: Value, b: Value)
{
    assume is#Vector(_struct);
    a := SelectField(_struct, AccessPathTest_ProverGhostTypes_a);
    assume $LibraAccount_T_is_well_formed(a);
    b := SelectField(_struct, AccessPathTest_ProverGhostTypes_b);
    assume $LibraCoin_T_is_well_formed(b);
}



// ** functions of module AccessPathTest

procedure {:inline 1} AccessPathTest_fail_if_empty_balance (a: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(a)))))) || b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(a))), LibraAccount_T_balance), LibraCoin_T_value), Integer(0)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraAccount_T_type_value(), a#Address(a)))))) || b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, GetResourceReference(LibraAccount_T_type_value(), a#Address(a))), LibraAccount_T_balance), LibraCoin_T_value), Integer(0))))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Value; // AddressType()
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    assume $DebugTrackLocal(9, 0, 0, 943, a);

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := LibraAccount_balance(GetLocal(__m, __frame + 1));
    if (__abort_flag) {
      assume $DebugTrackAbort(9, 0, 1117);
      goto Label_Abort;
    }
    assume IsValidU64(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3)));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __tmp := GetLocal(__m, __frame + 4);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(77);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    if (true) { assume $DebugTrackAbort(9, 0, 1167); }
    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure AccessPathTest_fail_if_empty_balance_verify (a: Value) returns ()
{
    call InitVerification();
    call AccessPathTest_fail_if_empty_balance(a);
}
