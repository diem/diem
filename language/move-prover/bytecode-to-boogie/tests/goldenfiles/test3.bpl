

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
    var x: Value; // Test3_T_type_value()
    var x_ref: Reference; // ReferenceType(Test3_T_type_value())
    var f_or_g_ref: Reference; // ReferenceType(IntegerType())
    var f_or_g_ref2: Reference; // ReferenceType(IntegerType())
    var f_ref: Reference; // ReferenceType(IntegerType())
    var g_ref: Reference; // ReferenceType(IntegerType())
    var f: Value; // IntegerType()
    var g: Value; // IntegerType()
    var __t9: Value; // IntegerType()
    var __t10: Value; // IntegerType()
    var __t11: Value; // Test3_T_type_value()
    var __t12: Reference; // ReferenceType(Test3_T_type_value())
    var __t13: Value; // BooleanType()
    var __t14: Reference; // ReferenceType(Test3_T_type_value())
    var __t15: Reference; // ReferenceType(IntegerType())
    var __t16: Reference; // ReferenceType(Test3_T_type_value())
    var __t17: Reference; // ReferenceType(IntegerType())
    var __t18: Value; // IntegerType()
    var __t19: Reference; // ReferenceType(IntegerType())
    var __t20: Value; // BooleanType()
    var __t21: Value; // BooleanType()
    var __t22: Reference; // ReferenceType(Test3_T_type_value())
    var __t23: Reference; // ReferenceType(IntegerType())
    var __t24: Reference; // ReferenceType(Test3_T_type_value())
    var __t25: Reference; // ReferenceType(IntegerType())
    var __t26: Value; // IntegerType()
    var __t27: Reference; // ReferenceType(IntegerType())
    var __t28: Reference; // ReferenceType(Test3_T_type_value())
    var __t29: Reference; // ReferenceType(IntegerType())
    var __t30: Reference; // ReferenceType(Test3_T_type_value())
    var __t31: Reference; // ReferenceType(IntegerType())
    var __t32: Reference; // ReferenceType(IntegerType())
    var __t33: Value; // IntegerType()
    var __t34: Reference; // ReferenceType(IntegerType())
    var __t35: Value; // IntegerType()
    var __t36: Value; // BooleanType()
    var __t37: Value; // IntegerType()
    var __t38: Value; // IntegerType()
    var __t39: Value; // BooleanType()
    var __t40: Value; // BooleanType()
    var __t41: Value; // IntegerType()
    var __t42: Value; // IntegerType()
    var __t43: Value; // IntegerType()
    var __t44: Value; // BooleanType()
    var __t45: Value; // BooleanType()
    var __t46: Value; // IntegerType()
    var __t47: Value; // IntegerType()
    var __t48: Value; // IntegerType()
    var __t49: Value; // BooleanType()
    var __t50: Value; // BooleanType()
    var __t51: Value; // IntegerType()
    var __t52: Value; // IntegerType()
    var __t53: Value; // IntegerType()
    var __t54: Value; // BooleanType()
    var __t55: Value; // BooleanType()
    var __t56: Value; // IntegerType()
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
    assume $DebugTrackLocal(0, 0, 0, 53, flag);

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := Pack_Test3_T(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(0, 0, 1, 267, __tmp);

    call __t12 := BorrowLoc(__frame + 1);

    call x_ref := CopyOrMoveRef(__t12);
    assume is#Vector(Dereference(__m, x_ref));
    assume $DebugTrackLocal(0, 0, 2, 290, Dereference(__m, x_ref));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    __tmp := GetLocal(__m, __frame + 13);
    if (!b#Boolean(__tmp)) { goto Label_12; }

    call __t14 := CopyOrMoveRef(x_ref);

    call __t15 := BorrowField(__t14, Test3_T_f);

    call f_or_g_ref := CopyOrMoveRef(__t15);
    assume IsValidU64(Dereference(__m, f_or_g_ref));
    assume $DebugTrackLocal(0, 0, 3, 330, Dereference(__m, f_or_g_ref));

    goto Label_15;

Label_12:
    call __t16 := CopyOrMoveRef(x_ref);

    call __t17 := BorrowField(__t16, Test3_T_g);

    call f_or_g_ref := CopyOrMoveRef(__t17);
    assume IsValidU64(Dereference(__m, f_or_g_ref));
    assume $DebugTrackLocal(0, 0, 3, 377, Dereference(__m, f_or_g_ref));

Label_15:
    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __t19 := CopyOrMoveRef(f_or_g_ref);

    call WriteRef(__t19, GetLocal(__m, __frame + 18));
    assume $DebugTrackLocal(0, 0, 1, 417, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    __tmp := GetLocal(__m, __frame + 21);
    if (!b#Boolean(__tmp)) { goto Label_25; }

    call __t22 := CopyOrMoveRef(x_ref);

    call __t23 := BorrowField(__t22, Test3_T_f);

    call f_or_g_ref2 := CopyOrMoveRef(__t23);
    assume IsValidU64(Dereference(__m, f_or_g_ref2));
    assume $DebugTrackLocal(0, 0, 4, 466, Dereference(__m, f_or_g_ref2));

    goto Label_28;

Label_25:
    call __t24 := CopyOrMoveRef(x_ref);

    call __t25 := BorrowField(__t24, Test3_T_g);

    call f_or_g_ref2 := CopyOrMoveRef(__t25);
    assume IsValidU64(Dereference(__m, f_or_g_ref2));
    assume $DebugTrackLocal(0, 0, 4, 514, Dereference(__m, f_or_g_ref2));

Label_28:
    call __tmp := LdConst(20);
    __m := UpdateLocal(__m, __frame + 26, __tmp);

    call __t27 := CopyOrMoveRef(f_or_g_ref2);

    call WriteRef(__t27, GetLocal(__m, __frame + 26));
    assume $DebugTrackLocal(0, 0, 1, 555, GetLocal(__m, __frame + 1));

    call __t28 := CopyOrMoveRef(x_ref);

    call __t29 := BorrowField(__t28, Test3_T_f);

    call f_ref := CopyOrMoveRef(__t29);
    assume IsValidU64(Dereference(__m, f_ref));
    assume $DebugTrackLocal(0, 0, 5, 583, Dereference(__m, f_ref));

    call __t30 := CopyOrMoveRef(x_ref);

    call __t31 := BorrowField(__t30, Test3_T_g);

    call g_ref := CopyOrMoveRef(__t31);
    assume IsValidU64(Dereference(__m, g_ref));
    assume $DebugTrackLocal(0, 0, 6, 609, Dereference(__m, g_ref));

    call __t32 := CopyOrMoveRef(f_ref);

    call __tmp := ReadRef(__t32);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 33, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 33));
    __m := UpdateLocal(__m, __frame + 7, __tmp);
    assume $DebugTrackLocal(0, 0, 7, 635, __tmp);

    call __t34 := CopyOrMoveRef(g_ref);

    call __tmp := ReadRef(__t34);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 35, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 35));
    __m := UpdateLocal(__m, __frame + 8, __tmp);
    assume $DebugTrackLocal(0, 0, 8, 655, __tmp);

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
    call InitVerification();
    call Test3_test3(flag);
}
