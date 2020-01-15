

// ** structs of module LibraCoin

const unique LibraCoin_T: TypeName;
const LibraCoin_T_value: FieldName;
axiom LibraCoin_T_value == 0;
function LibraCoin_T_type_value(): TypeValue {
    StructType(LibraCoin_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}

procedure {:inline 1} Pack_LibraCoin_T(v0: Value) returns (v: Value)
{
    assume is#Integer(v0);
    v := Vector(ExtendValueArray(EmptyValueArray, v0));

}

procedure {:inline 1} Unpack_LibraCoin_T(v: Value) returns (v0: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, LibraCoin_T_value);
}

const unique LibraCoin_MintCapability: TypeName;
const LibraCoin_MintCapability__dummy: FieldName;
axiom LibraCoin_MintCapability__dummy == 0;
function LibraCoin_MintCapability_type_value(): TypeValue {
    StructType(LibraCoin_MintCapability, ExtendTypeValueArray(EmptyTypeValueArray, BooleanType()))
}

procedure {:inline 1} Pack_LibraCoin_MintCapability(v0: Value) returns (v: Value)
{
    assume is#Boolean(v0);
    v := Vector(ExtendValueArray(EmptyValueArray, v0));

}

procedure {:inline 1} Unpack_LibraCoin_MintCapability(v: Value) returns (v0: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, LibraCoin_MintCapability__dummy);
}

const unique LibraCoin_MarketCap: TypeName;
const LibraCoin_MarketCap_total_value: FieldName;
axiom LibraCoin_MarketCap_total_value == 0;
function LibraCoin_MarketCap_type_value(): TypeValue {
    StructType(LibraCoin_MarketCap, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}

procedure {:inline 1} Pack_LibraCoin_MarketCap(v0: Value) returns (v: Value)
{
    assume is#Integer(v0);
    v := Vector(ExtendValueArray(EmptyValueArray, v0));

}

procedure {:inline 1} Unpack_LibraCoin_MarketCap(v: Value) returns (v0: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, LibraCoin_MarketCap_total_value);
}



// ** structs of module Hash



// ** structs of module U64Util



// ** structs of module AddressUtil



// ** structs of module BytearrayUtil



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
function LibraAccount_T_type_value(): TypeValue {
    StructType(LibraAccount_T, ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, ByteArrayType()), LibraCoin_T_type_value()), BooleanType()), BooleanType()), LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())), LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())), IntegerType()), LibraAccount_EventHandleGenerator_type_value()))
}

procedure {:inline 1} Pack_LibraAccount_T(v0: Value, v1: Value, v2: Value, v3: Value, v4: Value, v5: Value, v6: Value, v7: Value) returns (v: Value)
{
    assume is#ByteArray(v0);
    assume is#Vector(v1);
    assume is#Boolean(v2);
    assume is#Boolean(v3);
    assume is#Vector(v4);
    assume is#Vector(v5);
    assume is#Integer(v6);
    assume is#Vector(v7);
    v := Vector(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(EmptyValueArray, v0), v1), v2), v3), v4), v5), v6), v7));

}

procedure {:inline 1} Unpack_LibraAccount_T(v: Value) returns (v0: Value, v1: Value, v2: Value, v3: Value, v4: Value, v5: Value, v6: Value, v7: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, LibraAccount_T_authentication_key);
    v1 := SelectField(v, LibraAccount_T_balance);
    v2 := SelectField(v, LibraAccount_T_delegated_key_rotation_capability);
    v3 := SelectField(v, LibraAccount_T_delegated_withdrawal_capability);
    v4 := SelectField(v, LibraAccount_T_received_events);
    v5 := SelectField(v, LibraAccount_T_sent_events);
    v6 := SelectField(v, LibraAccount_T_sequence_number);
    v7 := SelectField(v, LibraAccount_T_event_generator);
}

const unique LibraAccount_WithdrawalCapability: TypeName;
const LibraAccount_WithdrawalCapability_account_address: FieldName;
axiom LibraAccount_WithdrawalCapability_account_address == 0;
function LibraAccount_WithdrawalCapability_type_value(): TypeValue {
    StructType(LibraAccount_WithdrawalCapability, ExtendTypeValueArray(EmptyTypeValueArray, AddressType()))
}

procedure {:inline 1} Pack_LibraAccount_WithdrawalCapability(v0: Value) returns (v: Value)
{
    assume is#Address(v0);
    v := Vector(ExtendValueArray(EmptyValueArray, v0));

}

procedure {:inline 1} Unpack_LibraAccount_WithdrawalCapability(v: Value) returns (v0: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, LibraAccount_WithdrawalCapability_account_address);
}

const unique LibraAccount_KeyRotationCapability: TypeName;
const LibraAccount_KeyRotationCapability_account_address: FieldName;
axiom LibraAccount_KeyRotationCapability_account_address == 0;
function LibraAccount_KeyRotationCapability_type_value(): TypeValue {
    StructType(LibraAccount_KeyRotationCapability, ExtendTypeValueArray(EmptyTypeValueArray, AddressType()))
}

procedure {:inline 1} Pack_LibraAccount_KeyRotationCapability(v0: Value) returns (v: Value)
{
    assume is#Address(v0);
    v := Vector(ExtendValueArray(EmptyValueArray, v0));

}

procedure {:inline 1} Unpack_LibraAccount_KeyRotationCapability(v: Value) returns (v0: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, LibraAccount_KeyRotationCapability_account_address);
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

procedure {:inline 1} Pack_LibraAccount_SentPaymentEvent(v0: Value, v1: Value, v2: Value) returns (v: Value)
{
    assume is#Integer(v0);
    assume is#Address(v1);
    assume is#ByteArray(v2);
    v := Vector(ExtendValueArray(ExtendValueArray(ExtendValueArray(EmptyValueArray, v0), v1), v2));

}

procedure {:inline 1} Unpack_LibraAccount_SentPaymentEvent(v: Value) returns (v0: Value, v1: Value, v2: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, LibraAccount_SentPaymentEvent_amount);
    v1 := SelectField(v, LibraAccount_SentPaymentEvent_payee);
    v2 := SelectField(v, LibraAccount_SentPaymentEvent_metadata);
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

procedure {:inline 1} Pack_LibraAccount_ReceivedPaymentEvent(v0: Value, v1: Value, v2: Value) returns (v: Value)
{
    assume is#Integer(v0);
    assume is#Address(v1);
    assume is#ByteArray(v2);
    v := Vector(ExtendValueArray(ExtendValueArray(ExtendValueArray(EmptyValueArray, v0), v1), v2));

}

procedure {:inline 1} Unpack_LibraAccount_ReceivedPaymentEvent(v: Value) returns (v0: Value, v1: Value, v2: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, LibraAccount_ReceivedPaymentEvent_amount);
    v1 := SelectField(v, LibraAccount_ReceivedPaymentEvent_payer);
    v2 := SelectField(v, LibraAccount_ReceivedPaymentEvent_metadata);
}

const unique LibraAccount_EventHandleGenerator: TypeName;
const LibraAccount_EventHandleGenerator_counter: FieldName;
axiom LibraAccount_EventHandleGenerator_counter == 0;
function LibraAccount_EventHandleGenerator_type_value(): TypeValue {
    StructType(LibraAccount_EventHandleGenerator, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}

procedure {:inline 1} Pack_LibraAccount_EventHandleGenerator(v0: Value) returns (v: Value)
{
    assume is#Integer(v0);
    v := Vector(ExtendValueArray(EmptyValueArray, v0));

}

procedure {:inline 1} Unpack_LibraAccount_EventHandleGenerator(v: Value) returns (v0: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, LibraAccount_EventHandleGenerator_counter);
}

const unique LibraAccount_EventHandle: TypeName;
const LibraAccount_EventHandle_counter: FieldName;
axiom LibraAccount_EventHandle_counter == 0;
const LibraAccount_EventHandle_guid: FieldName;
axiom LibraAccount_EventHandle_guid == 1;
function LibraAccount_EventHandle_type_value(tv0: TypeValue): TypeValue {
    StructType(LibraAccount_EventHandle, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), ByteArrayType()))
}

procedure {:inline 1} Pack_LibraAccount_EventHandle(tv0: TypeValue, v0: Value, v1: Value) returns (v: Value)
{
    assume is#Integer(v0);
    assume is#ByteArray(v1);
    v := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, v0), v1));

}

procedure {:inline 1} Unpack_LibraAccount_EventHandle(v: Value) returns (v0: Value, v1: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, LibraAccount_EventHandle_counter);
    v1 := SelectField(v, LibraAccount_EventHandle_guid);
}



// ** functions of module LibraCoin

procedure {:inline 1} LibraCoin_mint_with_default_capability (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)) == (Integer(i#Integer(old(SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))) + i#Integer(arg0)))));
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (arg0)));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(arg0) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(arg0) + i#Integer(SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(arg0) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(arg0) + i#Integer(SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807))))) ==> abort_flag;
{
    // declare local variables
    var t0: Value; // IntegerType()
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
    assume is#Integer(arg0);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, arg0);

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

procedure LibraCoin_mint_with_default_capability_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraCoin_mint_with_default_capability(arg0);
}

procedure {:inline 1} LibraCoin_mint (arg0: Value, arg1: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)) == (Integer(i#Integer(old(SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))) + i#Integer(arg0)))));
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (arg0)));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(arg0) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(arg0) + i#Integer(SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(arg0) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(arg0) + i#Integer(SelectField(Dereference(m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807))))) ==> abort_flag;
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Reference; // ReferenceType(LibraCoin_MintCapability_type_value())
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
    assume is#Integer(arg0);
    assume IsValidReferenceParameter(local_counter, arg1);

    old_size := local_counter;
    local_counter := local_counter + 24;
    m := UpdateLocal(m, old_size + 0, arg0);
    t1 := arg1;

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(t1);

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
    assume is#Integer(tmp);

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

    assume is#Integer(GetLocal(m, old_size + 22));

    call tmp := Pack_LibraCoin_T(GetLocal(m, old_size + 22));
    m := UpdateLocal(m, old_size + 23, tmp);

    ret0 := GetLocal(m, old_size + 23);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_mint_verify (arg0: Value, arg1: Reference) returns (ret0: Value)
{
    call ret0 := LibraCoin_mint(arg0, arg1);
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

    assume is#Boolean(GetLocal(m, old_size + 5));

    call tmp := Pack_LibraCoin_MintCapability(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 6, tmp);

    call MoveToSender(LibraCoin_MintCapability_type_value(), GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 7, tmp);

    assume is#Integer(GetLocal(m, old_size + 7));

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
    assume is#Integer(tmp);

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

    assume is#Integer(GetLocal(m, old_size + 0));

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
    call ret0 := LibraCoin_zero();
}

procedure {:inline 1} LibraCoin_value (arg0: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures b#Boolean(Boolean((ret0) == (SelectField(Dereference(m, arg0), LibraCoin_T_value))));
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t1: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);

    old_size := local_counter;
    local_counter := local_counter + 4;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, LibraCoin_T_value);

    call tmp := ReadRef(t2);
    assume is#Integer(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_value_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := LibraCoin_value(arg0);
}

procedure {:inline 1} LibraCoin_split (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // LibraCoin_T_type_value()
    var t1: Value; // IntegerType()
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
    assume is#Vector(arg0);
    assume is#Integer(arg1);

    old_size := local_counter;
    local_counter := local_counter + 8;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);

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

procedure LibraCoin_split_verify (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
{
    call ret0, ret1 := LibraCoin_split(arg0, arg1);
}

procedure {:inline 1} LibraCoin_withdraw (arg0: Reference, arg1: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(m, arg0), LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(Dereference(m, arg0), LibraCoin_T_value))) - i#Integer(arg1)))));
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (arg1)));
ensures old(!(b#Boolean(Boolean(i#Integer(SelectField(Dereference(m, arg0), LibraCoin_T_value)) < i#Integer(arg1))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(SelectField(Dereference(m, arg0), LibraCoin_T_value)) < i#Integer(arg1)))) ==> abort_flag;
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t1: Value; // IntegerType()
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
    assume IsValidReferenceParameter(local_counter, arg0);
    assume is#Integer(arg1);

    old_size := local_counter;
    local_counter := local_counter + 18;
    t0 := arg0;
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(t0);

    call t4 := BorrowField(t3, LibraCoin_T_value);

    call tmp := ReadRef(t4);
    assume is#Integer(tmp);

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

    call t14 := CopyOrMoveRef(t0);

    call t15 := BorrowField(t14, LibraCoin_T_value);

    call WriteRef(t15, GetLocal(m, old_size + 13));

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 16, tmp);

    assume is#Integer(GetLocal(m, old_size + 16));

    call tmp := Pack_LibraCoin_T(GetLocal(m, old_size + 16));
    m := UpdateLocal(m, old_size + 17, tmp);

    ret0 := GetLocal(m, old_size + 17);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_withdraw_verify (arg0: Reference, arg1: Value) returns (ret0: Value)
{
    call ret0 := LibraCoin_withdraw(arg0, arg1);
}

procedure {:inline 1} LibraCoin_join (arg0: Value, arg1: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(arg0, LibraCoin_T_value))) + i#Integer(old(SelectField(arg1, LibraCoin_T_value)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(arg0, LibraCoin_T_value)) + i#Integer(SelectField(arg1, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(arg0, LibraCoin_T_value)) + i#Integer(SelectField(arg1, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807))))) ==> abort_flag;
{
    // declare local variables
    var t0: Value; // LibraCoin_T_type_value()
    var t1: Value; // LibraCoin_T_type_value()
    var t2: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t3: Value; // LibraCoin_T_type_value()
    var t4: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(arg0);
    assume is#Vector(arg1);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);

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

procedure LibraCoin_join_verify (arg0: Value, arg1: Value) returns (ret0: Value)
{
    call ret0 := LibraCoin_join(arg0, arg1);
}

procedure {:inline 1} LibraCoin_deposit (arg0: Reference, arg1: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(Dereference(m, arg0), LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(Dereference(m, arg0), LibraCoin_T_value))) + i#Integer(old(SelectField(arg1, LibraCoin_T_value)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(m, arg0), LibraCoin_T_value)) + i#Integer(SelectField(arg1, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(m, arg0), LibraCoin_T_value)) + i#Integer(SelectField(arg1, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807))))) ==> abort_flag;
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t1: Value; // LibraCoin_T_type_value()
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
    assume IsValidReferenceParameter(local_counter, arg0);
    assume is#Vector(arg1);

    old_size := local_counter;
    local_counter := local_counter + 14;
    t0 := arg0;
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, LibraCoin_T_value);

    call tmp := ReadRef(t5);
    assume is#Integer(tmp);

    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    call t8 := Unpack_LibraCoin_T(GetLocal(m, old_size + 7));
    assume is#Integer(t8);

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

    call t12 := CopyOrMoveRef(t0);

    call t13 := BorrowField(t12, LibraCoin_T_value);

    call WriteRef(t13, GetLocal(m, old_size + 11));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraCoin_deposit_verify (arg0: Reference, arg1: Value) returns ()
{
    call LibraCoin_deposit(arg0, arg1);
}

procedure {:inline 1} LibraCoin_destroy_zero (arg0: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean((SelectField(arg0, LibraCoin_T_value)) != (Integer(0)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean((SelectField(arg0, LibraCoin_T_value)) != (Integer(0))))) ==> abort_flag;
{
    // declare local variables
    var t0: Value; // LibraCoin_T_type_value()
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
    assume is#Vector(arg0);

    old_size := local_counter;
    local_counter := local_counter + 9;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := Unpack_LibraCoin_T(GetLocal(m, old_size + 2));
    assume is#Integer(t3);

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

procedure LibraCoin_destroy_zero_verify (arg0: Value) returns ()
{
    call LibraCoin_destroy_zero(arg0);
}



// ** functions of module Hash

procedure {:inline 1} Hash_sha2_256 (arg0: Value) returns (ret0: Value);procedure {:inline 1} Hash_sha3_256 (arg0: Value) returns (ret0: Value);

// ** functions of module U64Util

procedure {:inline 1} U64Util_u64_to_bytes (arg0: Value) returns (ret0: Value);

// ** functions of module AddressUtil

procedure {:inline 1} AddressUtil_address_to_bytes (arg0: Value) returns (ret0: Value);

// ** functions of module BytearrayUtil

procedure {:inline 1} BytearrayUtil_bytearray_concat (arg0: Value, arg1: Value) returns (ret0: Value);

// ** functions of module LibraAccount

procedure {:inline 1} LibraAccount_make (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // LibraCoin_T_type_value()
    var t2: Value; // LibraAccount_EventHandleGenerator_type_value()
    var t3: Value; // LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())
    var t4: Value; // LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())
    var t5: Value; // ByteArrayType()
    var t6: Value; // AddressType()
    var t7: Value; // ByteArrayType()
    var t8: Value; // IntegerType()
    var t9: Value; // LibraAccount_EventHandleGenerator_type_value()
    var t10: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t11: Value; // AddressType()
    var t12: Value; // LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())
    var t13: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t14: Value; // AddressType()
    var t15: Value; // LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())
    var t16: Value; // LibraCoin_T_type_value()
    var t17: Value; // ByteArrayType()
    var t18: Value; // LibraCoin_T_type_value()
    var t19: Value; // BooleanType()
    var t20: Value; // BooleanType()
    var t21: Value; // LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())
    var t22: Value; // LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())
    var t23: Value; // IntegerType()
    var t24: Value; // LibraAccount_EventHandleGenerator_type_value()
    var t25: Value; // LibraAccount_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := local_counter;
    local_counter := local_counter + 26;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 6, tmp);

    call t7 := AddressUtil_address_to_bytes(GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t7);

    m := UpdateLocal(m, old_size + 7, t7);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 8, tmp);

    assume is#Integer(GetLocal(m, old_size + 8));

    call tmp := Pack_LibraAccount_EventHandleGenerator(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 9));
    m := UpdateLocal(m, old_size + 2, tmp);

    call t10 := BorrowLoc(old_size+2);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 11, tmp);

    call t12 := LibraAccount_new_event_handle_impl(LibraAccount_SentPaymentEvent_type_value(), t10, GetLocal(m, old_size + 11));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t12);

    m := UpdateLocal(m, old_size + 12, t12);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 12));
    m := UpdateLocal(m, old_size + 3, tmp);

    call t13 := BorrowLoc(old_size+2);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 14, tmp);

    call t15 := LibraAccount_new_event_handle_impl(LibraAccount_ReceivedPaymentEvent_type_value(), t13, GetLocal(m, old_size + 14));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t15);

    m := UpdateLocal(m, old_size + 15, t15);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 15));
    m := UpdateLocal(m, old_size + 4, tmp);

    call t16 := LibraCoin_zero();
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t16);

    m := UpdateLocal(m, old_size + 16, t16);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 16));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 17, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 18, tmp);

    call tmp := LdFalse();
    m := UpdateLocal(m, old_size + 19, tmp);

    call tmp := LdFalse();
    m := UpdateLocal(m, old_size + 20, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 21, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 22, tmp);

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 23, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 24, tmp);

    assume is#ByteArray(GetLocal(m, old_size + 17));

    assume is#Vector(GetLocal(m, old_size + 18));

    assume is#Boolean(GetLocal(m, old_size + 19));

    assume is#Boolean(GetLocal(m, old_size + 20));

    assume is#Vector(GetLocal(m, old_size + 21));

    assume is#Vector(GetLocal(m, old_size + 22));

    assume is#Integer(GetLocal(m, old_size + 23));

    assume is#Vector(GetLocal(m, old_size + 24));

    call tmp := Pack_LibraAccount_T(GetLocal(m, old_size + 17), GetLocal(m, old_size + 18), GetLocal(m, old_size + 19), GetLocal(m, old_size + 20), GetLocal(m, old_size + 21), GetLocal(m, old_size + 22), GetLocal(m, old_size + 23), GetLocal(m, old_size + 24));
    m := UpdateLocal(m, old_size + 25, tmp);

    ret0 := GetLocal(m, old_size + 25);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_make_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_make(arg0);
}

procedure {:inline 1} LibraAccount_deposit (arg0: Value, arg1: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // LibraCoin_T_type_value()
    var t2: Value; // AddressType()
    var t3: Value; // LibraCoin_T_type_value()
    var t4: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume is#Vector(arg1);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    // unimplemented instruction

    call LibraAccount_deposit_with_metadata(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_deposit_verify (arg0: Value, arg1: Value) returns ()
{
    call LibraAccount_deposit(arg0, arg1);
}

procedure {:inline 1} LibraAccount_deposit_with_metadata (arg0: Value, arg1: Value, arg2: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // LibraCoin_T_type_value()
    var t2: Value; // ByteArrayType()
    var t3: Value; // AddressType()
    var t4: Value; // AddressType()
    var t5: Value; // LibraCoin_T_type_value()
    var t6: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume is#Vector(arg1);
    assume is#ByteArray(arg2);

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);
    m := UpdateLocal(m, old_size + 2, arg2);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 6, tmp);

    call LibraAccount_deposit_with_sender_and_metadata(GetLocal(m, old_size + 3), GetLocal(m, old_size + 4), GetLocal(m, old_size + 5), GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_deposit_with_metadata_verify (arg0: Value, arg1: Value, arg2: Value) returns ()
{
    call LibraAccount_deposit_with_metadata(arg0, arg1, arg2);
}

procedure {:inline 1} LibraAccount_deposit_with_sender_and_metadata (arg0: Value, arg1: Value, arg2: Value, arg3: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Value; // LibraCoin_T_type_value()
    var t3: Value; // ByteArrayType()
    var t4: Value; // IntegerType()
    var t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // IntegerType()
    var t11: Value; // BooleanType()
    var t12: Value; // BooleanType()
    var t13: Value; // IntegerType()
    var t14: Value; // AddressType()
    var t15: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t16: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t17: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value()))
    var t18: Value; // IntegerType()
    var t19: Value; // AddressType()
    var t20: Value; // ByteArrayType()
    var t21: Value; // LibraAccount_SentPaymentEvent_type_value()
    var t22: Value; // AddressType()
    var t23: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t24: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t25: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t26: Value; // LibraCoin_T_type_value()
    var t27: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t28: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value()))
    var t29: Value; // IntegerType()
    var t30: Value; // AddressType()
    var t31: Value; // ByteArrayType()
    var t32: Value; // LibraAccount_ReceivedPaymentEvent_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume is#Address(arg1);
    assume is#Vector(arg2);
    assume is#ByteArray(arg3);

    old_size := local_counter;
    local_counter := local_counter + 33;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);
    m := UpdateLocal(m, old_size + 2, arg2);
    m := UpdateLocal(m, old_size + 3, arg3);

    // bytecode translation starts here
    call t7 := BorrowLoc(old_size+2);

    call t8 := LibraCoin_value(t7);
    if (abort_flag) { goto Label_Abort; }
    assume is#Integer(t8);

    m := UpdateLocal(m, old_size + 8, t8);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 10, tmp);

    call tmp := Gt(GetLocal(m, old_size + 9), GetLocal(m, old_size + 10));
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := Not(GetLocal(m, old_size + 11));
    m := UpdateLocal(m, old_size + 12, tmp);

    tmp := GetLocal(m, old_size + 12);
    if (!b#Boolean(tmp)) { goto Label_10; }

    call tmp := LdConst(7);
    m := UpdateLocal(m, old_size + 13, tmp);

    goto Label_Abort;

Label_10:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 14, tmp);

    call t15 := BorrowGlobal(GetLocal(m, old_size + 14), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t6 := CopyOrMoveRef(t15);

    call t16 := CopyOrMoveRef(t6);

    call t17 := BorrowField(t16, LibraAccount_T_sent_events);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 18, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 19, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 20, tmp);

    assume is#Integer(GetLocal(m, old_size + 18));

    assume is#Address(GetLocal(m, old_size + 19));

    assume is#ByteArray(GetLocal(m, old_size + 20));

    call tmp := Pack_LibraAccount_SentPaymentEvent(GetLocal(m, old_size + 18), GetLocal(m, old_size + 19), GetLocal(m, old_size + 20));
    m := UpdateLocal(m, old_size + 21, tmp);

    call LibraAccount_emit_event(LibraAccount_SentPaymentEvent_type_value(), t17, GetLocal(m, old_size + 21));
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 22, tmp);

    call t23 := BorrowGlobal(GetLocal(m, old_size + 22), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t5 := CopyOrMoveRef(t23);

    call t24 := CopyOrMoveRef(t5);

    call t25 := BorrowField(t24, LibraAccount_T_balance);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 26, tmp);

    call LibraCoin_deposit(t25, GetLocal(m, old_size + 26));
    if (abort_flag) { goto Label_Abort; }

    call t27 := CopyOrMoveRef(t5);

    call t28 := BorrowField(t27, LibraAccount_T_received_events);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 29, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 30, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 31, tmp);

    assume is#Integer(GetLocal(m, old_size + 29));

    assume is#Address(GetLocal(m, old_size + 30));

    assume is#ByteArray(GetLocal(m, old_size + 31));

    call tmp := Pack_LibraAccount_ReceivedPaymentEvent(GetLocal(m, old_size + 29), GetLocal(m, old_size + 30), GetLocal(m, old_size + 31));
    m := UpdateLocal(m, old_size + 32, tmp);

    call LibraAccount_emit_event(LibraAccount_ReceivedPaymentEvent_type_value(), t28, GetLocal(m, old_size + 32));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_deposit_with_sender_and_metadata_verify (arg0: Value, arg1: Value, arg2: Value, arg3: Value) returns ()
{
    call LibraAccount_deposit_with_sender_and_metadata(arg0, arg1, arg2, arg3);
}

procedure {:inline 1} LibraAccount_mint_to_address (arg0: Value, arg1: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // IntegerType()
    var t2: Value; // AddressType()
    var t3: Value; // BooleanType()
    var t4: Value; // BooleanType()
    var t5: Value; // AddressType()
    var t6: Value; // AddressType()
    var t7: Value; // IntegerType()
    var t8: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume is#Integer(arg1);

    old_size := local_counter;
    local_counter := local_counter + 9;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := Exists(GetLocal(m, old_size + 2), LibraAccount_T_type_value());
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := Not(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, tmp);

    tmp := GetLocal(m, old_size + 4);
    if (!b#Boolean(tmp)) { goto Label_6; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 5, tmp);

    call LibraAccount_create_account(GetLocal(m, old_size + 5));
    if (abort_flag) { goto Label_Abort; }

Label_6:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    call t8 := LibraCoin_mint_with_default_capability(GetLocal(m, old_size + 7));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t8);

    m := UpdateLocal(m, old_size + 8, t8);

    call LibraAccount_deposit(GetLocal(m, old_size + 6), GetLocal(m, old_size + 8));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_mint_to_address_verify (arg0: Value, arg1: Value) returns ()
{
    call LibraAccount_mint_to_address(arg0, arg1);
}

procedure {:inline 1} LibraAccount_withdraw_from_account (arg0: Reference, arg1: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (arg1)));
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(SelectField(Dereference(m, arg0), LibraAccount_T_balance), LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(SelectField(Dereference(m, arg0), LibraAccount_T_balance), LibraCoin_T_value))) - i#Integer(arg1)))));
ensures old(!(b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(m, arg0), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(arg1))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(m, arg0), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(arg1)))) ==> abort_flag;
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t1: Value; // IntegerType()
    var t2: Value; // LibraCoin_T_type_value()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t5: Value; // IntegerType()
    var t6: Value; // LibraCoin_T_type_value()
    var t7: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);
    assume is#Integer(arg1);

    old_size := local_counter;
    local_counter := local_counter + 8;
    t0 := arg0;
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(t0);

    call t4 := BorrowField(t3, LibraAccount_T_balance);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    call t6 := LibraCoin_withdraw(t4, GetLocal(m, old_size + 5));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t6);

    m := UpdateLocal(m, old_size + 6, t6);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 7, tmp);

    ret0 := GetLocal(m, old_size + 7);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_from_account_verify (arg0: Reference, arg1: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_withdraw_from_account(arg0, arg1);
}

procedure {:inline 1} LibraAccount_withdraw_from_sender (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (arg0)));
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(SelectField(Dereference(m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(txn))))), LibraAccount_T_balance), LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(SelectField(Dereference(m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(txn))))), LibraAccount_T_balance), LibraCoin_T_value))) - i#Integer(arg0)))));
ensures old(!(b#Boolean(SelectField(Dereference(m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(txn))))), LibraAccount_T_delegated_withdrawal_capability)) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(txn))))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(arg0))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(txn)))))))))) ==> !abort_flag;
ensures old(b#Boolean(SelectField(Dereference(m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(txn))))), LibraAccount_T_delegated_withdrawal_capability)) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(m, GetResourceReference(LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(txn))))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(arg0))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraAccount_T_type_value(), a#Address(Address(TxnSenderAddress(txn))))))))) ==> abort_flag;
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t5: Reference; // ReferenceType(BooleanType())
    var t6: Value; // BooleanType()
    var t7: Value; // IntegerType()
    var t8: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t9: Value; // IntegerType()
    var t10: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Integer(arg0);

    old_size := local_counter;
    local_counter := local_counter + 11;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := BorrowGlobal(GetLocal(m, old_size + 2), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, LibraAccount_T_delegated_withdrawal_capability);

    call tmp := ReadRef(t5);
    assume is#Boolean(tmp);

    m := UpdateLocal(m, old_size + 6, tmp);

    tmp := GetLocal(m, old_size + 6);
    if (!b#Boolean(tmp)) { goto Label_9; }

    call tmp := LdConst(11);
    m := UpdateLocal(m, old_size + 7, tmp);

    goto Label_Abort;

Label_9:
    call t8 := CopyOrMoveRef(t1);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 9, tmp);

    call t10 := LibraAccount_withdraw_from_account(t8, GetLocal(m, old_size + 9));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t10);

    m := UpdateLocal(m, old_size + 10, t10);

    ret0 := GetLocal(m, old_size + 10);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_from_sender_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_withdraw_from_sender(arg0);
}

procedure {:inline 1} LibraAccount_withdraw_with_capability (arg0: Reference, arg1: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(ret0, LibraCoin_T_value)) == (arg1)));
ensures !abort_flag ==> b#Boolean(Boolean((SelectField(SelectField(Dereference(m, GetResourceReference(LibraAccount_T_type_value(), a#Address(SelectField(Dereference(m, arg0), LibraAccount_WithdrawalCapability_account_address)))), LibraAccount_T_balance), LibraCoin_T_value)) == (Integer(i#Integer(old(SelectField(SelectField(Dereference(m, GetResourceReference(LibraAccount_T_type_value(), a#Address(SelectField(Dereference(m, arg0), LibraAccount_WithdrawalCapability_account_address)))), LibraAccount_T_balance), LibraCoin_T_value))) - i#Integer(arg1)))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraAccount_T_type_value(), a#Address(SelectField(Dereference(m, arg0), LibraAccount_WithdrawalCapability_account_address))))))) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(m, GetResourceReference(LibraAccount_T_type_value(), a#Address(SelectField(Dereference(m, arg0), LibraAccount_WithdrawalCapability_account_address)))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(arg1))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(m, LibraAccount_T_type_value(), a#Address(SelectField(Dereference(m, arg0), LibraAccount_WithdrawalCapability_account_address))))))) || b#Boolean(Boolean(i#Integer(SelectField(SelectField(Dereference(m, GetResourceReference(LibraAccount_T_type_value(), a#Address(SelectField(Dereference(m, arg0), LibraAccount_WithdrawalCapability_account_address)))), LibraAccount_T_balance), LibraCoin_T_value)) < i#Integer(arg1)))) ==> abort_flag;
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t1: Value; // IntegerType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t4: Reference; // ReferenceType(AddressType())
    var t5: Value; // AddressType()
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t8: Value; // IntegerType()
    var t9: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);
    assume is#Integer(arg1);

    old_size := local_counter;
    local_counter := local_counter + 10;
    t0 := arg0;
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(t0);

    call t4 := BorrowField(t3, LibraAccount_WithdrawalCapability_account_address);

    call tmp := ReadRef(t4);
    assume is#Address(tmp);

    m := UpdateLocal(m, old_size + 5, tmp);

    call t6 := BorrowGlobal(GetLocal(m, old_size + 5), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t6);

    call t7 := CopyOrMoveRef(t2);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 8, tmp);

    call t9 := LibraAccount_withdraw_from_account(t7, GetLocal(m, old_size + 8));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t9);

    m := UpdateLocal(m, old_size + 9, t9);

    ret0 := GetLocal(m, old_size + 9);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_with_capability_verify (arg0: Reference, arg1: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_withdraw_with_capability(arg0, arg1);
}

procedure {:inline 1} LibraAccount_extract_sender_withdrawal_capability () returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t2: Reference; // ReferenceType(BooleanType())
    var t3: Value; // AddressType()
    var t4: Value; // AddressType()
    var t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Reference; // ReferenceType(BooleanType())
    var t8: Reference; // ReferenceType(BooleanType())
    var t9: Value; // BooleanType()
    var t10: Value; // IntegerType()
    var t11: Value; // BooleanType()
    var t12: Reference; // ReferenceType(BooleanType())
    var t13: Value; // AddressType()
    var t14: Value; // LibraAccount_WithdrawalCapability_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 15;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    call t5 := BorrowGlobal(GetLocal(m, old_size + 4), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, LibraAccount_T_delegated_withdrawal_capability);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t8);
    assume is#Boolean(tmp);

    m := UpdateLocal(m, old_size + 9, tmp);

    tmp := GetLocal(m, old_size + 9);
    if (!b#Boolean(tmp)) { goto Label_13; }

    call tmp := LdConst(11);
    m := UpdateLocal(m, old_size + 10, tmp);

    goto Label_Abort;

Label_13:
    call tmp := LdTrue();
    m := UpdateLocal(m, old_size + 11, tmp);

    call t12 := CopyOrMoveRef(t2);

    call WriteRef(t12, GetLocal(m, old_size + 11));

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 13, tmp);

    assume is#Address(GetLocal(m, old_size + 13));

    call tmp := Pack_LibraAccount_WithdrawalCapability(GetLocal(m, old_size + 13));
    m := UpdateLocal(m, old_size + 14, tmp);

    ret0 := GetLocal(m, old_size + 14);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_extract_sender_withdrawal_capability_verify () returns (ret0: Value)
{
    call ret0 := LibraAccount_extract_sender_withdrawal_capability();
}

procedure {:inline 1} LibraAccount_restore_withdrawal_capability (arg0: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // LibraAccount_WithdrawalCapability_type_value()
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Value; // LibraAccount_WithdrawalCapability_type_value()
    var t4: Value; // AddressType()
    var t5: Value; // AddressType()
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Value; // BooleanType()
    var t8: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t9: Reference; // ReferenceType(BooleanType())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(arg0);

    old_size := local_counter;
    local_counter := local_counter + 10;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call t4 := Unpack_LibraAccount_WithdrawalCapability(GetLocal(m, old_size + 3));
    assume is#Address(t4);

    m := UpdateLocal(m, old_size + 4, t4);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    call t6 := BorrowGlobal(GetLocal(m, old_size + 5), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t6);

    call tmp := LdFalse();
    m := UpdateLocal(m, old_size + 7, tmp);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, LibraAccount_T_delegated_withdrawal_capability);

    call WriteRef(t9, GetLocal(m, old_size + 7));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_restore_withdrawal_capability_verify (arg0: Value) returns ()
{
    call LibraAccount_restore_withdrawal_capability(arg0);
}

procedure {:inline 1} LibraAccount_pay_from_capability (arg0: Value, arg1: Reference, arg2: Value, arg3: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t2: Value; // IntegerType()
    var t3: Value; // ByteArrayType()
    var t4: Value; // AddressType()
    var t5: Value; // BooleanType()
    var t6: Value; // BooleanType()
    var t7: Value; // AddressType()
    var t8: Value; // AddressType()
    var t9: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t10: Reference; // ReferenceType(AddressType())
    var t11: Value; // AddressType()
    var t12: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t13: Value; // IntegerType()
    var t14: Value; // LibraCoin_T_type_value()
    var t15: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume IsValidReferenceParameter(local_counter, arg1);
    assume is#Integer(arg2);
    assume is#ByteArray(arg3);

    old_size := local_counter;
    local_counter := local_counter + 16;
    m := UpdateLocal(m, old_size + 0, arg0);
    t1 := arg1;
    m := UpdateLocal(m, old_size + 2, arg2);
    m := UpdateLocal(m, old_size + 3, arg3);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Exists(GetLocal(m, old_size + 4), LibraAccount_T_type_value());
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := Not(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 6, tmp);

    tmp := GetLocal(m, old_size + 6);
    if (!b#Boolean(tmp)) { goto Label_6; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 7, tmp);

    call LibraAccount_create_account(GetLocal(m, old_size + 7));
    if (abort_flag) { goto Label_Abort; }

Label_6:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 8, tmp);

    call t9 := CopyOrMoveRef(t1);

    call t10 := BorrowField(t9, LibraAccount_WithdrawalCapability_account_address);

    call tmp := ReadRef(t10);
    assume is#Address(tmp);

    m := UpdateLocal(m, old_size + 11, tmp);

    call t12 := CopyOrMoveRef(t1);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 13, tmp);

    call t14 := LibraAccount_withdraw_with_capability(t12, GetLocal(m, old_size + 13));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t14);

    m := UpdateLocal(m, old_size + 14, t14);

    // unimplemented instruction

    call LibraAccount_deposit_with_sender_and_metadata(GetLocal(m, old_size + 8), GetLocal(m, old_size + 11), GetLocal(m, old_size + 14), GetLocal(m, old_size + 15));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_pay_from_capability_verify (arg0: Value, arg1: Reference, arg2: Value, arg3: Value) returns ()
{
    call LibraAccount_pay_from_capability(arg0, arg1, arg2, arg3);
}

procedure {:inline 1} LibraAccount_pay_from_sender_with_metadata (arg0: Value, arg1: Value, arg2: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // IntegerType()
    var t2: Value; // ByteArrayType()
    var t3: Value; // AddressType()
    var t4: Value; // BooleanType()
    var t5: Value; // BooleanType()
    var t6: Value; // AddressType()
    var t7: Value; // AddressType()
    var t8: Value; // IntegerType()
    var t9: Value; // LibraCoin_T_type_value()
    var t10: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume is#Integer(arg1);
    assume is#ByteArray(arg2);

    old_size := local_counter;
    local_counter := local_counter + 11;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);
    m := UpdateLocal(m, old_size + 2, arg2);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := Exists(GetLocal(m, old_size + 3), LibraAccount_T_type_value());
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Not(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);

    tmp := GetLocal(m, old_size + 5);
    if (!b#Boolean(tmp)) { goto Label_6; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 6, tmp);

    call LibraAccount_create_account(GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }

Label_6:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 8, tmp);

    call t9 := LibraAccount_withdraw_from_sender(GetLocal(m, old_size + 8));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t9);

    m := UpdateLocal(m, old_size + 9, t9);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 10, tmp);

    call LibraAccount_deposit_with_metadata(GetLocal(m, old_size + 7), GetLocal(m, old_size + 9), GetLocal(m, old_size + 10));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_pay_from_sender_with_metadata_verify (arg0: Value, arg1: Value, arg2: Value) returns ()
{
    call LibraAccount_pay_from_sender_with_metadata(arg0, arg1, arg2);
}

procedure {:inline 1} LibraAccount_pay_from_sender (arg0: Value, arg1: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // IntegerType()
    var t2: Value; // AddressType()
    var t3: Value; // IntegerType()
    var t4: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume is#Integer(arg1);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    // unimplemented instruction

    call LibraAccount_pay_from_sender_with_metadata(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_pay_from_sender_verify (arg0: Value, arg1: Value) returns ()
{
    call LibraAccount_pay_from_sender(arg0, arg1);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key_for_account (arg0: Reference, arg1: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t1: Value; // ByteArrayType()
    var t2: Value; // ByteArrayType()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(ByteArrayType())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);
    assume is#ByteArray(arg1);

    old_size := local_counter;
    local_counter := local_counter + 5;
    t0 := arg0;
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := CopyOrMoveRef(t0);

    call t4 := BorrowField(t3, LibraAccount_T_authentication_key);

    call WriteRef(t4, GetLocal(m, old_size + 2));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_rotate_authentication_key_for_account_verify (arg0: Reference, arg1: Value) returns ()
{
    call LibraAccount_rotate_authentication_key_for_account(arg0, arg1);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key (arg0: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // ByteArrayType()
    var t1: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t5: Reference; // ReferenceType(BooleanType())
    var t6: Value; // BooleanType()
    var t7: Value; // IntegerType()
    var t8: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t9: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#ByteArray(arg0);

    old_size := local_counter;
    local_counter := local_counter + 10;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := BorrowGlobal(GetLocal(m, old_size + 2), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, LibraAccount_T_delegated_key_rotation_capability);

    call tmp := ReadRef(t5);
    assume is#Boolean(tmp);

    m := UpdateLocal(m, old_size + 6, tmp);

    tmp := GetLocal(m, old_size + 6);
    if (!b#Boolean(tmp)) { goto Label_9; }

    call tmp := LdConst(11);
    m := UpdateLocal(m, old_size + 7, tmp);

    goto Label_Abort;

Label_9:
    call t8 := CopyOrMoveRef(t1);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 9, tmp);

    call LibraAccount_rotate_authentication_key_for_account(t8, GetLocal(m, old_size + 9));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_rotate_authentication_key_verify (arg0: Value) returns ()
{
    call LibraAccount_rotate_authentication_key(arg0);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key_with_capability (arg0: Reference, arg1: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var t1: Value; // ByteArrayType()
    var t2: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var t3: Reference; // ReferenceType(AddressType())
    var t4: Value; // AddressType()
    var t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t6: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);
    assume is#ByteArray(arg1);

    old_size := local_counter;
    local_counter := local_counter + 7;
    t0 := arg0;
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call t2 := CopyOrMoveRef(t0);

    call t3 := BorrowField(t2, LibraAccount_KeyRotationCapability_account_address);

    call tmp := ReadRef(t3);
    assume is#Address(tmp);

    m := UpdateLocal(m, old_size + 4, tmp);

    call t5 := BorrowGlobal(GetLocal(m, old_size + 4), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 6, tmp);

    call LibraAccount_rotate_authentication_key_for_account(t5, GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_rotate_authentication_key_with_capability_verify (arg0: Reference, arg1: Value) returns ()
{
    call LibraAccount_rotate_authentication_key_with_capability(arg0, arg1);
}

procedure {:inline 1} LibraAccount_extract_sender_key_rotation_capability () returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(BooleanType())
    var t2: Value; // AddressType()
    var t3: Value; // AddressType()
    var t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t5: Reference; // ReferenceType(BooleanType())
    var t6: Reference; // ReferenceType(BooleanType())
    var t7: Value; // BooleanType()
    var t8: Value; // IntegerType()
    var t9: Value; // BooleanType()
    var t10: Reference; // ReferenceType(BooleanType())
    var t11: Value; // AddressType()
    var t12: Value; // LibraAccount_KeyRotationCapability_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 13;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call t4 := BorrowGlobal(GetLocal(m, old_size + 3), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t5 := BorrowField(t4, LibraAccount_T_delegated_key_rotation_capability);

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call tmp := ReadRef(t6);
    assume is#Boolean(tmp);

    m := UpdateLocal(m, old_size + 7, tmp);

    tmp := GetLocal(m, old_size + 7);
    if (!b#Boolean(tmp)) { goto Label_11; }

    call tmp := LdConst(11);
    m := UpdateLocal(m, old_size + 8, tmp);

    goto Label_Abort;

Label_11:
    call tmp := LdTrue();
    m := UpdateLocal(m, old_size + 9, tmp);

    call t10 := CopyOrMoveRef(t1);

    call WriteRef(t10, GetLocal(m, old_size + 9));

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 11, tmp);

    assume is#Address(GetLocal(m, old_size + 11));

    call tmp := Pack_LibraAccount_KeyRotationCapability(GetLocal(m, old_size + 11));
    m := UpdateLocal(m, old_size + 12, tmp);

    ret0 := GetLocal(m, old_size + 12);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_extract_sender_key_rotation_capability_verify () returns (ret0: Value)
{
    call ret0 := LibraAccount_extract_sender_key_rotation_capability();
}

procedure {:inline 1} LibraAccount_restore_key_rotation_capability (arg0: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // LibraAccount_KeyRotationCapability_type_value()
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Value; // LibraAccount_KeyRotationCapability_type_value()
    var t4: Value; // AddressType()
    var t5: Value; // AddressType()
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Value; // BooleanType()
    var t8: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t9: Reference; // ReferenceType(BooleanType())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(arg0);

    old_size := local_counter;
    local_counter := local_counter + 10;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call t4 := Unpack_LibraAccount_KeyRotationCapability(GetLocal(m, old_size + 3));
    assume is#Address(t4);

    m := UpdateLocal(m, old_size + 4, t4);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    call t6 := BorrowGlobal(GetLocal(m, old_size + 5), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t6);

    call tmp := LdFalse();
    m := UpdateLocal(m, old_size + 7, tmp);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, LibraAccount_T_delegated_key_rotation_capability);

    call WriteRef(t9, GetLocal(m, old_size + 7));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_restore_key_rotation_capability_verify (arg0: Value) returns ()
{
    call LibraAccount_restore_key_rotation_capability(arg0);
}

procedure {:inline 1} LibraAccount_create_account (arg0: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // LibraAccount_EventHandleGenerator_type_value()
    var t2: Value; // IntegerType()
    var t3: Value; // LibraAccount_EventHandleGenerator_type_value()
    var t4: Value; // AddressType()
    var t5: Value; // AddressType()
    var t6: Value; // ByteArrayType()
    var t7: Value; // LibraCoin_T_type_value()
    var t8: Value; // BooleanType()
    var t9: Value; // BooleanType()
    var t10: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t11: Value; // AddressType()
    var t12: Value; // LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())
    var t13: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t14: Value; // AddressType()
    var t15: Value; // LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())
    var t16: Value; // IntegerType()
    var t17: Value; // LibraAccount_EventHandleGenerator_type_value()
    var t18: Value; // LibraAccount_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := local_counter;
    local_counter := local_counter + 19;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 2, tmp);

    assume is#Integer(GetLocal(m, old_size + 2));

    call tmp := Pack_LibraAccount_EventHandleGenerator(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 5, tmp);

    call t6 := AddressUtil_address_to_bytes(GetLocal(m, old_size + 5));
    if (abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t6);

    m := UpdateLocal(m, old_size + 6, t6);

    call t7 := LibraCoin_zero();
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t7);

    m := UpdateLocal(m, old_size + 7, t7);

    call tmp := LdFalse();
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := LdFalse();
    m := UpdateLocal(m, old_size + 9, tmp);

    call t10 := BorrowLoc(old_size+1);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 11, tmp);

    call t12 := LibraAccount_new_event_handle_impl(LibraAccount_ReceivedPaymentEvent_type_value(), t10, GetLocal(m, old_size + 11));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t12);

    m := UpdateLocal(m, old_size + 12, t12);

    call t13 := BorrowLoc(old_size+1);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 14, tmp);

    call t15 := LibraAccount_new_event_handle_impl(LibraAccount_SentPaymentEvent_type_value(), t13, GetLocal(m, old_size + 14));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t15);

    m := UpdateLocal(m, old_size + 15, t15);

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 16, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 17, tmp);

    assume is#ByteArray(GetLocal(m, old_size + 6));

    assume is#Vector(GetLocal(m, old_size + 7));

    assume is#Boolean(GetLocal(m, old_size + 8));

    assume is#Boolean(GetLocal(m, old_size + 9));

    assume is#Vector(GetLocal(m, old_size + 12));

    assume is#Vector(GetLocal(m, old_size + 15));

    assume is#Integer(GetLocal(m, old_size + 16));

    assume is#Vector(GetLocal(m, old_size + 17));

    call tmp := Pack_LibraAccount_T(GetLocal(m, old_size + 6), GetLocal(m, old_size + 7), GetLocal(m, old_size + 8), GetLocal(m, old_size + 9), GetLocal(m, old_size + 12), GetLocal(m, old_size + 15), GetLocal(m, old_size + 16), GetLocal(m, old_size + 17));
    m := UpdateLocal(m, old_size + 18, tmp);

    call LibraAccount_save_account(GetLocal(m, old_size + 4), GetLocal(m, old_size + 18));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_create_account_verify (arg0: Value) returns ()
{
    call LibraAccount_create_account(arg0);
}

procedure {:inline 1} LibraAccount_create_new_account (arg0: Value, arg1: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // IntegerType()
    var t2: Value; // AddressType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // BooleanType()
    var t6: Value; // AddressType()
    var t7: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume is#Integer(arg1);

    old_size := local_counter;
    local_counter := local_counter + 8;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call LibraAccount_create_account(GetLocal(m, old_size + 2));
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Gt(GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);

    tmp := GetLocal(m, old_size + 5);
    if (!b#Boolean(tmp)) { goto Label_9; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    call LibraAccount_pay_from_sender(GetLocal(m, old_size + 6), GetLocal(m, old_size + 7));
    if (abort_flag) { goto Label_Abort; }

Label_9:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_create_new_account_verify (arg0: Value, arg1: Value) returns ()
{
    call LibraAccount_create_new_account(arg0, arg1);
}

procedure {:inline 1} LibraAccount_save_account (arg0: Value, arg1: Value) returns ();procedure {:inline 1} LibraAccount_balance_for_account (arg0: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t1: Value; // IntegerType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);

    old_size := local_counter;
    local_counter := local_counter + 6;
    t0 := arg0;

    // bytecode translation starts here
    call t2 := CopyOrMoveRef(t0);

    call t3 := BorrowField(t2, LibraAccount_T_balance);

    call t4 := LibraCoin_value(t3);
    if (abort_flag) { goto Label_Abort; }
    assume is#Integer(t4);

    m := UpdateLocal(m, old_size + 4, t4);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    ret0 := GetLocal(m, old_size + 5);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_balance_for_account_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := LibraAccount_balance_for_account(arg0);
}

procedure {:inline 1} LibraAccount_balance (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := local_counter;
    local_counter := local_counter + 4;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call t2 := BorrowGlobal(GetLocal(m, old_size + 1), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t3 := LibraAccount_balance_for_account(t2);
    if (abort_flag) { goto Label_Abort; }
    assume is#Integer(t3);

    m := UpdateLocal(m, old_size + 3, t3);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_balance_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_balance(arg0);
}

procedure {:inline 1} LibraAccount_sequence_number_for_account (arg0: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t1: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);

    old_size := local_counter;
    local_counter := local_counter + 4;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, LibraAccount_T_sequence_number);

    call tmp := ReadRef(t2);
    assume is#Integer(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_sequence_number_for_account_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := LibraAccount_sequence_number_for_account(arg0);
}

procedure {:inline 1} LibraAccount_sequence_number (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := local_counter;
    local_counter := local_counter + 4;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call t2 := BorrowGlobal(GetLocal(m, old_size + 1), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t3 := LibraAccount_sequence_number_for_account(t2);
    if (abort_flag) { goto Label_Abort; }
    assume is#Integer(t3);

    m := UpdateLocal(m, old_size + 3, t3);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_sequence_number_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_sequence_number(arg0);
}

procedure {:inline 1} LibraAccount_delegated_key_rotation_capability (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(BooleanType())
    var t4: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call t2 := BorrowGlobal(GetLocal(m, old_size + 1), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t3 := BorrowField(t2, LibraAccount_T_delegated_key_rotation_capability);

    call tmp := ReadRef(t3);
    assume is#Boolean(tmp);

    m := UpdateLocal(m, old_size + 4, tmp);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_delegated_key_rotation_capability_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_delegated_key_rotation_capability(arg0);
}

procedure {:inline 1} LibraAccount_delegated_withdrawal_capability (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(BooleanType())
    var t4: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call t2 := BorrowGlobal(GetLocal(m, old_size + 1), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t3 := BorrowField(t2, LibraAccount_T_delegated_withdrawal_capability);

    call tmp := ReadRef(t3);
    assume is#Boolean(tmp);

    m := UpdateLocal(m, old_size + 4, tmp);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_delegated_withdrawal_capability_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_delegated_withdrawal_capability(arg0);
}

procedure {:inline 1} LibraAccount_withdrawal_capability_address (arg0: Reference) returns (ret0: Reference)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t1: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t2: Reference; // ReferenceType(AddressType())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);

    old_size := local_counter;
    local_counter := local_counter + 3;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, LibraAccount_WithdrawalCapability_account_address);

    ret0 := t2;
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultReference;
}

procedure LibraAccount_withdrawal_capability_address_verify (arg0: Reference) returns (ret0: Reference)
{
    call ret0 := LibraAccount_withdrawal_capability_address(arg0);
}

procedure {:inline 1} LibraAccount_key_rotation_capability_address (arg0: Reference) returns (ret0: Reference)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var t1: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var t2: Reference; // ReferenceType(AddressType())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);

    old_size := local_counter;
    local_counter := local_counter + 3;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, LibraAccount_KeyRotationCapability_account_address);

    ret0 := t2;
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultReference;
}

procedure LibraAccount_key_rotation_capability_address_verify (arg0: Reference) returns (ret0: Reference)
{
    call ret0 := LibraAccount_key_rotation_capability_address(arg0);
}

procedure {:inline 1} LibraAccount_exists (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := local_counter;
    local_counter := local_counter + 3;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := Exists(GetLocal(m, old_size + 1), LibraAccount_T_type_value());
    m := UpdateLocal(m, old_size + 2, tmp);

    ret0 := GetLocal(m, old_size + 2);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_exists_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_exists(arg0);
}

procedure {:inline 1} LibraAccount_prologue (arg0: Value, arg1: Value, arg2: Value, arg3: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // ByteArrayType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // AddressType()
    var t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // AddressType()
    var t11: Value; // AddressType()
    var t12: Value; // BooleanType()
    var t13: Value; // BooleanType()
    var t14: Value; // IntegerType()
    var t15: Value; // AddressType()
    var t16: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t17: Value; // ByteArrayType()
    var t18: Value; // ByteArrayType()
    var t19: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t20: Reference; // ReferenceType(ByteArrayType())
    var t21: Value; // ByteArrayType()
    var t22: Value; // BooleanType()
    var t23: Value; // BooleanType()
    var t24: Value; // IntegerType()
    var t25: Value; // IntegerType()
    var t26: Value; // IntegerType()
    var t27: Value; // IntegerType()
    var t28: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t29: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t30: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t31: Value; // IntegerType()
    var t32: Value; // IntegerType()
    var t33: Value; // IntegerType()
    var t34: Value; // BooleanType()
    var t35: Value; // BooleanType()
    var t36: Value; // IntegerType()
    var t37: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t38: Reference; // ReferenceType(IntegerType())
    var t39: Value; // IntegerType()
    var t40: Value; // IntegerType()
    var t41: Value; // IntegerType()
    var t42: Value; // BooleanType()
    var t43: Value; // BooleanType()
    var t44: Value; // IntegerType()
    var t45: Value; // IntegerType()
    var t46: Value; // IntegerType()
    var t47: Value; // BooleanType()
    var t48: Value; // BooleanType()
    var t49: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#ByteArray(arg1);
    assume is#Integer(arg2);
    assume is#Integer(arg3);

    old_size := local_counter;
    local_counter := local_counter + 50;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);
    m := UpdateLocal(m, old_size + 2, arg2);
    m := UpdateLocal(m, old_size + 3, arg3);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 10, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 10));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := Exists(GetLocal(m, old_size + 11), LibraAccount_T_type_value());
    m := UpdateLocal(m, old_size + 12, tmp);

    call tmp := Not(GetLocal(m, old_size + 12));
    m := UpdateLocal(m, old_size + 13, tmp);

    tmp := GetLocal(m, old_size + 13);
    if (!b#Boolean(tmp)) { goto Label_8; }

    call tmp := LdConst(5);
    m := UpdateLocal(m, old_size + 14, tmp);

    goto Label_Abort;

Label_8:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 15, tmp);

    call t16 := BorrowGlobal(GetLocal(m, old_size + 15), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t5 := CopyOrMoveRef(t16);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 17, tmp);

    call t18 := Hash_sha3_256(GetLocal(m, old_size + 17));
    if (abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t18);

    m := UpdateLocal(m, old_size + 18, t18);

    call t19 := CopyOrMoveRef(t5);

    call t20 := BorrowField(t19, LibraAccount_T_authentication_key);

    call tmp := ReadRef(t20);
    assume is#ByteArray(tmp);

    m := UpdateLocal(m, old_size + 21, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 18), GetLocal(m, old_size + 21)));
    m := UpdateLocal(m, old_size + 22, tmp);

    call tmp := Not(GetLocal(m, old_size + 22));
    m := UpdateLocal(m, old_size + 23, tmp);

    tmp := GetLocal(m, old_size + 23);
    if (!b#Boolean(tmp)) { goto Label_21; }

    call tmp := LdConst(2);
    m := UpdateLocal(m, old_size + 24, tmp);

    goto Label_Abort;

Label_21:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 25, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 26, tmp);

    call tmp := Mul(GetLocal(m, old_size + 25), GetLocal(m, old_size + 26));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 27, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 27));
    m := UpdateLocal(m, old_size + 7, tmp);

    call t28 := CopyOrMoveRef(t5);

    call t29 := FreezeRef(t28);

    call t6 := CopyOrMoveRef(t29);

    call t30 := CopyOrMoveRef(t6);

    call t31 := LibraAccount_balance_for_account(t30);
    if (abort_flag) { goto Label_Abort; }
    assume is#Integer(t31);

    m := UpdateLocal(m, old_size + 31, t31);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 31));
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 32, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 33, tmp);

    call tmp := Ge(GetLocal(m, old_size + 32), GetLocal(m, old_size + 33));
    m := UpdateLocal(m, old_size + 34, tmp);

    call tmp := Not(GetLocal(m, old_size + 34));
    m := UpdateLocal(m, old_size + 35, tmp);

    tmp := GetLocal(m, old_size + 35);
    if (!b#Boolean(tmp)) { goto Label_38; }

    call tmp := LdConst(6);
    m := UpdateLocal(m, old_size + 36, tmp);

    goto Label_Abort;

Label_38:
    call t37 := CopyOrMoveRef(t5);

    call t38 := BorrowField(t37, LibraAccount_T_sequence_number);

    call tmp := ReadRef(t38);
    assume is#Integer(tmp);

    m := UpdateLocal(m, old_size + 39, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 39));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 40, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 9));
    m := UpdateLocal(m, old_size + 41, tmp);

    call tmp := Ge(GetLocal(m, old_size + 40), GetLocal(m, old_size + 41));
    m := UpdateLocal(m, old_size + 42, tmp);

    call tmp := Not(GetLocal(m, old_size + 42));
    m := UpdateLocal(m, old_size + 43, tmp);

    tmp := GetLocal(m, old_size + 43);
    if (!b#Boolean(tmp)) { goto Label_49; }

    call tmp := LdConst(3);
    m := UpdateLocal(m, old_size + 44, tmp);

    goto Label_Abort;

Label_49:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 45, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 9));
    m := UpdateLocal(m, old_size + 46, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 45), GetLocal(m, old_size + 46)));
    m := UpdateLocal(m, old_size + 47, tmp);

    call tmp := Not(GetLocal(m, old_size + 47));
    m := UpdateLocal(m, old_size + 48, tmp);

    tmp := GetLocal(m, old_size + 48);
    if (!b#Boolean(tmp)) { goto Label_56; }

    call tmp := LdConst(4);
    m := UpdateLocal(m, old_size + 49, tmp);

    goto Label_Abort;

Label_56:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_prologue_verify (arg0: Value, arg1: Value, arg2: Value, arg3: Value) returns ()
{
    call LibraAccount_prologue(arg0, arg1, arg2, arg3);
}

procedure {:inline 1} LibraAccount_epilogue (arg0: Value, arg1: Value, arg2: Value, arg3: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Value; // IntegerType()
    var t8: Value; // LibraCoin_T_type_value()
    var t9: Value; // AddressType()
    var t10: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t11: Value; // IntegerType()
    var t12: Value; // IntegerType()
    var t13: Value; // IntegerType()
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t17: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t18: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t19: Value; // IntegerType()
    var t20: Value; // IntegerType()
    var t21: Value; // BooleanType()
    var t22: Value; // BooleanType()
    var t23: Value; // IntegerType()
    var t24: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t25: Value; // IntegerType()
    var t26: Value; // LibraCoin_T_type_value()
    var t27: Value; // IntegerType()
    var t28: Value; // IntegerType()
    var t29: Value; // IntegerType()
    var t30: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t31: Reference; // ReferenceType(IntegerType())
    var t32: Value; // AddressType()
    var t33: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t34: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t35: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t36: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#Integer(arg1);
    assume is#Integer(arg2);
    assume is#Integer(arg3);

    old_size := local_counter;
    local_counter := local_counter + 37;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);
    m := UpdateLocal(m, old_size + 2, arg2);
    m := UpdateLocal(m, old_size + 3, arg3);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 9, tmp);

    call t10 := BorrowGlobal(GetLocal(m, old_size + 9), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t4 := CopyOrMoveRef(t10);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 12, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 13, tmp);

    call tmp := Sub(GetLocal(m, old_size + 12), GetLocal(m, old_size + 13));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 14, tmp);

    call tmp := Mul(GetLocal(m, old_size + 11), GetLocal(m, old_size + 14));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 15, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 15));
    m := UpdateLocal(m, old_size + 7, tmp);

    call t16 := CopyOrMoveRef(t4);

    call t17 := FreezeRef(t16);

    call t6 := CopyOrMoveRef(t17);

    call t18 := CopyOrMoveRef(t6);

    call t19 := LibraAccount_balance_for_account(t18);
    if (abort_flag) { goto Label_Abort; }
    assume is#Integer(t19);

    m := UpdateLocal(m, old_size + 19, t19);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 20, tmp);

    call tmp := Ge(GetLocal(m, old_size + 19), GetLocal(m, old_size + 20));
    m := UpdateLocal(m, old_size + 21, tmp);

    call tmp := Not(GetLocal(m, old_size + 21));
    m := UpdateLocal(m, old_size + 22, tmp);

    tmp := GetLocal(m, old_size + 22);
    if (!b#Boolean(tmp)) { goto Label_20; }

    call tmp := LdConst(6);
    m := UpdateLocal(m, old_size + 23, tmp);

    goto Label_Abort;

Label_20:
    call t24 := CopyOrMoveRef(t4);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 25, tmp);

    call t26 := LibraAccount_withdraw_from_account(t24, GetLocal(m, old_size + 25));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t26);

    m := UpdateLocal(m, old_size + 26, t26);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 26));
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 27, tmp);

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 28, tmp);

    call tmp := Add(GetLocal(m, old_size + 27), GetLocal(m, old_size + 28));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 29, tmp);

    call t30 := CopyOrMoveRef(t4);

    call t31 := BorrowField(t30, LibraAccount_T_sequence_number);

    call WriteRef(t31, GetLocal(m, old_size + 29));

    call tmp := LdAddr(4078);
    m := UpdateLocal(m, old_size + 32, tmp);

    call t33 := BorrowGlobal(GetLocal(m, old_size + 32), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t5 := CopyOrMoveRef(t33);

    call t34 := CopyOrMoveRef(t5);

    call t35 := BorrowField(t34, LibraAccount_T_balance);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 36, tmp);

    call LibraCoin_deposit(t35, GetLocal(m, old_size + 36));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_epilogue_verify (arg0: Value, arg1: Value, arg2: Value, arg3: Value) returns ()
{
    call LibraAccount_epilogue(arg0, arg1, arg2, arg3);
}

procedure {:inline 1} LibraAccount_fresh_guid (arg0: Reference, arg1: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // ByteArrayType()
    var t4: Value; // ByteArrayType()
    var t5: Value; // ByteArrayType()
    var t6: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t7: Reference; // ReferenceType(IntegerType())
    var t8: Value; // AddressType()
    var t9: Value; // ByteArrayType()
    var t10: Reference; // ReferenceType(IntegerType())
    var t11: Value; // IntegerType()
    var t12: Value; // ByteArrayType()
    var t13: Reference; // ReferenceType(IntegerType())
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Value; // IntegerType()
    var t17: Reference; // ReferenceType(IntegerType())
    var t18: Value; // ByteArrayType()
    var t19: Value; // ByteArrayType()
    var t20: Value; // ByteArrayType()
    var t21: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);
    assume is#Address(arg1);

    old_size := local_counter;
    local_counter := local_counter + 22;
    t0 := arg0;
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call t6 := CopyOrMoveRef(t0);

    call t7 := BorrowField(t6, LibraAccount_EventHandleGenerator_counter);

    call t2 := CopyOrMoveRef(t7);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 8, tmp);

    call t9 := AddressUtil_address_to_bytes(GetLocal(m, old_size + 8));
    if (abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t9);

    m := UpdateLocal(m, old_size + 9, t9);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 9));
    m := UpdateLocal(m, old_size + 5, tmp);

    call t10 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t10);
    assume is#Integer(tmp);

    m := UpdateLocal(m, old_size + 11, tmp);

    call t12 := U64Util_u64_to_bytes(GetLocal(m, old_size + 11));
    if (abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t12);

    m := UpdateLocal(m, old_size + 12, t12);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 12));
    m := UpdateLocal(m, old_size + 3, tmp);

    call t13 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t13);
    assume is#Integer(tmp);

    m := UpdateLocal(m, old_size + 14, tmp);

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 15, tmp);

    call tmp := Add(GetLocal(m, old_size + 14), GetLocal(m, old_size + 15));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 16, tmp);

    call t17 := CopyOrMoveRef(t2);

    call WriteRef(t17, GetLocal(m, old_size + 16));

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 18, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 19, tmp);

    call t20 := BytearrayUtil_bytearray_concat(GetLocal(m, old_size + 18), GetLocal(m, old_size + 19));
    if (abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t20);

    m := UpdateLocal(m, old_size + 20, t20);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 20));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 21, tmp);

    ret0 := GetLocal(m, old_size + 21);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_fresh_guid_verify (arg0: Reference, arg1: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_fresh_guid(arg0, arg1);
}

procedure {:inline 1} LibraAccount_new_event_handle_impl (tv0: TypeValue, arg0: Reference, arg1: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t1: Value; // AddressType()
    var t2: Value; // IntegerType()
    var t3: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t4: Value; // AddressType()
    var t5: Value; // ByteArrayType()
    var t6: Value; // LibraAccount_EventHandle_type_value(tv0)

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);
    assume is#Address(arg1);

    old_size := local_counter;
    local_counter := local_counter + 7;
    t0 := arg0;
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := CopyOrMoveRef(t0);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 4, tmp);

    call t5 := LibraAccount_fresh_guid(t3, GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t5);

    m := UpdateLocal(m, old_size + 5, t5);

    assume is#Integer(GetLocal(m, old_size + 2));

    assume is#ByteArray(GetLocal(m, old_size + 5));

    call tmp := Pack_LibraAccount_EventHandle(tv0, GetLocal(m, old_size + 2), GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 6, tmp);

    ret0 := GetLocal(m, old_size + 6);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_new_event_handle_impl_verify (tv0: TypeValue, arg0: Reference, arg1: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_new_event_handle_impl(tv0: TypeValue, arg0, arg1);
}

procedure {:inline 1} LibraAccount_new_event_handle (tv0: TypeValue) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t1: Value; // ByteArrayType()
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t5: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t6: Value; // AddressType()
    var t7: Value; // LibraAccount_EventHandle_type_value(tv0)

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 8;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := BorrowGlobal(GetLocal(m, old_size + 2), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t0 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, LibraAccount_T_event_generator);

    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 6, tmp);

    call t7 := LibraAccount_new_event_handle_impl(tv0, t5, GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t7);

    m := UpdateLocal(m, old_size + 7, t7);

    ret0 := GetLocal(m, old_size + 7);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_new_event_handle_verify (tv0: TypeValue) returns (ret0: Value)
{
    call ret0 := LibraAccount_new_event_handle(tv0: TypeValue);
}

procedure {:inline 1} LibraAccount_emit_event (tv0: TypeValue, arg0: Reference, arg1: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(tv0))
    var t1: Value; // tv0
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // ByteArrayType()
    var t4: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(tv0))
    var t5: Reference; // ReferenceType(ByteArrayType())
    var t6: Value; // ByteArrayType()
    var t7: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(tv0))
    var t8: Reference; // ReferenceType(IntegerType())
    var t9: Value; // ByteArrayType()
    var t10: Reference; // ReferenceType(IntegerType())
    var t11: Value; // IntegerType()
    var t12: Value; // tv0
    var t13: Reference; // ReferenceType(IntegerType())
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Value; // IntegerType()
    var t17: Reference; // ReferenceType(IntegerType())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);

    old_size := local_counter;
    local_counter := local_counter + 18;
    t0 := arg0;
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, LibraAccount_EventHandle_guid);

    call tmp := ReadRef(t5);
    assume is#ByteArray(tmp);

    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 3, tmp);

    call t7 := CopyOrMoveRef(t0);

    call t8 := BorrowField(t7, LibraAccount_EventHandle_counter);

    call t2 := CopyOrMoveRef(t8);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 9, tmp);

    call t10 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t10);
    assume is#Integer(tmp);

    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 12, tmp);

    call LibraAccount_write_to_event_store(tv0, GetLocal(m, old_size + 9), GetLocal(m, old_size + 11), GetLocal(m, old_size + 12));
    if (abort_flag) { goto Label_Abort; }

    call t13 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t13);
    assume is#Integer(tmp);

    m := UpdateLocal(m, old_size + 14, tmp);

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 15, tmp);

    call tmp := Add(GetLocal(m, old_size + 14), GetLocal(m, old_size + 15));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 16, tmp);

    call t17 := CopyOrMoveRef(t2);

    call WriteRef(t17, GetLocal(m, old_size + 16));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_emit_event_verify (tv0: TypeValue, arg0: Reference, arg1: Value) returns ()
{
    call LibraAccount_emit_event(tv0: TypeValue, arg0, arg1);
}

procedure {:inline 1} LibraAccount_write_to_event_store (tv0: TypeValue, arg0: Value, arg1: Value, arg2: Value) returns ();procedure {:inline 1} LibraAccount_destroy_handle (tv0: TypeValue, arg0: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // LibraAccount_EventHandle_type_value(tv0)
    var t1: Value; // ByteArrayType()
    var t2: Value; // IntegerType()
    var t3: Value; // LibraAccount_EventHandle_type_value(tv0)
    var t4: Value; // IntegerType()
    var t5: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(arg0);

    old_size := local_counter;
    local_counter := local_counter + 6;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call t4, t5 := Unpack_LibraAccount_EventHandle(GetLocal(m, old_size + 3));
    assume is#Integer(t4);

    assume is#ByteArray(t5);

    m := UpdateLocal(m, old_size + 4, t4);
    m := UpdateLocal(m, old_size + 5, t5);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 2, tmp);

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_destroy_handle_verify (tv0: TypeValue, arg0: Value) returns ()
{
    call LibraAccount_destroy_handle(tv0: TypeValue, arg0);
}
