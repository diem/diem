

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
requires ExistsTxnSenderAccount(__m, __txn);
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
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 12;

    // process and type check arguments
    assume IsValidU64(x1);
    __m := UpdateLocal(__m, __frame + 0, x1);
    assume IsValidU64(x2);
    __m := UpdateLocal(__m, __frame + 1, x2);

    // bytecode translation starts here
    call t4 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t4);

    __m := UpdateLocal(__m, __frame + 4, t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call t5 := BorrowLoc(__frame + 2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    call t7 := BorrowLoc(__frame + 2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call Vector_push_back(IntegerType(), t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := Pack_TestGenerics_R(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 10));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call MoveToSender(TestGenerics_R_type_value(), GetLocal(__m, __frame + 11));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestGenerics_move2_verify (x1: Value, x2: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call TestGenerics_move2(x1, x2);
}

procedure {:inline 1} TestGenerics_create (tv0: TypeValue, x: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // Vector_T_type_value(tv0)
    var t2: Value; // Vector_T_type_value(tv0)
    var t3: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var t4: Value; // tv0
    var t5: Value; // Vector_T_type_value(tv0)
    var t6: Value; // TestGenerics_T_type_value(tv0)
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments
    __m := UpdateLocal(__m, __frame + 0, x);

    // bytecode translation starts here
    call t2 := Vector_empty(tv0);
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t2);

    __m := UpdateLocal(__m, __frame + 2, t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call t3 := BorrowLoc(__frame + 1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(tv0, t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Pack_TestGenerics_T(tv0, GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    ret0 := GetLocal(__m, __frame + 6);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure TestGenerics_create_verify (tv0: TypeValue, x: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := TestGenerics_create(tv0, x);
}

procedure {:inline 1} TestGenerics_overcomplicated_equals (tv0: TypeValue, x: Value, y: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
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
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 13;

    // process and type check arguments
    __m := UpdateLocal(__m, __frame + 0, x);
    __m := UpdateLocal(__m, __frame + 1, y);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call t6 := TestGenerics_create(tv0, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t6);

    __m := UpdateLocal(__m, __frame + 6, t6);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call t8 := TestGenerics_create(tv0, GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t8);

    __m := UpdateLocal(__m, __frame + 8, t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10)));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    ret0 := GetLocal(__m, __frame + 12);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure TestGenerics_overcomplicated_equals_verify (tv0: TypeValue, x: Value, y: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := TestGenerics_overcomplicated_equals(tv0, x, y);
}

procedure {:inline 1} TestGenerics_test () returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t0: Value; // BooleanType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // BooleanType()
    var t4: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call t3 := TestGenerics_overcomplicated_equals(IntegerType(), GetLocal(__m, __frame + 1), GetLocal(__m, __frame + 2));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Boolean(t3);

    __m := UpdateLocal(__m, __frame + 3, t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    ret0 := GetLocal(__m, __frame + 4);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure TestGenerics_test_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := TestGenerics_test();
}
