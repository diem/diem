

// ** synthetics of module TestGenerics



// ** structs of module TestGenerics

const unique TestGenerics_R: TypeName;
const TestGenerics_R_v: FieldName;
axiom TestGenerics_R_v == 0;
function TestGenerics_R_type_value(): TypeValue {
    StructType(TestGenerics_R, ExtendTypeValueArray(EmptyTypeValueArray, Vector_T_type_value(IntegerType())))
}
function {:inline 1} $TestGenerics_R_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && $Vector_T_is_well_formed(SelectField(__this, TestGenerics_R_v))
}

procedure {:inline 1} Pack_TestGenerics_R(module_idx: int, func_idx: int, var_idx: int, code_idx: int, v: Value) returns (_struct: Value)
{
    assume $Vector_T_is_well_formed(v);
    _struct := Vector(ExtendValueArray(EmptyValueArray, v));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_TestGenerics_R(_struct: Value) returns (v: Value)
{
    assume is#Vector(_struct);
    v := SelectField(_struct, TestGenerics_R_v);
    assume $Vector_T_is_well_formed(v);
}

const unique TestGenerics_T: TypeName;
const TestGenerics_T_v: FieldName;
axiom TestGenerics_T_v == 0;
function TestGenerics_T_type_value(tv0: TypeValue): TypeValue {
    StructType(TestGenerics_T, ExtendTypeValueArray(EmptyTypeValueArray, Vector_T_type_value(tv0)))
}
function {:inline 1} $TestGenerics_T_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && $Vector_T_is_well_formed(SelectField(__this, TestGenerics_T_v))
}

procedure {:inline 1} Pack_TestGenerics_T(module_idx: int, func_idx: int, var_idx: int, code_idx: int, tv0: TypeValue, v: Value) returns (_struct: Value)
{
    assume $Vector_T_is_well_formed(v);
    _struct := Vector(ExtendValueArray(EmptyValueArray, v));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_TestGenerics_T(_struct: Value) returns (v: Value)
{
    assume is#Vector(_struct);
    v := SelectField(_struct, TestGenerics_T_v);
    assume $Vector_T_is_well_formed(v);
}



// ** functions of module TestGenerics

procedure {:inline 1} TestGenerics_move2 (x1: Value, x2: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var v: Value; // Vector_T_type_value(IntegerType())
    var r: Value; // TestGenerics_R_type_value()
    var __t4: Value; // Vector_T_type_value(IntegerType())
    var __t5: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t6: Value; // IntegerType()
    var __t7: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t8: Value; // IntegerType()
    var __t9: Value; // Vector_T_type_value(IntegerType())
    var __t10: Value; // TestGenerics_R_type_value()
    var __t11: Value; // TestGenerics_R_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU64(x1);
    __m := UpdateLocal(__m, __frame + 0, x1);
    assume $DebugTrackLocal(1, 0, 0, 162, x1);
    assume IsValidU64(x2);
    __m := UpdateLocal(__m, __frame + 1, x2);
    assume $DebugTrackLocal(1, 0, 1, 162, x2);

    // increase the local counter
    __local_counter := __local_counter + 12;

    // bytecode translation starts here
    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 0, 260);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 0, 2, 256, __tmp);

    call __t5 := BorrowLoc(__frame + 2, Vector_T_type_value(IntegerType()));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 0, 289);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 0, 2, 289, GetLocal(__m, __frame + 2));

    call __t7 := BorrowLoc(__frame + 2, Vector_T_type_value(IntegerType()));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call Vector_push_back(IntegerType(), __t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 0, 338);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 0, 2, 338, GetLocal(__m, __frame + 2));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := Pack_TestGenerics_R(1, 0, 3, 391, GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 10));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(1, 0, 3, 387, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call MoveToSender(TestGenerics_R_type_value(), GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 0, 416);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestGenerics_move2_verify (x1: Value, x2: Value) returns ()
{
    call InitVerification();
    call TestGenerics_move2(x1, x2);
}

procedure {:inline 1} TestGenerics_create (tv0: TypeValue, x: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var v: Value; // Vector_T_type_value(tv0)
    var __t2: Value; // Vector_T_type_value(tv0)
    var __t3: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t4: Value; // tv0
    var __t5: Value; // Vector_T_type_value(tv0)
    var __t6: Value; // TestGenerics_T_type_value(tv0)
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    __m := UpdateLocal(__m, __frame + 0, x);
    assume $DebugTrackLocal(1, 1, 0, 471, x);

    // increase the local counter
    __local_counter := __local_counter + 7;

    // bytecode translation starts here
    call __t2 := Vector_empty(tv0);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 1, 561);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 1, 1, 557, __tmp);

    call __t3 := BorrowLoc(__frame + 1, Vector_T_type_value(tv0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(tv0, __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 1, 588);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 1, 1, 588, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Pack_TestGenerics_T(0, 0, 0, 0, tv0, GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __ret0 := GetLocal(__m, __frame + 6);
    assume $DebugTrackLocal(1, 1, 2, 634, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestGenerics_create_verify (tv0: TypeValue, x: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestGenerics_create(tv0, x);
}

procedure {:inline 1} TestGenerics_overcomplicated_equals (tv0: TypeValue, x: Value, y: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var r: Value; // BooleanType()
    var x1: Value; // TestGenerics_T_type_value(tv0)
    var y1: Value; // TestGenerics_T_type_value(tv0)
    var __t5: Value; // tv0
    var __t6: Value; // TestGenerics_T_type_value(tv0)
    var __t7: Value; // tv0
    var __t8: Value; // TestGenerics_T_type_value(tv0)
    var __t9: Value; // TestGenerics_T_type_value(tv0)
    var __t10: Value; // TestGenerics_T_type_value(tv0)
    var __t11: Value; // BooleanType()
    var __t12: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    __m := UpdateLocal(__m, __frame + 0, x);
    assume $DebugTrackLocal(1, 2, 0, 672, x);
    __m := UpdateLocal(__m, __frame + 1, y);
    assume $DebugTrackLocal(1, 2, 1, 672, y);

    // increase the local counter
    __local_counter := __local_counter + 13;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := TestGenerics_create(tv0, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 2, 827);
      goto Label_Abort;
    }
    assume $TestGenerics_T_is_well_formed(__t6);

    __m := UpdateLocal(__m, __frame + 6, __t6);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(1, 2, 3, 822, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := TestGenerics_create(tv0, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 2, 865);
      goto Label_Abort;
    }
    assume $TestGenerics_T_is_well_formed(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 4, __tmp);
    assume $DebugTrackLocal(1, 2, 4, 860, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10)));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 2, 2, 898, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    __ret0 := GetLocal(__m, __frame + 12);
    assume $DebugTrackLocal(1, 2, 5, 932, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestGenerics_overcomplicated_equals_verify (tv0: TypeValue, x: Value, y: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestGenerics_overcomplicated_equals(tv0, x, y);
}

procedure {:inline 1} TestGenerics_test () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var r: Value; // BooleanType()
    var __t1: Value; // IntegerType()
    var __t2: Value; // IntegerType()
    var __t3: Value; // BooleanType()
    var __t4: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := TestGenerics_overcomplicated_equals(IntegerType(), GetLocal(__m, __frame + 1), GetLocal(__m, __frame + 2));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1010);
      goto Label_Abort;
    }
    assume is#Boolean(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 3, 0, 1006, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(1, 3, 1, 1056, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestGenerics_test_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestGenerics_test();
}
