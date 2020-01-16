

// ** structs of module TestSpecs

const unique TestSpecs_R: TypeName;
const TestSpecs_R_x: FieldName;
axiom TestSpecs_R_x == 0;
function TestSpecs_R_type_value(): TypeValue {
    StructType(TestSpecs_R, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}

procedure {:inline 1} Pack_TestSpecs_R(v0: Value) returns (v: Value)
{
    assume IsValidInteger(v0);
    v := Vector(ExtendValueArray(EmptyValueArray, v0));

}

procedure {:inline 1} Unpack_TestSpecs_R(v: Value) returns (v0: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, TestSpecs_R_x);
}



// ** functions of module TestSpecs

procedure {:inline 1} TestSpecs_create_resource () returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(ExistsResource(m, TestSpecs_R_type_value(), a#Address(Address(TxnSenderAddress(txn)))));
ensures old(!(b#Boolean(ExistsResource(m, TestSpecs_R_type_value(), a#Address(Address(TxnSenderAddress(txn))))))) ==> !abort_flag;
ensures old(b#Boolean(ExistsResource(m, TestSpecs_R_type_value(), a#Address(Address(TxnSenderAddress(txn)))))) ==> abort_flag;
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // BooleanType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // TestSpecs_R_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 5;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := Exists(GetLocal(m, old_size + 0), TestSpecs_R_type_value());
    m := UpdateLocal(m, old_size + 1, tmp);

    tmp := GetLocal(m, old_size + 1);
    if (!b#Boolean(tmp)) { goto Label_5; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 2, tmp);

    goto Label_Abort;

Label_5:
    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 3, tmp);

    assume IsValidInteger(GetLocal(m, old_size + 3));

    call tmp := Pack_TestSpecs_R(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, tmp);

    call MoveToSender(TestSpecs_R_type_value(), GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestSpecs_create_resource_verify () returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestSpecs_create_resource();
}

procedure {:inline 1} TestSpecs_create_resource_error () returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(ExistsResource(m, TestSpecs_R_type_value(), a#Address(Address(TxnSenderAddress(txn)))));
ensures old(!(b#Boolean(ExistsResource(m, TestSpecs_R_type_value(), a#Address(Address(TxnSenderAddress(txn))))))) ==> !abort_flag;
ensures old(b#Boolean(ExistsResource(m, TestSpecs_R_type_value(), a#Address(Address(TxnSenderAddress(txn)))))) ==> abort_flag;
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // BooleanType()
    var t2: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 3;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := Exists(GetLocal(m, old_size + 0), TestSpecs_R_type_value());
    m := UpdateLocal(m, old_size + 1, tmp);

    tmp := GetLocal(m, old_size + 1);
    if (!b#Boolean(tmp)) { goto Label_5; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 2, tmp);

    goto Label_Abort;

Label_5:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestSpecs_create_resource_error_verify () returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestSpecs_create_resource_error();
}
