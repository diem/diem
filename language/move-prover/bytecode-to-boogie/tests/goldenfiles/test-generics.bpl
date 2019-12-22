

// ** structs of module TestGenerics

const unique TestGenerics_R: TypeName;
const TestGenerics_R_v: FieldName;
axiom TestGenerics_R_v == 0;
function TestGenerics_R_type_value(): TypeValue {
    StructType(TestGenerics_R, ExtendTypeValueArray(EmptyTypeValueArray, Vector_T_type_value(IntegerType())))
}
procedure {:inline 1} Pack_TestGenerics_R(v: Value) returns (_struct: Value)
{
    assume is#Vector(v);
    _struct := Vector(ExtendValueArray(EmptyValueArray, v));

}

procedure {:inline 1} Unpack_TestGenerics_R(_struct: Value) returns (v: Value)
{
    assume is#Vector(_struct);
    v := SelectField(_struct, TestGenerics_R_v);
    assume is#Vector(v);
}

const unique TestGenerics_T: TypeName;
const TestGenerics_T_v: FieldName;
axiom TestGenerics_T_v == 0;
function TestGenerics_T_type_value(tv0: TypeValue): TypeValue {
    StructType(TestGenerics_T, ExtendTypeValueArray(EmptyTypeValueArray, Vector_T_type_value(tv0)))
}
procedure {:inline 1} Pack_TestGenerics_T(tv0: TypeValue, v: Value) returns (_struct: Value)
{
    assume is#Vector(v);
    _struct := Vector(ExtendValueArray(EmptyValueArray, v));

}

procedure {:inline 1} Unpack_TestGenerics_T(_struct: Value) returns (v: Value)
{
    assume is#Vector(_struct);
    v := SelectField(_struct, TestGenerics_T_v);
    assume is#Vector(v);
}



// ** functions of module TestGenerics

procedure {:inline 1} TestGenerics_move2 (x1: Value, x2: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t2: Value; // Vector_T_type_value(IntegerType())
    var t3: Value; // TestGenerics_R_type_value()
    var t4: Value; // Vector_T_type_value(IntegerType())
    var t5: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var t6: Value; // IntegerType()
    var t7: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var t8: Value; // IntegerType()
    var t9: Value; // Vector_T_type_value(IntegerType())
    var t10: Value; // TestGenerics_R_type_value()
    var t11: Value; // TestGenerics_R_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidInteger(x1);
    assume IsValidInteger(x2);

    old_size := local_counter;
    local_counter := local_counter + 12;
    m := UpdateLocal(m, old_size + 0, x1);
    m := UpdateLocal(m, old_size + 1, x2);

    // bytecode translation starts here
    call t4 := Vector_empty(IntegerType());
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t4);

    m := UpdateLocal(m, old_size + 4, t4);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 2, tmp);

    call t5 := BorrowLoc(old_size+2);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 6, tmp);

    call Vector_push_back(IntegerType(), t5, GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }

    call t7 := BorrowLoc(old_size+2);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 8, tmp);

    call Vector_push_back(IntegerType(), t7, GetLocal(m, old_size + 8));
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := Pack_TestGenerics_R(GetLocal(m, old_size + 9));
    m := UpdateLocal(m, old_size + 10, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 10));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 11, tmp);

    call MoveToSender(TestGenerics_R_type_value(), GetLocal(m, old_size + 11));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestGenerics_move2_verify (x1: Value, x2: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestGenerics_move2(x1, x2);
}

procedure {:inline 1} TestGenerics_create (tv0: TypeValue, x: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // Vector_T_type_value(tv0)
    var t2: Value; // Vector_T_type_value(tv0)
    var t3: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var t4: Value; // tv0
    var t5: Value; // Vector_T_type_value(tv0)
    var t6: Value; // TestGenerics_T_type_value(tv0)

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 0, x);

    // bytecode translation starts here
    call t2 := Vector_empty(tv0);
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t2);

    m := UpdateLocal(m, old_size + 2, t2);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 1, tmp);

    call t3 := BorrowLoc(old_size+1);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    call Vector_push_back(tv0, t3, GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := Pack_TestGenerics_T(tv0, GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 6, tmp);

    ret0 := GetLocal(m, old_size + 6);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestGenerics_create_verify (tv0: TypeValue, x: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestGenerics_create(tv0, x);
}

procedure {:inline 1} TestGenerics_overcomplicated_equals (tv0: TypeValue, x: Value, y: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t2: Value; // BooleanType()
    var t3: Value; // TestGenerics_T_type_value(tv0)
    var t4: Value; // TestGenerics_T_type_value(tv0)
    var t5: Value; // tv0
    var t6: Value; // TestGenerics_T_type_value(tv0)
    var t7: Value; // tv0
    var t8: Value; // TestGenerics_T_type_value(tv0)
    var t9: Value; // TestGenerics_T_type_value(tv0)
    var t10: Value; // TestGenerics_T_type_value(tv0)
    var t11: Value; // BooleanType()
    var t12: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 13;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 5, tmp);

    call t6 := TestGenerics_create(tv0, GetLocal(m, old_size + 5));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t6);

    m := UpdateLocal(m, old_size + 6, t6);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    call t8 := TestGenerics_create(tv0, GetLocal(m, old_size + 7));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t8);

    m := UpdateLocal(m, old_size + 8, t8);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 10, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 9), GetLocal(m, old_size + 10)));
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 11));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 12, tmp);

    ret0 := GetLocal(m, old_size + 12);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestGenerics_overcomplicated_equals_verify (tv0: TypeValue, x: Value, y: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestGenerics_overcomplicated_equals(tv0, x, y);
}

procedure {:inline 1} TestGenerics_test () returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // BooleanType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // BooleanType()
    var t4: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 5;

    // bytecode translation starts here
    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := TestGenerics_overcomplicated_equals(IntegerType(), GetLocal(m, old_size + 1), GetLocal(m, old_size + 2));
    if (abort_flag) { goto Label_Abort; }
    assume is#Boolean(t3);

    m := UpdateLocal(m, old_size + 3, t3);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestGenerics_test_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestGenerics_test();
}
