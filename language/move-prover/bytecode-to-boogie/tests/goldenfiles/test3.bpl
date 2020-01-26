

// ** structs of module Test3

const unique Test3_T: TypeName;
const Test3_T_f: FieldName;
axiom Test3_T_f == 0;
const Test3_T_g: FieldName;
axiom Test3_T_g == 1;
function Test3_T_type_value(): TypeValue {
    StructType(Test3_T, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), IntegerType()))
}
procedure {:inline 1} Pack_Test3_T(f: Value, g: Value) returns (_struct: Value)
{
    assume IsValidU64(f);
    assume IsValidU64(g);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, f), g));
}

procedure {:inline 1} Unpack_Test3_T(_struct: Value) returns (f: Value, g: Value)
{
    assume is#Vector(_struct);
    f := SelectField(_struct, Test3_T_f);
    assume IsValidU64(f);
    g := SelectField(_struct, Test3_T_g);
    assume IsValidU64(g);
}



// ** functions of module Test3

procedure {:inline 1} Test3_test3 (flag: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var t1: Value; // Test3_T_type_value()
    var t2: Reference; // ReferenceType(Test3_T_type_value())
    var t3: Reference; // ReferenceType(IntegerType())
    var t4: Reference; // ReferenceType(IntegerType())
    var t5: Reference; // ReferenceType(IntegerType())
    var t6: Reference; // ReferenceType(IntegerType())
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // IntegerType()
    var t11: Value; // Test3_T_type_value()
    var t12: Reference; // ReferenceType(Test3_T_type_value())
    var t13: Value; // BooleanType()
    var t14: Reference; // ReferenceType(Test3_T_type_value())
    var t15: Reference; // ReferenceType(IntegerType())
    var t16: Reference; // ReferenceType(Test3_T_type_value())
    var t17: Reference; // ReferenceType(IntegerType())
    var t18: Value; // IntegerType()
    var t19: Reference; // ReferenceType(IntegerType())
    var t20: Value; // BooleanType()
    var t21: Value; // BooleanType()
    var t22: Reference; // ReferenceType(Test3_T_type_value())
    var t23: Reference; // ReferenceType(IntegerType())
    var t24: Reference; // ReferenceType(Test3_T_type_value())
    var t25: Reference; // ReferenceType(IntegerType())
    var t26: Value; // IntegerType()
    var t27: Reference; // ReferenceType(IntegerType())
    var t28: Reference; // ReferenceType(Test3_T_type_value())
    var t29: Reference; // ReferenceType(IntegerType())
    var t30: Reference; // ReferenceType(Test3_T_type_value())
    var t31: Reference; // ReferenceType(IntegerType())
    var t32: Reference; // ReferenceType(IntegerType())
    var t33: Value; // IntegerType()
    var t34: Reference; // ReferenceType(IntegerType())
    var t35: Value; // IntegerType()
    var t36: Value; // BooleanType()
    var t37: Value; // IntegerType()
    var t38: Value; // IntegerType()
    var t39: Value; // BooleanType()
    var t40: Value; // BooleanType()
    var t41: Value; // IntegerType()
    var t42: Value; // IntegerType()
    var t43: Value; // IntegerType()
    var t44: Value; // BooleanType()
    var t45: Value; // BooleanType()
    var t46: Value; // IntegerType()
    var t47: Value; // IntegerType()
    var t48: Value; // IntegerType()
    var t49: Value; // BooleanType()
    var t50: Value; // BooleanType()
    var t51: Value; // IntegerType()
    var t52: Value; // IntegerType()
    var t53: Value; // IntegerType()
    var t54: Value; // BooleanType()
    var t55: Value; // BooleanType()
    var t56: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 57;

    // process and type check arguments
    assume is#Boolean(flag);
    __m := UpdateLocal(__m, __frame + 0, flag);

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := Pack_Test3_T(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call t12 := BorrowLoc(__frame + 1);

    call t2 := CopyOrMoveRef(t12);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    __tmp := GetLocal(__m, __frame + 13);
    if (!b#Boolean(__tmp)) { goto Label_12; }

    call t14 := CopyOrMoveRef(t2);

    call t15 := BorrowField(t14, Test3_T_f);

    call t3 := CopyOrMoveRef(t15);

    goto Label_15;

Label_12:
    call t16 := CopyOrMoveRef(t2);

    call t17 := BorrowField(t16, Test3_T_g);

    call t3 := CopyOrMoveRef(t17);

Label_15:
    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call t19 := CopyOrMoveRef(t3);

    call WriteRef(t19, GetLocal(__m, __frame + 18));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    __tmp := GetLocal(__m, __frame + 21);
    if (!b#Boolean(__tmp)) { goto Label_25; }

    call t22 := CopyOrMoveRef(t2);

    call t23 := BorrowField(t22, Test3_T_f);

    call t4 := CopyOrMoveRef(t23);

    goto Label_28;

Label_25:
    call t24 := CopyOrMoveRef(t2);

    call t25 := BorrowField(t24, Test3_T_g);

    call t4 := CopyOrMoveRef(t25);

Label_28:
    call __tmp := LdConst(20);
    __m := UpdateLocal(__m, __frame + 26, __tmp);

    call t27 := CopyOrMoveRef(t4);

    call WriteRef(t27, GetLocal(__m, __frame + 26));

    call t28 := CopyOrMoveRef(t2);

    call t29 := BorrowField(t28, Test3_T_f);

    call t5 := CopyOrMoveRef(t29);

    call t30 := CopyOrMoveRef(t2);

    call t31 := BorrowField(t30, Test3_T_g);

    call t6 := CopyOrMoveRef(t31);

    call t32 := CopyOrMoveRef(t5);

    call __tmp := ReadRef(t32);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 33, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 33));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call t34 := CopyOrMoveRef(t6);

    call __tmp := ReadRef(t34);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 35, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 35));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 36, __tmp);

    __tmp := GetLocal(__m, __frame + 36);
    if (!b#Boolean(__tmp)) { goto Label_60; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 37, __tmp);

    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 38, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 37), GetLocal(__m, __frame + 38)));
    __m := UpdateLocal(__m, __frame + 39, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 39));
    __m := UpdateLocal(__m, __frame + 40, __tmp);

    __tmp := GetLocal(__m, __frame + 40);
    if (!b#Boolean(__tmp)) { goto Label_52; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 41, __tmp);

    goto Label_Abort;

Label_52:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 42, __tmp);

    call __tmp := LdConst(20);
    __m := UpdateLocal(__m, __frame + 43, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 42), GetLocal(__m, __frame + 43)));
    __m := UpdateLocal(__m, __frame + 44, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 44));
    __m := UpdateLocal(__m, __frame + 45, __tmp);

    __tmp := GetLocal(__m, __frame + 45);
    if (!b#Boolean(__tmp)) { goto Label_59; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 46, __tmp);

    goto Label_Abort;

Label_59:
    goto Label_74;

Label_60:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 47, __tmp);

    call __tmp := LdConst(20);
    __m := UpdateLocal(__m, __frame + 48, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 47), GetLocal(__m, __frame + 48)));
    __m := UpdateLocal(__m, __frame + 49, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 49));
    __m := UpdateLocal(__m, __frame + 50, __tmp);

    __tmp := GetLocal(__m, __frame + 50);
    if (!b#Boolean(__tmp)) { goto Label_67; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 51, __tmp);

    goto Label_Abort;

Label_67:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 52, __tmp);

    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 53, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 52), GetLocal(__m, __frame + 53)));
    __m := UpdateLocal(__m, __frame + 54, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 54));
    __m := UpdateLocal(__m, __frame + 55, __tmp);

    __tmp := GetLocal(__m, __frame + 55);
    if (!b#Boolean(__tmp)) { goto Label_74; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 56, __tmp);

    goto Label_Abort;

Label_74:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure Test3_test3_verify (flag: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call Test3_test3(flag);
}
