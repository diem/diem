

// ** structs of module U64Util



// ** functions of module U64Util

procedure {:inline 1} U64Util_u64_to_bytes (i: Value) returns (__ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);



// ** structs of module AddressUtil



// ** functions of module AddressUtil

procedure {:inline 1} AddressUtil_address_to_bytes (addr: Value) returns (__ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);



// ** structs of module BytearrayUtil



// ** functions of module BytearrayUtil

procedure {:inline 1} BytearrayUtil_bytearray_concat (data1: Value, data2: Value) returns (__ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);



// ** structs of module Hash



// ** functions of module Hash

procedure {:inline 1} Hash_sha2_256 (data: Value) returns (__ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);

procedure {:inline 1} Hash_sha3_256 (data: Value) returns (__ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);



// ** structs of module LibraCoin

const unique LibraCoin_T: TypeName;
const LibraCoin_T_value: FieldName;
axiom LibraCoin_T_value == 0;
function LibraCoin_T_type_value(): TypeValue {
    StructType(LibraCoin_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_LibraCoin_T(value: Value) returns (_struct: Value)
{
    assume IsValidU64(value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, value));
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
procedure {:inline 1} Pack_LibraCoin_MintCapability(_dummy: Value) returns (_struct: Value)
{
    assume is#Boolean(_dummy);
    _struct := Vector(ExtendValueArray(EmptyValueArray, _dummy));
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
procedure {:inline 1} Pack_LibraCoin_MarketCap(total_value: Value) returns (_struct: Value)
{
    assume IsValidU128(total_value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, total_value));
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
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 0, amount);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraCoin_MintCapability_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __t4 := LibraCoin_mint(GetLocal(__m, __frame + 1), __t3);
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    __ret0 := GetLocal(__m, __frame + 4);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_mint_with_default_capability_verify (amount: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 21;

    // process and type check arguments
    assume IsValidU64(value);
    __m := UpdateLocal(__m, __frame + 0, value);
    assume is#Vector(Dereference(__m, capability));
    assume IsValidReferenceParameter(__m, __frame, capability);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(1000000000);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := LdConst(1000000);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Le(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    __tmp := GetLocal(__m, __frame + 8);
    if (!b#Boolean(__tmp)) { goto Label_9; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    goto Label_Abort;

Label_9:
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __t11 := BorrowGlobal(GetLocal(__m, __frame + 10), LibraCoin_MarketCap_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __t12 := BorrowField(__t11, LibraCoin_MarketCap_total_value);

    call total_value_ref := CopyOrMoveRef(__t12);

    call __t13 := CopyOrMoveRef(total_value_ref);

    call __tmp := ReadRef(__t13);
    assume IsValidU128(__tmp);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := CastU128(GetLocal(__m, __frame + 15));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := AddU128(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 16));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __t18 := CopyOrMoveRef(total_value_ref);

    call WriteRef(__t18, GetLocal(__m, __frame + 17));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __tmp := Pack_LibraCoin_T(GetLocal(__m, __frame + 19));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    __ret0 := GetLocal(__m, __frame + 20);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_mint_verify (value: Value, capability: Reference) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 9;

    // process and type check arguments

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

    goto Label_Abort;

Label_7:
    call __tmp := LdTrue();
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Pack_LibraCoin_MintCapability(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call MoveToSender(LibraCoin_MintCapability_type_value(), GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := Pack_LibraCoin_MarketCap(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call MoveToSender(LibraCoin_MarketCap_type_value(), GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraCoin_initialize_verify () returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 4;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t1 := BorrowGlobal(GetLocal(__m, __frame + 0), LibraCoin_MarketCap_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __t2 := BorrowField(__t1, LibraCoin_MarketCap_total_value);

    call __tmp := ReadRef(__t2);
    assume IsValidU128(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_market_cap_verify () returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 2;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := Pack_LibraCoin_T(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_zero_verify () returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume is#Vector(Dereference(__m, coin_ref));
    assume IsValidReferenceParameter(__m, __frame, coin_ref);

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(coin_ref);

    call __t2 := BorrowField(__t1, LibraCoin_T_value);

    call __tmp := ReadRef(__t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_value_verify (coin_ref: Reference) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 8;

    // process and type check arguments
    assume is#Vector(coin);
    __m := UpdateLocal(__m, __frame + 0, coin);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);

    // bytecode translation starts here
    call __t3 := BorrowLoc(__frame + 0);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __t5 := LibraCoin_withdraw(__t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t5);

    __m := UpdateLocal(__m, __frame + 5, __t5);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __ret0 := GetLocal(__m, __frame + 6);
    __ret1 := GetLocal(__m, __frame + 7);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure LibraCoin_split_verify (coin: Value, amount: Value) returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 18;

    // process and type check arguments
    assume is#Vector(Dereference(__m, coin_ref));
    assume IsValidReferenceParameter(__m, __frame, coin_ref);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);

    // bytecode translation starts here
    call __t3 := CopyOrMoveRef(coin_ref);

    call __t4 := BorrowField(__t3, LibraCoin_T_value);

    call __tmp := ReadRef(__t4);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

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

    goto Label_Abort;

Label_11:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 12));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __t14 := CopyOrMoveRef(coin_ref);

    call __t15 := BorrowField(__t14, LibraCoin_T_value);

    call WriteRef(__t15, GetLocal(__m, __frame + 13));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := Pack_LibraCoin_T(GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_withdraw_verify (coin_ref: Reference, amount: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume is#Vector(coin1);
    __m := UpdateLocal(__m, __frame + 0, coin1);
    assume is#Vector(coin2);
    __m := UpdateLocal(__m, __frame + 1, coin2);

    // bytecode translation starts here
    call __t2 := BorrowLoc(__frame + 0);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call LibraCoin_deposit(__t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraCoin_join_verify (coin1: Value, coin2: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 14;

    // process and type check arguments
    assume is#Vector(Dereference(__m, coin_ref));
    assume IsValidReferenceParameter(__m, __frame, coin_ref);
    assume is#Vector(check);
    __m := UpdateLocal(__m, __frame + 1, check);

    // bytecode translation starts here
    call __t4 := CopyOrMoveRef(coin_ref);

    call __t5 := BorrowField(__t4, LibraCoin_T_value);

    call __tmp := ReadRef(__t5);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := Unpack_LibraCoin_T(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := CopyOrMoveRef(coin_ref);

    call __t13 := BorrowField(__t12, LibraCoin_T_value);

    call WriteRef(__t13, GetLocal(__m, __frame + 11));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraCoin_deposit_verify (coin_ref: Reference, check: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 9;

    // process and type check arguments
    assume is#Vector(coin);
    __m := UpdateLocal(__m, __frame + 0, coin);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := Unpack_LibraCoin_T(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

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

    goto Label_Abort;

Label_10:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraCoin_destroy_zero_verify (coin: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraCoin_destroy_zero(coin);
}



// ** structs of module LibraTimestamp

const unique LibraTimestamp_CurrentTimeMicroseconds: TypeName;
const LibraTimestamp_CurrentTimeMicroseconds_microseconds: FieldName;
axiom LibraTimestamp_CurrentTimeMicroseconds_microseconds == 0;
function LibraTimestamp_CurrentTimeMicroseconds_type_value(): TypeValue {
    StructType(LibraTimestamp_CurrentTimeMicroseconds, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_LibraTimestamp_CurrentTimeMicroseconds(microseconds: Value) returns (_struct: Value)
{
    assume IsValidU64(microseconds);
    _struct := Vector(ExtendValueArray(EmptyValueArray, microseconds));
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
    __local_counter := __local_counter + 9;

    // process and type check arguments

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

    goto Label_Abort;

Label_7:
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Pack_LibraTimestamp_CurrentTimeMicroseconds(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call MoveToSender(LibraTimestamp_CurrentTimeMicroseconds_type_value(), GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraTimestamp_initialize_timer_verify () returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 25;

    // process and type check arguments
    assume is#Address(proposer);
    __m := UpdateLocal(__m, __frame + 0, proposer);
    assume IsValidU64(timestamp);
    __m := UpdateLocal(__m, __frame + 1, timestamp);

    // bytecode translation starts here
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := BorrowGlobal(GetLocal(__m, __frame + 3), LibraTimestamp_CurrentTimeMicroseconds_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call global_timer := CopyOrMoveRef(__t4);

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
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 4;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t1 := BorrowGlobal(GetLocal(__m, __frame + 0), LibraTimestamp_CurrentTimeMicroseconds_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __t2 := BorrowField(__t1, LibraTimestamp_CurrentTimeMicroseconds_microseconds);

    call __tmp := ReadRef(__t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraTimestamp_now_microseconds_verify () returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call __ret0 := LibraTimestamp_now_microseconds();
}



// ** structs of module LibraTransactionTimeout

const unique LibraTransactionTimeout_TTL: TypeName;
const LibraTransactionTimeout_TTL_duration_microseconds: FieldName;
axiom LibraTransactionTimeout_TTL_duration_microseconds == 0;
function LibraTransactionTimeout_TTL_type_value(): TypeValue {
    StructType(LibraTransactionTimeout_TTL, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_LibraTransactionTimeout_TTL(duration_microseconds: Value) returns (_struct: Value)
{
    assume IsValidU64(duration_microseconds);
    _struct := Vector(ExtendValueArray(EmptyValueArray, duration_microseconds));
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
    __local_counter := __local_counter + 9;

    // process and type check arguments

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

    goto Label_Abort;

Label_7:
    call __tmp := LdConst(86400000000);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Pack_LibraTransactionTimeout_TTL(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call MoveToSender(LibraTransactionTimeout_TTL_type_value(), GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraTransactionTimeout_initialize_verify () returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 12;

    // process and type check arguments
    assume IsValidU64(new_duration);
    __m := UpdateLocal(__m, __frame + 0, new_duration);

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

    goto Label_Abort;

Label_7:
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := BorrowGlobal(GetLocal(__m, __frame + 7), LibraTransactionTimeout_TTL_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call timeout := CopyOrMoveRef(__t8);

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
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 23;

    // process and type check arguments
    assume IsValidU64(timestamp);
    __m := UpdateLocal(__m, __frame + 0, timestamp);

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
    return;

Label_6:
    call __t9 := LibraTimestamp_now_microseconds();
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __t11 := BorrowGlobal(GetLocal(__m, __frame + 10), LibraTransactionTimeout_TTL_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __t12 := BorrowField(__t11, LibraTransactionTimeout_TTL_duration_microseconds);

    call __tmp := ReadRef(__t12);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 13));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := LdConst(1000000);
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 17), GetLocal(__m, __frame + 18));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 19));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    call __tmp := Lt(GetLocal(__m, __frame + 20), GetLocal(__m, __frame + 21));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    __ret0 := GetLocal(__m, __frame + 22);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraTransactionTimeout_is_valid_transaction_timestamp_verify (timestamp: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call __ret0 := LibraTransactionTimeout_is_valid_transaction_timestamp(timestamp);
}



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
procedure {:inline 1} Pack_LibraAccount_T(authentication_key: Value, balance: Value, delegated_key_rotation_capability: Value, delegated_withdrawal_capability: Value, received_events: Value, sent_events: Value, sequence_number: Value, event_generator: Value) returns (_struct: Value)
{
    assume is#ByteArray(authentication_key);
    assume is#Vector(balance);
    assume is#Boolean(delegated_key_rotation_capability);
    assume is#Boolean(delegated_withdrawal_capability);
    assume is#Vector(received_events);
    assume is#Vector(sent_events);
    assume IsValidU64(sequence_number);
    assume is#Vector(event_generator);
    _struct := Vector(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(EmptyValueArray, authentication_key), balance), delegated_key_rotation_capability), delegated_withdrawal_capability), received_events), sent_events), sequence_number), event_generator));
}

procedure {:inline 1} Unpack_LibraAccount_T(_struct: Value) returns (authentication_key: Value, balance: Value, delegated_key_rotation_capability: Value, delegated_withdrawal_capability: Value, received_events: Value, sent_events: Value, sequence_number: Value, event_generator: Value)
{
    assume is#Vector(_struct);
    authentication_key := SelectField(_struct, LibraAccount_T_authentication_key);
    assume is#ByteArray(authentication_key);
    balance := SelectField(_struct, LibraAccount_T_balance);
    assume is#Vector(balance);
    delegated_key_rotation_capability := SelectField(_struct, LibraAccount_T_delegated_key_rotation_capability);
    assume is#Boolean(delegated_key_rotation_capability);
    delegated_withdrawal_capability := SelectField(_struct, LibraAccount_T_delegated_withdrawal_capability);
    assume is#Boolean(delegated_withdrawal_capability);
    received_events := SelectField(_struct, LibraAccount_T_received_events);
    assume is#Vector(received_events);
    sent_events := SelectField(_struct, LibraAccount_T_sent_events);
    assume is#Vector(sent_events);
    sequence_number := SelectField(_struct, LibraAccount_T_sequence_number);
    assume IsValidU64(sequence_number);
    event_generator := SelectField(_struct, LibraAccount_T_event_generator);
    assume is#Vector(event_generator);
}

const unique LibraAccount_WithdrawalCapability: TypeName;
const LibraAccount_WithdrawalCapability_account_address: FieldName;
axiom LibraAccount_WithdrawalCapability_account_address == 0;
function LibraAccount_WithdrawalCapability_type_value(): TypeValue {
    StructType(LibraAccount_WithdrawalCapability, ExtendTypeValueArray(EmptyTypeValueArray, AddressType()))
}
procedure {:inline 1} Pack_LibraAccount_WithdrawalCapability(account_address: Value) returns (_struct: Value)
{
    assume is#Address(account_address);
    _struct := Vector(ExtendValueArray(EmptyValueArray, account_address));
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
procedure {:inline 1} Pack_LibraAccount_KeyRotationCapability(account_address: Value) returns (_struct: Value)
{
    assume is#Address(account_address);
    _struct := Vector(ExtendValueArray(EmptyValueArray, account_address));
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
procedure {:inline 1} Pack_LibraAccount_SentPaymentEvent(amount: Value, payee: Value, metadata: Value) returns (_struct: Value)
{
    assume IsValidU64(amount);
    assume is#Address(payee);
    assume is#ByteArray(metadata);
    _struct := Vector(ExtendValueArray(ExtendValueArray(ExtendValueArray(EmptyValueArray, amount), payee), metadata));
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
procedure {:inline 1} Pack_LibraAccount_ReceivedPaymentEvent(amount: Value, payer: Value, metadata: Value) returns (_struct: Value)
{
    assume IsValidU64(amount);
    assume is#Address(payer);
    assume is#ByteArray(metadata);
    _struct := Vector(ExtendValueArray(ExtendValueArray(ExtendValueArray(EmptyValueArray, amount), payer), metadata));
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
procedure {:inline 1} Pack_LibraAccount_EventHandleGenerator(counter: Value) returns (_struct: Value)
{
    assume IsValidU64(counter);
    _struct := Vector(ExtendValueArray(EmptyValueArray, counter));
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
procedure {:inline 1} Pack_LibraAccount_EventHandle(tv0: TypeValue, counter: Value, guid: Value) returns (_struct: Value)
{
    assume IsValidU64(counter);
    assume is#ByteArray(guid);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, counter), guid));
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
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume is#Vector(to_deposit);
    __m := UpdateLocal(__m, __frame + 1, to_deposit);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    // unimplemented instruction: LdByteArray(4, ByteArrayPoolIndex(0))

    call LibraAccount_deposit_with_metadata(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_deposit_verify (payee: Value, to_deposit: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_deposit(payee, to_deposit);
}

procedure {:inline 1} LibraAccount_deposit_with_metadata (payee: Value, to_deposit: Value, metadata: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 7;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume is#Vector(to_deposit);
    __m := UpdateLocal(__m, __frame + 1, to_deposit);
    assume is#ByteArray(metadata);
    __m := UpdateLocal(__m, __frame + 2, metadata);

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
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_deposit_with_metadata_verify (payee: Value, to_deposit: Value, metadata: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 33;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume is#Address(sender);
    __m := UpdateLocal(__m, __frame + 1, sender);
    assume is#Vector(to_deposit);
    __m := UpdateLocal(__m, __frame + 2, to_deposit);
    assume is#ByteArray(metadata);
    __m := UpdateLocal(__m, __frame + 3, metadata);

    // bytecode translation starts here
    call __t7 := BorrowLoc(__frame + 2);

    call __t8 := LibraCoin_value(__t7);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

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

    goto Label_Abort;

Label_10:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __t15 := BorrowGlobal(GetLocal(__m, __frame + 14), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call sender_account_ref := CopyOrMoveRef(__t15);

    call __t16 := CopyOrMoveRef(sender_account_ref);

    call __t17 := BorrowField(__t16, LibraAccount_T_sent_events);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := Pack_LibraAccount_SentPaymentEvent(GetLocal(__m, __frame + 18), GetLocal(__m, __frame + 19), GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    call LibraAccount_emit_event(LibraAccount_SentPaymentEvent_type_value(), __t17, GetLocal(__m, __frame + 21));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    call __t23 := BorrowGlobal(GetLocal(__m, __frame + 22), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call payee_account_ref := CopyOrMoveRef(__t23);

    call __t24 := CopyOrMoveRef(payee_account_ref);

    call __t25 := BorrowField(__t24, LibraAccount_T_balance);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 26, __tmp);

    call LibraCoin_deposit(__t25, GetLocal(__m, __frame + 26));
    if (__abort_flag) { goto Label_Abort; }

    call __t27 := CopyOrMoveRef(payee_account_ref);

    call __t28 := BorrowField(__t27, LibraAccount_T_received_events);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 29, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 30, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 31, __tmp);

    call __tmp := Pack_LibraAccount_ReceivedPaymentEvent(GetLocal(__m, __frame + 29), GetLocal(__m, __frame + 30), GetLocal(__m, __frame + 31));
    __m := UpdateLocal(__m, __frame + 32, __tmp);

    call LibraAccount_emit_event(LibraAccount_ReceivedPaymentEvent_type_value(), __t28, GetLocal(__m, __frame + 32));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_deposit_with_sender_and_metadata_verify (payee: Value, sender: Value, to_deposit: Value, metadata: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 9;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);

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
    if (__abort_flag) { goto Label_Abort; }

Label_6:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := LibraCoin_mint_with_default_capability(GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);

    call LibraAccount_deposit(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_mint_to_address_verify (payee: Value, amount: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 8;

    // process and type check arguments
    assume is#Vector(Dereference(__m, account));
    assume IsValidReferenceParameter(__m, __frame, account);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);

    // bytecode translation starts here
    call __t3 := CopyOrMoveRef(account);

    call __t4 := BorrowField(__t3, LibraAccount_T_balance);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := LibraCoin_withdraw(__t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t6);

    __m := UpdateLocal(__m, __frame + 6, __t6);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __ret0 := GetLocal(__m, __frame + 7);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_from_account_verify (account: Reference, amount: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 11;

    // process and type check arguments
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 0, amount);

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call sender_account := CopyOrMoveRef(__t3);

    call __t4 := CopyOrMoveRef(sender_account);

    call __t5 := BorrowField(__t4, LibraAccount_T_delegated_withdrawal_capability);

    call __tmp := ReadRef(__t5);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_9; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    goto Label_Abort;

Label_9:
    call __t8 := CopyOrMoveRef(sender_account);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := LibraAccount_withdraw_from_account(__t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t10);

    __m := UpdateLocal(__m, __frame + 10, __t10);

    __ret0 := GetLocal(__m, __frame + 10);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_from_sender_verify (amount: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 10;

    // process and type check arguments
    assume is#Vector(Dereference(__m, cap));
    assume IsValidReferenceParameter(__m, __frame, cap);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);

    // bytecode translation starts here
    call __t3 := CopyOrMoveRef(cap);

    call __t4 := BorrowField(__t3, LibraAccount_WithdrawalCapability_account_address);

    call __tmp := ReadRef(__t4);
    assume is#Address(__tmp);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := BorrowGlobal(GetLocal(__m, __frame + 5), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call account := CopyOrMoveRef(__t6);

    call __t7 := CopyOrMoveRef(account);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := LibraAccount_withdraw_from_account(__t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    __ret0 := GetLocal(__m, __frame + 9);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_with_capability_verify (cap: Reference, amount: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 15;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __t5 := BorrowGlobal(GetLocal(__m, __frame + 4), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call sender_account := CopyOrMoveRef(__t5);

    call __t6 := CopyOrMoveRef(sender_account);

    call __t7 := BorrowField(__t6, LibraAccount_T_delegated_withdrawal_capability);

    call delegated_ref := CopyOrMoveRef(__t7);

    call __t8 := CopyOrMoveRef(delegated_ref);

    call __tmp := ReadRef(__t8);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __tmp := GetLocal(__m, __frame + 9);
    if (!b#Boolean(__tmp)) { goto Label_13; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    goto Label_Abort;

Label_13:
    call __tmp := LdTrue();
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := CopyOrMoveRef(delegated_ref);

    call WriteRef(__t12, GetLocal(__m, __frame + 11));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := Pack_LibraAccount_WithdrawalCapability(GetLocal(__m, __frame + 13));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __ret0 := GetLocal(__m, __frame + 14);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_extract_sender_withdrawal_capability_verify () returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 10;

    // process and type check arguments
    assume is#Vector(cap);
    __m := UpdateLocal(__m, __frame + 0, cap);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Unpack_LibraAccount_WithdrawalCapability(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := BorrowGlobal(GetLocal(__m, __frame + 5), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call account := CopyOrMoveRef(__t6);

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
    assume ExistsTxnSenderAccount(__m, __txn);
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
    var __t15: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 16;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume is#Vector(Dereference(__m, cap));
    assume IsValidReferenceParameter(__m, __frame, cap);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 2, amount);
    assume is#ByteArray(metadata);
    __m := UpdateLocal(__m, __frame + 3, metadata);

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
    if (__abort_flag) { goto Label_Abort; }

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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t14);

    __m := UpdateLocal(__m, __frame + 14, __t14);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call LibraAccount_deposit_with_sender_and_metadata(GetLocal(__m, __frame + 8), GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_pay_from_capability_verify (payee: Value, cap: Reference, amount: Value, metadata: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    var __t10: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 11;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume is#ByteArray(metadata);
    __m := UpdateLocal(__m, __frame + 2, metadata);

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
    if (__abort_flag) { goto Label_Abort; }

Label_6:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := LibraAccount_withdraw_from_sender(GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call LibraAccount_deposit_with_metadata(GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_pay_from_sender_with_metadata_verify (payee: Value, amount: Value, metadata: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    // unimplemented instruction: LdByteArray(4, ByteArrayPoolIndex(0))

    call LibraAccount_pay_from_sender_with_metadata(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_pay_from_sender_verify (payee: Value, amount: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_pay_from_sender(payee, amount);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key_for_account (account: Reference, new_authentication_key: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume is#Vector(Dereference(__m, account));
    assume IsValidReferenceParameter(__m, __frame, account);
    assume is#ByteArray(new_authentication_key);
    __m := UpdateLocal(__m, __frame + 1, new_authentication_key);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := CopyOrMoveRef(account);

    call __t4 := BorrowField(__t3, LibraAccount_T_authentication_key);

    call WriteRef(__t4, GetLocal(__m, __frame + 2));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_rotate_authentication_key_for_account_verify (account: Reference, new_authentication_key: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    var __t9: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 10;

    // process and type check arguments
    assume is#ByteArray(new_authentication_key);
    __m := UpdateLocal(__m, __frame + 0, new_authentication_key);

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call sender_account := CopyOrMoveRef(__t3);

    call __t4 := CopyOrMoveRef(sender_account);

    call __t5 := BorrowField(__t4, LibraAccount_T_delegated_key_rotation_capability);

    call __tmp := ReadRef(__t5);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_9; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    goto Label_Abort;

Label_9:
    call __t8 := CopyOrMoveRef(sender_account);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call LibraAccount_rotate_authentication_key_for_account(__t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_rotate_authentication_key_verify (new_authentication_key: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    var __t6: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments
    assume is#Vector(Dereference(__m, cap));
    assume IsValidReferenceParameter(__m, __frame, cap);
    assume is#ByteArray(new_authentication_key);
    __m := UpdateLocal(__m, __frame + 1, new_authentication_key);

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(cap);

    call __t3 := BorrowField(__t2, LibraAccount_KeyRotationCapability_account_address);

    call __tmp := ReadRef(__t3);
    assume is#Address(__tmp);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __t5 := BorrowGlobal(GetLocal(__m, __frame + 4), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call LibraAccount_rotate_authentication_key_for_account(__t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_rotate_authentication_key_with_capability_verify (cap: Reference, new_authentication_key: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 13;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := BorrowGlobal(GetLocal(__m, __frame + 3), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __t5 := BorrowField(__t4, LibraAccount_T_delegated_key_rotation_capability);

    call delegated_ref := CopyOrMoveRef(__t5);

    call __t6 := CopyOrMoveRef(delegated_ref);

    call __tmp := ReadRef(__t6);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __tmp := GetLocal(__m, __frame + 7);
    if (!b#Boolean(__tmp)) { goto Label_11; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    goto Label_Abort;

Label_11:
    call __tmp := LdTrue();
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := CopyOrMoveRef(delegated_ref);

    call WriteRef(__t10, GetLocal(__m, __frame + 9));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := Pack_LibraAccount_KeyRotationCapability(GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    __ret0 := GetLocal(__m, __frame + 12);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_extract_sender_key_rotation_capability_verify () returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 10;

    // process and type check arguments
    assume is#Vector(cap);
    __m := UpdateLocal(__m, __frame + 0, cap);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Unpack_LibraAccount_KeyRotationCapability(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := BorrowGlobal(GetLocal(__m, __frame + 5), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call account := CopyOrMoveRef(__t6);

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
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 19;

    // process and type check arguments
    assume is#Address(fresh_address);
    __m := UpdateLocal(__m, __frame + 0, fresh_address);

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := Pack_LibraAccount_EventHandleGenerator(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := AddressUtil_address_to_bytes(GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    assume is#ByteArray(__t6);

    __m := UpdateLocal(__m, __frame + 6, __t6);

    call __t7 := LibraCoin_zero();
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := BorrowLoc(__frame + 1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := LibraAccount_new_event_handle_impl(LibraAccount_ReceivedPaymentEvent_type_value(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t12);

    __m := UpdateLocal(__m, __frame + 12, __t12);

    call __t13 := BorrowLoc(__frame + 1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __t15 := LibraAccount_new_event_handle_impl(LibraAccount_SentPaymentEvent_type_value(), __t13, GetLocal(__m, __frame + 14));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t15);

    __m := UpdateLocal(__m, __frame + 15, __t15);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := Pack_LibraAccount_T(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8), GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 12), GetLocal(__m, __frame + 15), GetLocal(__m, __frame + 16), GetLocal(__m, __frame + 17));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call LibraAccount_save_account(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 18));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_create_account_verify (fresh_address: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 8;

    // process and type check arguments
    assume is#Address(fresh_address);
    __m := UpdateLocal(__m, __frame + 0, fresh_address);
    assume IsValidU64(initial_balance);
    __m := UpdateLocal(__m, __frame + 1, initial_balance);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call LibraAccount_create_account(GetLocal(__m, __frame + 2));
    if (__abort_flag) { goto Label_Abort; }

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
    if (__abort_flag) { goto Label_Abort; }

Label_9:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_create_new_account_verify (fresh_address: Value, initial_balance: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 6;

    // process and type check arguments
    assume is#Vector(Dereference(__m, account));
    assume IsValidReferenceParameter(__m, __frame, account);

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(account);

    call __t3 := BorrowField(__t2, LibraAccount_T_balance);

    call __t4 := LibraCoin_value(__t3);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_balance_for_account_verify (account: Reference) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __t3 := LibraAccount_balance_for_account(__t2);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    __ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_balance_verify (addr: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume is#Vector(Dereference(__m, account));
    assume IsValidReferenceParameter(__m, __frame, account);

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(account);

    call __t2 := BorrowField(__t1, LibraAccount_T_sequence_number);

    call __tmp := ReadRef(__t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_sequence_number_for_account_verify (account: Reference) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __t3 := LibraAccount_sequence_number_for_account(__t2);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    __ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_sequence_number_verify (addr: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __t3 := BorrowField(__t2, LibraAccount_T_delegated_key_rotation_capability);

    call __tmp := ReadRef(__t3);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_delegated_key_rotation_capability_verify (addr: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __t3 := BorrowField(__t2, LibraAccount_T_delegated_withdrawal_capability);

    call __tmp := ReadRef(__t3);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_delegated_withdrawal_capability_verify (addr: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 3;

    // process and type check arguments
    assume is#Vector(Dereference(__m, cap));
    assume IsValidReferenceParameter(__m, __frame, cap);

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(cap);

    call __t2 := BorrowField(__t1, LibraAccount_WithdrawalCapability_account_address);

    __ret0 := __t2;
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultReference;
}

procedure LibraAccount_withdrawal_capability_address_verify (cap: Reference) returns (__ret0: Reference)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 3;

    // process and type check arguments
    assume is#Vector(Dereference(__m, cap));
    assume IsValidReferenceParameter(__m, __frame, cap);

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(cap);

    call __t2 := BorrowField(__t1, LibraAccount_KeyRotationCapability_account_address);

    __ret0 := __t2;
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultReference;
}

procedure LibraAccount_key_rotation_capability_address_verify (cap: Reference) returns (__ret0: Reference)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 3;

    // process and type check arguments
    assume is#Address(check_addr);
    __m := UpdateLocal(__m, __frame + 0, check_addr);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    __ret0 := GetLocal(__m, __frame + 2);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_exists_verify (check_addr: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    var __t18: Value; // ByteArrayType()
    var __t19: Value; // ByteArrayType()
    var __t20: Reference; // ReferenceType(LibraAccount_T_type_value())
    var __t21: Reference; // ReferenceType(ByteArrayType())
    var __t22: Value; // ByteArrayType()
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
    __local_counter := __local_counter + 55;

    // process and type check arguments
    assume IsValidU64(txn_sequence_number);
    __m := UpdateLocal(__m, __frame + 0, txn_sequence_number);
    assume is#ByteArray(txn_public_key);
    __m := UpdateLocal(__m, __frame + 1, txn_public_key);
    assume IsValidU64(txn_gas_price);
    __m := UpdateLocal(__m, __frame + 2, txn_gas_price);
    assume IsValidU64(txn_max_gas_units);
    __m := UpdateLocal(__m, __frame + 3, txn_max_gas_units);
    assume IsValidU64(txn_expiration_time);
    __m := UpdateLocal(__m, __frame + 4, txn_expiration_time);

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

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

    goto Label_Abort;

Label_8:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __t17 := BorrowGlobal(GetLocal(__m, __frame + 16), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call sender_account := CopyOrMoveRef(__t17);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __t19 := Hash_sha3_256(GetLocal(__m, __frame + 18));
    if (__abort_flag) { goto Label_Abort; }
    assume is#ByteArray(__t19);

    __m := UpdateLocal(__m, __frame + 19, __t19);

    call __t20 := CopyOrMoveRef(sender_account);

    call __t21 := BorrowField(__t20, LibraAccount_T_authentication_key);

    call __tmp := ReadRef(__t21);
    assume is#ByteArray(__tmp);
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 19), GetLocal(__m, __frame + 22)));
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 23));
    __m := UpdateLocal(__m, __frame + 24, __tmp);

    __tmp := GetLocal(__m, __frame + 24);
    if (!b#Boolean(__tmp)) { goto Label_21; }

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 25, __tmp);

    goto Label_Abort;

Label_21:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 26, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 27, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 26), GetLocal(__m, __frame + 27));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 28, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 28));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t29 := CopyOrMoveRef(sender_account);

    call __t30 := FreezeRef(__t29);

    call imm_sender_account := CopyOrMoveRef(__t30);

    call __t31 := CopyOrMoveRef(imm_sender_account);

    call __t32 := LibraAccount_balance_for_account(__t31);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t32);

    __m := UpdateLocal(__m, __frame + 32, __t32);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 32));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

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

    goto Label_Abort;

Label_38:
    call __t38 := CopyOrMoveRef(sender_account);

    call __t39 := BorrowField(__t38, LibraAccount_T_sequence_number);

    call __tmp := ReadRef(__t39);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 40, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 40));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

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

    goto Label_Abort;

Label_56:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 51, __tmp);

    call __t52 := LibraTransactionTimeout_is_valid_transaction_timestamp(GetLocal(__m, __frame + 51));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Boolean(__t52);

    __m := UpdateLocal(__m, __frame + 52, __t52);

    call __tmp := Not(GetLocal(__m, __frame + 52));
    __m := UpdateLocal(__m, __frame + 53, __tmp);

    __tmp := GetLocal(__m, __frame + 53);
    if (!b#Boolean(__tmp)) { goto Label_62; }

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 54, __tmp);

    goto Label_Abort;

Label_62:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_prologue_verify (txn_sequence_number: Value, txn_public_key: Value, txn_gas_price: Value, txn_max_gas_units: Value, txn_expiration_time: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 37;

    // process and type check arguments
    assume IsValidU64(txn_sequence_number);
    __m := UpdateLocal(__m, __frame + 0, txn_sequence_number);
    assume IsValidU64(txn_gas_price);
    __m := UpdateLocal(__m, __frame + 1, txn_gas_price);
    assume IsValidU64(txn_max_gas_units);
    __m := UpdateLocal(__m, __frame + 2, txn_max_gas_units);
    assume IsValidU64(gas_units_remaining);
    __m := UpdateLocal(__m, __frame + 3, gas_units_remaining);

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := BorrowGlobal(GetLocal(__m, __frame + 9), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call sender_account := CopyOrMoveRef(__t10);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 12), GetLocal(__m, __frame + 13));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 14));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 15));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t16 := CopyOrMoveRef(sender_account);

    call __t17 := FreezeRef(__t16);

    call imm_sender_account := CopyOrMoveRef(__t17);

    call __t18 := CopyOrMoveRef(imm_sender_account);

    call __t19 := LibraAccount_balance_for_account(__t18);
    if (__abort_flag) { goto Label_Abort; }
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

    goto Label_Abort;

Label_20:
    call __t24 := CopyOrMoveRef(sender_account);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 25, __tmp);

    call __t26 := LibraAccount_withdraw_from_account(__t24, GetLocal(__m, __frame + 25));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t26);

    __m := UpdateLocal(__m, __frame + 26, __t26);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 26));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 27, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 28, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 27), GetLocal(__m, __frame + 28));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 29, __tmp);

    call __t30 := CopyOrMoveRef(sender_account);

    call __t31 := BorrowField(__t30, LibraAccount_T_sequence_number);

    call WriteRef(__t31, GetLocal(__m, __frame + 29));

    call __tmp := LdAddr(4078);
    __m := UpdateLocal(__m, __frame + 32, __tmp);

    call __t33 := BorrowGlobal(GetLocal(__m, __frame + 32), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call transaction_fee_account := CopyOrMoveRef(__t33);

    call __t34 := CopyOrMoveRef(transaction_fee_account);

    call __t35 := BorrowField(__t34, LibraAccount_T_balance);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 36, __tmp);

    call LibraCoin_deposit(__t35, GetLocal(__m, __frame + 36));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_epilogue_verify (txn_sequence_number: Value, txn_gas_price: Value, txn_max_gas_units: Value, gas_units_remaining: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_epilogue(txn_sequence_number, txn_gas_price, txn_max_gas_units, gas_units_remaining);
}

procedure {:inline 1} LibraAccount_fresh_guid (counter: Reference, sender: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 22;

    // process and type check arguments
    assume is#Vector(Dereference(__m, counter));
    assume IsValidReferenceParameter(__m, __frame, counter);
    assume is#Address(sender);
    __m := UpdateLocal(__m, __frame + 1, sender);

    // bytecode translation starts here
    call __t6 := CopyOrMoveRef(counter);

    call __t7 := BorrowField(__t6, LibraAccount_EventHandleGenerator_counter);

    call count := CopyOrMoveRef(__t7);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := AddressUtil_address_to_bytes(GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }
    assume is#ByteArray(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t10 := CopyOrMoveRef(count);

    call __tmp := ReadRef(__t10);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := U64Util_u64_to_bytes(GetLocal(__m, __frame + 11));
    if (__abort_flag) { goto Label_Abort; }
    assume is#ByteArray(__t12);

    __m := UpdateLocal(__m, __frame + 12, __t12);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t13 := CopyOrMoveRef(count);

    call __tmp := ReadRef(__t13);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __t17 := CopyOrMoveRef(count);

    call WriteRef(__t17, GetLocal(__m, __frame + 16));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __t20 := BytearrayUtil_bytearray_concat(GetLocal(__m, __frame + 18), GetLocal(__m, __frame + 19));
    if (__abort_flag) { goto Label_Abort; }
    assume is#ByteArray(__t20);

    __m := UpdateLocal(__m, __frame + 20, __t20);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    __ret0 := GetLocal(__m, __frame + 21);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_fresh_guid_verify (counter: Reference, sender: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call __ret0 := LibraAccount_fresh_guid(counter, sender);
}

procedure {:inline 1} LibraAccount_new_event_handle_impl (tv0: TypeValue, counter: Reference, sender: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 7;

    // process and type check arguments
    assume is#Vector(Dereference(__m, counter));
    assume IsValidReferenceParameter(__m, __frame, counter);
    assume is#Address(sender);
    __m := UpdateLocal(__m, __frame + 1, sender);

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := CopyOrMoveRef(counter);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __t5 := LibraAccount_fresh_guid(__t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    assume is#ByteArray(__t5);

    __m := UpdateLocal(__m, __frame + 5, __t5);

    call __tmp := Pack_LibraAccount_EventHandle(tv0, GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __ret0 := GetLocal(__m, __frame + 6);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_new_event_handle_impl_verify (tv0: TypeValue, counter: Reference, sender: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call __ret0 := LibraAccount_new_event_handle_impl(tv0, counter, sender);
}

procedure {:inline 1} LibraAccount_new_event_handle (tv0: TypeValue) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 8;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call sender_account_ref := CopyOrMoveRef(__t3);

    call __t4 := CopyOrMoveRef(sender_account_ref);

    call __t5 := BorrowField(__t4, LibraAccount_T_event_generator);

    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := LibraAccount_new_event_handle_impl(tv0, __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    __ret0 := GetLocal(__m, __frame + 7);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure LibraAccount_new_event_handle_verify (tv0: TypeValue) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call __ret0 := LibraAccount_new_event_handle(tv0);
}

procedure {:inline 1} LibraAccount_emit_event (tv0: TypeValue, handle_ref: Reference, msg: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 18;

    // process and type check arguments
    assume is#Vector(Dereference(__m, handle_ref));
    assume IsValidReferenceParameter(__m, __frame, handle_ref);
    __m := UpdateLocal(__m, __frame + 1, msg);

    // bytecode translation starts here
    call __t4 := CopyOrMoveRef(handle_ref);

    call __t5 := BorrowField(__t4, LibraAccount_EventHandle_guid);

    call __tmp := ReadRef(__t5);
    assume is#ByteArray(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t7 := CopyOrMoveRef(handle_ref);

    call __t8 := BorrowField(__t7, LibraAccount_EventHandle_counter);

    call count := CopyOrMoveRef(__t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := CopyOrMoveRef(count);

    call __tmp := ReadRef(__t10);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call LibraAccount_write_to_event_store(tv0, GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 12));
    if (__abort_flag) { goto Label_Abort; }

    call __t13 := CopyOrMoveRef(count);

    call __tmp := ReadRef(__t13);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __t17 := CopyOrMoveRef(count);

    call WriteRef(__t17, GetLocal(__m, __frame + 16));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_emit_event_verify (tv0: TypeValue, handle_ref: Reference, msg: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_emit_event(tv0, handle_ref, msg);
}

procedure {:inline 1} LibraAccount_write_to_event_store (tv0: TypeValue, guid: Value, count: Value, msg: Value) returns ();
requires ExistsTxnSenderAccount(__m, __txn);

procedure {:inline 1} LibraAccount_destroy_handle (tv0: TypeValue, handle: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 6;

    // process and type check arguments
    assume is#Vector(handle);
    __m := UpdateLocal(__m, __frame + 0, handle);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4, __t5 := Unpack_LibraAccount_EventHandle(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __t4);
    __m := UpdateLocal(__m, __frame + 5, __t5);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_destroy_handle_verify (tv0: TypeValue, handle: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_destroy_handle(tv0, handle);
}



// ** structs of module AccessPathTest

const unique AccessPathTest_ProverGhostTypes: TypeName;
const AccessPathTest_ProverGhostTypes_a: FieldName;
axiom AccessPathTest_ProverGhostTypes_a == 0;
const AccessPathTest_ProverGhostTypes_b: FieldName;
axiom AccessPathTest_ProverGhostTypes_b == 1;
function AccessPathTest_ProverGhostTypes_type_value(): TypeValue {
    StructType(AccessPathTest_ProverGhostTypes, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, LibraAccount_T_type_value()), LibraCoin_T_type_value()))
}
procedure {:inline 1} Pack_AccessPathTest_ProverGhostTypes(a: Value, b: Value) returns (_struct: Value)
{
    assume is#Vector(a);
    assume is#Vector(b);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, a), b));
}

procedure {:inline 1} Unpack_AccessPathTest_ProverGhostTypes(_struct: Value) returns (a: Value, b: Value)
{
    assume is#Vector(_struct);
    a := SelectField(_struct, AccessPathTest_ProverGhostTypes_a);
    assume is#Vector(a);
    b := SelectField(_struct, AccessPathTest_ProverGhostTypes_b);
    assume is#Vector(b);
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
    __local_counter := __local_counter + 6;

    // process and type check arguments
    assume is#Address(a);
    __m := UpdateLocal(__m, __frame + 0, a);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := LibraAccount_balance(GetLocal(__m, __frame + 1));
    if (__abort_flag) { goto Label_Abort; }
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

    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure AccessPathTest_fail_if_empty_balance_verify (a: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call AccessPathTest_fail_if_empty_balance(a);
}
