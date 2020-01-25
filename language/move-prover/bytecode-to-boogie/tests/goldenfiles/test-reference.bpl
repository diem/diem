

// ** structs of module TestReference

const unique TestReference_T: TypeName;
const TestReference_T_value: FieldName;
axiom TestReference_T_value == 0;
function TestReference_T_type_value(): TypeValue {
    StructType(TestReference_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_TestReference_T(value: Value) returns (_struct: Value)
{
    assume IsValidU64(value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, value));

}

procedure {:inline 1} Unpack_TestReference_T(_struct: Value) returns (value: Value)
{
    assume is#Vector(_struct);
    value := SelectField(_struct, TestReference_T_value);
    assume IsValidU64(value);
}



// ** functions of module TestReference

procedure {:inline 1} TestReference_mut_b (b: Reference) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Reference; // ReferenceType(IntegerType())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(Dereference(m, b));
    assume IsValidReferenceParameter(m, local_counter, b);

    old_size := local_counter;
    local_counter := local_counter + 3;

    // bytecode translation starts here
    call tmp := LdConst(10);
    m := UpdateLocal(m, old_size + 1, tmp);

    call t2 := CopyOrMoveRef(b);

    call WriteRef(t2, GetLocal(m, old_size + 1));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestReference_mut_b_verify (b: Reference) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestReference_mut_b(b);
}

procedure {:inline 1} TestReference_mut_ref () returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> abort_flag;
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

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 0, tmp);

    call t3 := BorrowLoc(old_size+0);

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call TestReference_mut_b(t4);
    if (abort_flag) { goto Label_Abort; }

    call t5 := CopyOrMoveRef(t1);

    call tmp := ReadRef(t5);
    assume IsValidU64(tmp);

    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := LdConst(10);
    m := UpdateLocal(m, old_size + 8, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 7), GetLocal(m, old_size + 8)));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := Not(GetLocal(m, old_size + 9));
    m := UpdateLocal(m, old_size + 10, tmp);

    tmp := GetLocal(m, old_size + 10);
    if (!b#Boolean(tmp)) { goto Label_16; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 11, tmp);

    goto Label_Abort;

Label_16:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestReference_mut_ref_verify () returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestReference_mut_ref();
}
