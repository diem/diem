

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
    assume IsValidU64(total_value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, total_value));
}

procedure {:inline 1} Unpack_LibraCoin_MarketCap(_struct: Value) returns (total_value: Value)
{
    assume is#Vector(_struct);
    total_value := SelectField(_struct, LibraCoin_MarketCap_total_value);
    assume IsValidU64(total_value);
}



// ** functions of module LibraCoin

procedure {:inline 1} LibraCoin_mint_with_default_capability (amount: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)) == (Integer(i#Integer(old(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))) + i#Integer(amount)))));
ensures !__abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (amount)));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(amount) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(amount) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(amount) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(amount) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(LibraCoin_MintCapability_type_value())
    var t4: Value; // LibraCoin_T_type_value()
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

    call t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraCoin_MintCapability_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t4 := LibraCoin_mint(GetLocal(__m, __frame + 1), t3);
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t4);

    __m := UpdateLocal(__m, __frame + 4, t4);

    ret0 := GetLocal(__m, __frame + 4);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_mint_with_default_capability_verify (amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_mint_with_default_capability(amount);
}

procedure {:inline 1} LibraCoin_mint (value: Value, capability: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)) == (Integer(i#Integer(old(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))) + i#Integer(value)))));
ensures !__abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (value)));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(value) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(value) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(value) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(value) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var t2: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var t3: Value; // IntegerType()
    var t4: Reference; // ReferenceType(LibraCoin_MintCapability_type_value())
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // BooleanType()
    var t10: Value; // BooleanType()
    var t11: Value; // IntegerType()
    var t12: Value; // AddressType()
    var t13: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var t14: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var t15: Reference; // ReferenceType(IntegerType())
    var t16: Value; // IntegerType()
    var t17: Value; // IntegerType()
    var t18: Value; // IntegerType()
    var t19: Value; // IntegerType()
    var t20: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var t21: Reference; // ReferenceType(IntegerType())
    var t22: Value; // IntegerType()
    var t23: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 24;

    // process and type check arguments
    assume IsValidU64(value);
    __m := UpdateLocal(__m, __frame + 0, value);
    assume is#Vector(Dereference(__m, capability));
    assume IsValidReferenceParameter(__m, __frame, capability);

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(capability);

    // unimplemented instruction: NoOp

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(1000000000);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := LdConst(1000000);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Le(GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __tmp := GetLocal(__m, __frame + 10);
    if (!b#Boolean(__tmp)) { goto Label_11; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    goto Label_Abort;

Label_11:
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call t13 := BorrowGlobal(GetLocal(__m, __frame + 12), LibraCoin_MarketCap_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t13);

    call t14 := CopyOrMoveRef(t2);

    call t15 := BorrowField(t14, LibraCoin_MarketCap_total_value);

    call __tmp := ReadRef(t15);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 17), GetLocal(__m, __frame + 18));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call t20 := CopyOrMoveRef(t2);

    call t21 := BorrowField(t20, LibraCoin_MarketCap_total_value);

    call WriteRef(t21, GetLocal(__m, __frame + 19));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    call __tmp := Pack_LibraCoin_T(GetLocal(__m, __frame + 22));
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    ret0 := GetLocal(__m, __frame + 23);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_mint_verify (value: Value, capability: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_mint(value, capability);
}

procedure {:inline 1} LibraCoin_initialize () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn)))));
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(__txn)))));
ensures !__abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraCoin_MarketCap_total_value)) == (Integer(0))));
ensures old(!(b#Boolean(Boolean((Address(TxnSenderAddress(__txn))) != (Address(173345816)))) || b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn))))) || b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(__txn))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean((Address(TxnSenderAddress(__txn))) != (Address(173345816)))) || b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn))))) || b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))) ==> __abort_flag;

{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Value; // BooleanType()
    var t3: Value; // BooleanType()
    var t4: Value; // IntegerType()
    var t5: Value; // BooleanType()
    var t6: Value; // LibraCoin_MintCapability_type_value()
    var t7: Value; // IntegerType()
    var t8: Value; // LibraCoin_MarketCap_type_value()
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

procedure {:inline 1} LibraCoin_market_cap () returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean((ret0) == (SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))))))) ==> __abort_flag;

{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()
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

    call t1 := BorrowGlobal(GetLocal(__m, __frame + 0), LibraCoin_MarketCap_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t2 := BorrowField(t1, LibraCoin_MarketCap_total_value);

    call __tmp := ReadRef(t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_market_cap_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_market_cap();
}

procedure {:inline 1} LibraCoin_zero () returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (Integer(0))));
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // LibraCoin_T_type_value()
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

    ret0 := GetLocal(__m, __frame + 1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_zero_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_zero();
}

procedure {:inline 1} LibraCoin_value (coin_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean((ret0) == (SelectField(Dereference(__m, coin_ref), LibraCoin_T_value))));
{
    // declare local variables
    var t1: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()
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
    call t1 := CopyOrMoveRef(coin_ref);

    call t2 := BorrowField(t1, LibraCoin_T_value);

    call __tmp := ReadRef(t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_value_verify (coin_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_value(coin_ref);
}

procedure {:inline 1} LibraCoin_split (coin: Value, amount: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(b#Boolean(Boolean((SelectField(ret1, LibraCoin_T_value)) == (amount))) && b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(coin, LibraCoin_T_value))) - i#Integer(amount)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(SelectField(coin, LibraCoin_T_value)) < i#Integer(amount))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(SelectField(coin, LibraCoin_T_value)) < i#Integer(amount)))) ==> __abort_flag;

{
    // declare local variables
    var t2: Value; // LibraCoin_T_type_value()
    var t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t4: Value; // IntegerType()
    var t5: Value; // LibraCoin_T_type_value()
    var t6: Value; // LibraCoin_T_type_value()
    var t7: Value; // LibraCoin_T_type_value()
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
    call t3 := BorrowLoc(__frame + 0);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call t5 := LibraCoin_withdraw(t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t5);

    __m := UpdateLocal(__m, __frame + 5, t5);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    ret0 := GetLocal(__m, __frame + 6);
    ret1 := GetLocal(__m, __frame + 7);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
}

procedure LibraCoin_split_verify (coin: Value, amount: Value) returns (ret0: Value, ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0, ret1 := LibraCoin_split(coin, amount);
}

procedure {:inline 1} LibraCoin_withdraw (coin_ref: Reference, amount: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value))) - i#Integer(amount)))));
ensures !__abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (amount)));
ensures old(!(b#Boolean(Boolean(i#Integer(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) < i#Integer(amount))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) < i#Integer(amount)))) ==> __abort_flag;

{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t4: Reference; // ReferenceType(IntegerType())
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // BooleanType()
    var t9: Value; // BooleanType()
    var t10: Value; // IntegerType()
    var t11: Value; // IntegerType()
    var t12: Value; // IntegerType()
    var t13: Value; // IntegerType()
    var t14: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t15: Reference; // ReferenceType(IntegerType())
    var t16: Value; // IntegerType()
    var t17: Value; // LibraCoin_T_type_value()
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
    call t3 := CopyOrMoveRef(coin_ref);

    call t4 := BorrowField(t3, LibraCoin_T_value);

    call __tmp := ReadRef(t4);
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

    call t14 := CopyOrMoveRef(coin_ref);

    call t15 := BorrowField(t14, LibraCoin_T_value);

    call WriteRef(t15, GetLocal(__m, __frame + 13));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := Pack_LibraCoin_T(GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    ret0 := GetLocal(__m, __frame + 17);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_withdraw_verify (coin_ref: Reference, amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_withdraw(coin_ref, amount);
}

procedure {:inline 1} LibraCoin_join (coin1: Value, coin2: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(coin1, LibraCoin_T_value))) + i#Integer(old(SelectField(coin2, LibraCoin_T_value)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(coin1, LibraCoin_T_value)) + i#Integer(SelectField(coin2, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(coin1, LibraCoin_T_value)) + i#Integer(SelectField(coin2, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var t2: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t3: Value; // LibraCoin_T_type_value()
    var t4: Value; // LibraCoin_T_type_value()
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
    call t2 := BorrowLoc(__frame + 0);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call LibraCoin_deposit(t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    ret0 := GetLocal(__m, __frame + 4);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_join_verify (coin1: Value, coin2: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_join(coin1, coin2);
}

procedure {:inline 1} LibraCoin_deposit (coin_ref: Reference, check: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value))) + i#Integer(old(SelectField(check, LibraCoin_T_value)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) + i#Integer(SelectField(check, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) + i#Integer(SelectField(check, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t5: Reference; // ReferenceType(IntegerType())
    var t6: Value; // IntegerType()
    var t7: Value; // LibraCoin_T_type_value()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // IntegerType()
    var t11: Value; // IntegerType()
    var t12: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t13: Reference; // ReferenceType(IntegerType())
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
    call t4 := CopyOrMoveRef(coin_ref);

    call t5 := BorrowField(t4, LibraCoin_T_value);

    call __tmp := ReadRef(t5);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call t8 := Unpack_LibraCoin_T(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call t12 := CopyOrMoveRef(coin_ref);

    call t13 := BorrowField(t12, LibraCoin_T_value);

    call WriteRef(t13, GetLocal(__m, __frame + 11));

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
ensures old(!(b#Boolean(Boolean((SelectField(coin, LibraCoin_T_value)) != (Integer(0)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean((SelectField(coin, LibraCoin_T_value)) != (Integer(0))))) ==> __abort_flag;

{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // LibraCoin_T_type_value()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // BooleanType()
    var t7: Value; // BooleanType()
    var t8: Value; // IntegerType()
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

    call t3 := Unpack_LibraCoin_T(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, t3);

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
