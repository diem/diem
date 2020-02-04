

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
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var __t1: Value; // IntegerType()
    var __t2: Reference; // ReferenceType(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 3;

    // process and type check arguments
    assume IsValidU64(Dereference(__m, b));
    assume IsValidReferenceParameter(__m, __frame, b);
    assume IsValidU64(Dereference(__m, b));
    assume $DebugTrackLocal(0, 0, 0, 71, Dereference(__m, b));

    // bytecode translation starts here
    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := CopyOrMoveRef(b);

    call WriteRef(__t2, GetLocal(__m, __frame + 1));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestReference_mut_b_verify (b: Reference) returns ()
{
    call InitVerification();
    call TestReference_mut_b(b);
}

procedure {:inline 1} TestReference_mut_ref () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var b: Value; // IntegerType()
    var b_ref: Reference; // ReferenceType(IntegerType())
    var __t2: Value; // IntegerType()
    var __t3: Reference; // ReferenceType(IntegerType())
    var __t4: Reference; // ReferenceType(IntegerType())
    var __t5: Reference; // ReferenceType(IntegerType())
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // BooleanType()
    var __t10: Value; // BooleanType()
    var __t11: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 12;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdConst(20);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 1, 0, 249, __tmp);

    call __t3 := BorrowLoc(__frame + 0);

    call b_ref := CopyOrMoveRef(__t3);
    assume IsValidU64(Dereference(__m, b_ref));
    assume $DebugTrackLocal(0, 1, 1, 265, Dereference(__m, b_ref));

    call __t4 := CopyOrMoveRef(b_ref);

    call TestReference_mut_b(__t4);
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 1, 346);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(0, 1, 0, 346, GetLocal(__m, __frame + 0));

    call __t5 := CopyOrMoveRef(b_ref);

    call __tmp := ReadRef(__t5);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 1, 0, 379, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8)));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __tmp := GetLocal(__m, __frame + 10);
    if (!b#Boolean(__tmp)) { goto Label_16; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    goto Label_Abort;

Label_16:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestReference_mut_ref_verify () returns ()
{
    call InitVerification();
    call TestReference_mut_ref();
}
