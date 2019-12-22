

// ** structs of module LibraCoin

const unique LibraCoin_T: TypeName;
const LibraCoin_T_value: FieldName;
axiom LibraCoin_T_value == 0;
function LibraCoin_T_type_value(): TypeValue {
    StructType(LibraCoin_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_LibraCoin_T(value: Value) returns (_struct: Value)
{
    assume IsValidInteger(value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, value));

}

procedure {:inline 1} Unpack_LibraCoin_T(_struct: Value) returns (value: Value)
{
    assume is#Vector(_struct);
    value := SelectField(_struct, LibraCoin_T_value);
    assume IsValidInteger(value);
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
    assume IsValidInteger(total_value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, total_value));

}

procedure {:inline 1} Unpack_LibraCoin_MarketCap(_struct: Value) returns (total_value: Value)
{
    assume is#Vector(_struct);
    total_value := SelectField(_struct, LibraCoin_MarketCap_total_value);
    assume IsValidInteger(total_value);
}



// ** functions of module LibraCoin

procedure {:inline 1} LibraCoin_mint_with_default_capability (amount: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)) == (Integer(i#Integer(old(SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))) + i#Integer(amount)))));
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (amount)));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(amount) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(amount) + i#Integer(SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(amount) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(amount) + i#Integer(SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807))))) ==> abort_flag;
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(LibraCoin_MintCapability_type_value())
    var t4: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidInteger(amount);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, amount);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := BorrowGlobal(GetLocal(m, old_size + 2), LibraCoin_MintCapability_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t4 := LibraCoin_mint(GetLocal(m, old_size + 1), t3);
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t4);

    m := UpdateLocal(m, old_size + 4, t4);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_mint_with_default_capability_verify (amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_mint_with_default_capability(amount);
}

procedure {:inline 1} LibraCoin_mint (value: Value, capability: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)) == (Integer(i#Integer(old(SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))) + i#Integer(value)))));
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (value)));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(value) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(value) + i#Integer(SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(value) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(value) + i#Integer(SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807))))) ==> abort_flag;
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidInteger(value);
    assume is#Vector(Dereference(m, capability));
    assume IsValidReferenceParameter(m, local_counter, capability);

    old_size := local_counter;
    local_counter := local_counter + 24;
    m := UpdateLocal(m, old_size + 0, value);

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(capability);

    // unimplemented instruction

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := LdConst(1000000000);
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := LdConst(1000000);
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := Mul(GetLocal(m, old_size + 6), GetLocal(m, old_size + 7));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := Le(GetLocal(m, old_size + 5), GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := Not(GetLocal(m, old_size + 9));
    m := UpdateLocal(m, old_size + 10, tmp);

    tmp := GetLocal(m, old_size + 10);
    if (!b#Boolean(tmp)) { goto Label_11; }

    call tmp := LdConst(11);
    m := UpdateLocal(m, old_size + 11, tmp);

    goto Label_Abort;

Label_11:
    call tmp := LdAddr(173345816);
    m := UpdateLocal(m, old_size + 12, tmp);

    call t13 := BorrowGlobal(GetLocal(m, old_size + 12), LibraCoin_MarketCap_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t13);

    call t14 := CopyOrMoveRef(t2);

    call t15 := BorrowField(t14, LibraCoin_MarketCap_total_value);

    call tmp := ReadRef(t15);
    assume IsValidInteger(tmp);

    m := UpdateLocal(m, old_size + 16, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 16));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 17, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 18, tmp);

    call tmp := Add(GetLocal(m, old_size + 17), GetLocal(m, old_size + 18));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 19, tmp);

    call t20 := CopyOrMoveRef(t2);

    call t21 := BorrowField(t20, LibraCoin_MarketCap_total_value);

    call WriteRef(t21, GetLocal(m, old_size + 19));

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 22, tmp);

    call tmp := Pack_LibraCoin_T(GetLocal(m, old_size + 22));
    m := UpdateLocal(m, old_size + 23, tmp);

    ret0 := GetLocal(m, old_size + 23);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_mint_verify (value: Value, capability: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_mint(value, capability);
}

procedure {:inline 1} LibraCoin_initialize () returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(ExistsResource(m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(txn)))));
ensures !abort_flag ==> b#Boolean(ExistsResource(m, LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(txn)))));
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(txn))))), LibraCoin_MarketCap_total_value)) == (Integer(0))));
ensures old(!(b#Boolean(Boolean((Address(TxnSenderAddress(txn))) != (Address(173345816)))) || b#Boolean(ExistsResource(m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(txn))))) || b#Boolean(ExistsResource(m, LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(txn))))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean((Address(TxnSenderAddress(txn))) != (Address(173345816)))) || b#Boolean(ExistsResource(m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(txn))))) || b#Boolean(ExistsResource(m, LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(txn)))))) ==> abort_flag;
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 9;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := LdAddr(173345816);
    m := UpdateLocal(m, old_size + 1, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 0), GetLocal(m, old_size + 1)));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := Not(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 3, tmp);

    tmp := GetLocal(m, old_size + 3);
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 4, tmp);

    goto Label_Abort;

Label_7:
    call tmp := LdTrue();
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := Pack_LibraCoin_MintCapability(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 6, tmp);

    call MoveToSender(LibraCoin_MintCapability_type_value(), GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := Pack_LibraCoin_MarketCap(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 8, tmp);

    call MoveToSender(LibraCoin_MarketCap_type_value(), GetLocal(m, old_size + 8));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraCoin_initialize_verify () returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraCoin_initialize();
}

procedure {:inline 1} LibraCoin_market_cap () returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((ret0) == (SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))))))) ==> abort_flag;
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 4;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := UpdateLocal(m, old_size + 0, tmp);

    call t1 := BorrowGlobal(GetLocal(m, old_size + 0), LibraCoin_MarketCap_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t2 := BorrowField(t1, LibraCoin_MarketCap_total_value);

    call tmp := ReadRef(t2);
    assume IsValidInteger(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_market_cap_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_market_cap();
}

procedure {:inline 1} LibraCoin_zero () returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (Integer(0))));
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 2;

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := Pack_LibraCoin_T(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    ret0 := GetLocal(m, old_size + 1);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_zero_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_zero();
}

procedure {:inline 1} LibraCoin_value (coin_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures b#Boolean(Boolean((ret0) == (SelectField(Dereference(m, coin_ref), LibraCoin_T_value))));
{
    // declare local variables
    var t1: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, coin_ref));
    assume IsValidReferenceParameter(m, local_counter, coin_ref);

    old_size := local_counter;
    local_counter := local_counter + 4;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(coin_ref);

    call t2 := BorrowField(t1, LibraCoin_T_value);

    call tmp := ReadRef(t2);
    assume IsValidInteger(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_value_verify (coin_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_value(coin_ref);
}

procedure {:inline 1} LibraCoin_split (coin: Value, amount: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean(b#Boolean(Boolean((SelectField(ret1, LibraCoin_T_value)) == (amount))) && b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(coin, LibraCoin_T_value))) - i#Integer(amount)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(SelectField(coin, LibraCoin_T_value)) < i#Integer(amount))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(SelectField(coin, LibraCoin_T_value)) < i#Integer(amount)))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // LibraCoin_T_type_value()
    var t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t4: Value; // IntegerType()
    var t5: Value; // LibraCoin_T_type_value()
    var t6: Value; // LibraCoin_T_type_value()
    var t7: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(coin);
    assume IsValidInteger(amount);

    old_size := local_counter;
    local_counter := local_counter + 8;
    m := UpdateLocal(m, old_size + 0, coin);
    m := UpdateLocal(m, old_size + 1, amount);

    // bytecode translation starts here
    call t3 := BorrowLoc(old_size+0);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 4, tmp);

    call t5 := LibraCoin_withdraw(t3, GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t5);

    m := UpdateLocal(m, old_size + 5, t5);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 7, tmp);

    ret0 := GetLocal(m, old_size + 6);
    ret1 := GetLocal(m, old_size + 7);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
}

procedure LibraCoin_split_verify (coin: Value, amount: Value) returns (ret0: Value, ret1: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0, ret1 := LibraCoin_split(coin, amount);
}

procedure {:inline 1} LibraCoin_withdraw (coin_ref: Reference, amount: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(m, coin_ref), LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(Dereference(m, coin_ref), LibraCoin_T_value))) - i#Integer(amount)))));
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (amount)));
ensures old(!(b#Boolean(Boolean(i#Integer(SelectField(Dereference(m, coin_ref), LibraCoin_T_value)) < i#Integer(amount))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(SelectField(Dereference(m, coin_ref), LibraCoin_T_value)) < i#Integer(amount)))) ==> abort_flag;
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, coin_ref));
    assume IsValidReferenceParameter(m, local_counter, coin_ref);
    assume IsValidInteger(amount);

    old_size := local_counter;
    local_counter := local_counter + 18;
    m := UpdateLocal(m, old_size + 1, amount);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(coin_ref);

    call t4 := BorrowField(t3, LibraCoin_T_value);

    call tmp := ReadRef(t4);
    assume IsValidInteger(tmp);

    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := Ge(GetLocal(m, old_size + 6), GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := Not(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 9, tmp);

    tmp := GetLocal(m, old_size + 9);
    if (!b#Boolean(tmp)) { goto Label_11; }

    call tmp := LdConst(10);
    m := UpdateLocal(m, old_size + 10, tmp);

    goto Label_Abort;

Label_11:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 12, tmp);

    call tmp := Sub(GetLocal(m, old_size + 11), GetLocal(m, old_size + 12));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 13, tmp);

    call t14 := CopyOrMoveRef(coin_ref);

    call t15 := BorrowField(t14, LibraCoin_T_value);

    call WriteRef(t15, GetLocal(m, old_size + 13));

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 16, tmp);

    call tmp := Pack_LibraCoin_T(GetLocal(m, old_size + 16));
    m := UpdateLocal(m, old_size + 17, tmp);

    ret0 := GetLocal(m, old_size + 17);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_withdraw_verify (coin_ref: Reference, amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_withdraw(coin_ref, amount);
}

procedure {:inline 1} LibraCoin_join (coin1: Value, coin2: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(coin1, LibraCoin_T_value))) + i#Integer(old(SelectField(coin2, LibraCoin_T_value)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(coin1, LibraCoin_T_value)) + i#Integer(SelectField(coin2, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(coin1, LibraCoin_T_value)) + i#Integer(SelectField(coin2, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807))))) ==> abort_flag;
{
    // declare local variables
    var t2: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t3: Value; // LibraCoin_T_type_value()
    var t4: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(coin1);
    assume is#Vector(coin2);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, coin1);
    m := UpdateLocal(m, old_size + 1, coin2);

    // bytecode translation starts here
    call t2 := BorrowLoc(old_size+0);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call LibraCoin_deposit(t2, GetLocal(m, old_size + 3));
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_join_verify (coin1: Value, coin2: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_join(coin1, coin2);
}

procedure {:inline 1} LibraCoin_deposit (coin_ref: Reference, check: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(m, coin_ref), LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(Dereference(m, coin_ref), LibraCoin_T_value))) + i#Integer(old(SelectField(check, LibraCoin_T_value)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(m, coin_ref), LibraCoin_T_value)) + i#Integer(SelectField(check, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(m, coin_ref), LibraCoin_T_value)) + i#Integer(SelectField(check, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807))))) ==> abort_flag;
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, coin_ref));
    assume IsValidReferenceParameter(m, local_counter, coin_ref);
    assume is#Vector(check);

    old_size := local_counter;
    local_counter := local_counter + 14;
    m := UpdateLocal(m, old_size + 1, check);

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(coin_ref);

    call t5 := BorrowField(t4, LibraCoin_T_value);

    call tmp := ReadRef(t5);
    assume IsValidInteger(tmp);

    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    call t8 := Unpack_LibraCoin_T(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 8, t8);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 10, tmp);

    call tmp := Add(GetLocal(m, old_size + 9), GetLocal(m, old_size + 10));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 11, tmp);

    call t12 := CopyOrMoveRef(coin_ref);

    call t13 := BorrowField(t12, LibraCoin_T_value);

    call WriteRef(t13, GetLocal(m, old_size + 11));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraCoin_deposit_verify (coin_ref: Reference, check: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraCoin_deposit(coin_ref, check);
}

procedure {:inline 1} LibraCoin_destroy_zero (coin: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean((SelectField(coin, LibraCoin_T_value)) != (Integer(0)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean((SelectField(coin, LibraCoin_T_value)) != (Integer(0))))) ==> abort_flag;
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(coin);

    old_size := local_counter;
    local_counter := local_counter + 9;
    m := UpdateLocal(m, old_size + 0, coin);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := Unpack_LibraCoin_T(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 3, t3);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 5, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 4), GetLocal(m, old_size + 5)));
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := Not(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 7, tmp);

    tmp := GetLocal(m, old_size + 7);
    if (!b#Boolean(tmp)) { goto Label_10; }

    call tmp := LdConst(11);
    m := UpdateLocal(m, old_size + 8, tmp);

    goto Label_Abort;

Label_10:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraCoin_destroy_zero_verify (coin: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraCoin_destroy_zero(coin);
}
