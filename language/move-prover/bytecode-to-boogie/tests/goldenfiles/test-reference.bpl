

// ** structs of module TestReference

const unique TestReference_T: TypeName;
const TestReference_T_value: FieldName;
axiom TestReference_T_value == 0;
function TestReference_T_type_value(): TypeValue {
    StructType(TestReference_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}

procedure {:inline 1} Pack_TestReference_T(v0: Value) returns (v: Value)
{
    assume is#Integer(v0);
    v := Vector(ExtendValueArray(EmptyValueArray, v0));

}

procedure {:inline 1} Unpack_TestReference_T(v: Value) returns (v0: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, TestReference_T_value);
}



// ** functions of module TestReference

procedure {:inline 1} TestReference_mut_b (arg0: Reference) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(IntegerType())
    var t1: Value; // IntegerType()
    var t2: Reference; // ReferenceType(IntegerType())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 3;
    t0 := arg0;

    // bytecode translation starts here
    call tmp := LdConst(10);
    m := UpdateLocal(m, old_size + 1, tmp);
    if (abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t0);
    if (abort_flag) { goto Label_Abort; }

    call WriteRef(t2, GetLocal(m, old_size + 1));
    if (abort_flag) { goto Label_Abort; }

    return;
    if (abort_flag) { goto Label_Abort; }

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestReference_mut_b_verify (arg0: Reference) returns ()
{
    call TestReference_mut_b(arg0);
}

procedure {:inline 1} TestReference_mut_ref () returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Reference; // ReferenceType(IntegerType())
    var t2: Value; // IntegerType()
    var t3: Reference; // ReferenceType(IntegerType())
    var t4: Reference; // ReferenceType(IntegerType())
    var t5: Reference; // ReferenceType(IntegerType())
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // BooleanType()
    var t10: Value; // BooleanType()
    var t11: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 12;

    // bytecode translation starts here
    call tmp := LdConst(20);
    m := UpdateLocal(m, old_size + 2, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 0, tmp);
    if (abort_flag) { goto Label_Abort; }

    call t3 := BorrowLoc(old_size+0);
    if (abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t3);
    if (abort_flag) { goto Label_Abort; }

    call t4 := CopyOrMoveRef(t1);
    if (abort_flag) { goto Label_Abort; }

    call TestReference_mut_b(t4);
    if (abort_flag) { goto Label_Abort; }

    call t5 := CopyOrMoveRef(t1);
    if (abort_flag) { goto Label_Abort; }

    call tmp := ReadRef(t5);
    assume is#Integer(tmp);

    m := UpdateLocal(m, old_size + 6, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 0, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 7, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(10);
    m := UpdateLocal(m, old_size + 8, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 7), GetLocal(m, old_size + 8)));
    m := UpdateLocal(m, old_size + 9, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Not(GetLocal(m, old_size + 9));
    m := UpdateLocal(m, old_size + 10, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := GetLocal(m, old_size + 10);
    if (!b#Boolean(tmp)) { goto Label_16; }
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 11, tmp);
    if (abort_flag) { goto Label_Abort; }

    goto Label_Abort;
    if (abort_flag) { goto Label_Abort; }

Label_16:
    return;
    if (abort_flag) { goto Label_Abort; }

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestReference_mut_ref_verify () returns ()
{
    call TestReference_mut_ref();
}
