

// ** structs of module Test3

const unique Test3_T: TypeName;
const Test3_T_f: FieldName;
axiom Test3_T_f == 0;
const Test3_T_g: FieldName;
axiom Test3_T_g == 1;
function Test3_T_type_value(): TypeValue {
    StructType(Test3_T, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), IntegerType()))
}

procedure {:inline 1} Pack_Test3_T(v0: Value, v1: Value) returns (v: Value)
{
    assume is#Integer(v0);
    assume is#Integer(v1);
    v := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, v0), v1));

}

procedure {:inline 1} Unpack_Test3_T(v: Value) returns (v0: Value, v1: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, Test3_T_f);
    v1 := SelectField(v, Test3_T_g);
}



// ** functions of module Test3

procedure {:inline 1} Test3_test3 (arg0: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // BooleanType()
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Boolean(arg0);

    old_size := local_counter;
    local_counter := local_counter + 57;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 9, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 10, tmp);
    if (abort_flag) { goto Label_Abort; }

    assume is#Integer(GetLocal(m, old_size + 9));

    assume is#Integer(GetLocal(m, old_size + 10));

    call tmp := Pack_Test3_T(GetLocal(m, old_size + 9), GetLocal(m, old_size + 10));
    m := UpdateLocal(m, old_size + 11, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 11));
    m := UpdateLocal(m, old_size + 1, tmp);
    if (abort_flag) { goto Label_Abort; }

    call t12 := BorrowLoc(old_size+1);
    if (abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t12);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 13, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := GetLocal(m, old_size + 13);
    if (!b#Boolean(tmp)) { goto Label_12; }
    if (abort_flag) { goto Label_Abort; }

    call t14 := CopyOrMoveRef(t2);
    if (abort_flag) { goto Label_Abort; }

    call t15 := BorrowField(t14, Test3_T_f);
    if (abort_flag) { goto Label_Abort; }

    call t3 := CopyOrMoveRef(t15);
    if (abort_flag) { goto Label_Abort; }

    goto Label_15;
    if (abort_flag) { goto Label_Abort; }

Label_12:
    call t16 := CopyOrMoveRef(t2);
    if (abort_flag) { goto Label_Abort; }

    call t17 := BorrowField(t16, Test3_T_g);
    if (abort_flag) { goto Label_Abort; }

    call t3 := CopyOrMoveRef(t17);
    if (abort_flag) { goto Label_Abort; }

Label_15:
    call tmp := LdConst(10);
    m := UpdateLocal(m, old_size + 18, tmp);
    if (abort_flag) { goto Label_Abort; }

    call t19 := CopyOrMoveRef(t3);
    if (abort_flag) { goto Label_Abort; }

    call WriteRef(t19, GetLocal(m, old_size + 18));
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 20, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Not(GetLocal(m, old_size + 20));
    m := UpdateLocal(m, old_size + 21, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := GetLocal(m, old_size + 21);
    if (!b#Boolean(tmp)) { goto Label_25; }
    if (abort_flag) { goto Label_Abort; }

    call t22 := CopyOrMoveRef(t2);
    if (abort_flag) { goto Label_Abort; }

    call t23 := BorrowField(t22, Test3_T_f);
    if (abort_flag) { goto Label_Abort; }

    call t4 := CopyOrMoveRef(t23);
    if (abort_flag) { goto Label_Abort; }

    goto Label_28;
    if (abort_flag) { goto Label_Abort; }

Label_25:
    call t24 := CopyOrMoveRef(t2);
    if (abort_flag) { goto Label_Abort; }

    call t25 := BorrowField(t24, Test3_T_g);
    if (abort_flag) { goto Label_Abort; }

    call t4 := CopyOrMoveRef(t25);
    if (abort_flag) { goto Label_Abort; }

Label_28:
    call tmp := LdConst(20);
    m := UpdateLocal(m, old_size + 26, tmp);
    if (abort_flag) { goto Label_Abort; }

    call t27 := CopyOrMoveRef(t4);
    if (abort_flag) { goto Label_Abort; }

    call WriteRef(t27, GetLocal(m, old_size + 26));
    if (abort_flag) { goto Label_Abort; }

    call t28 := CopyOrMoveRef(t2);
    if (abort_flag) { goto Label_Abort; }

    call t29 := BorrowField(t28, Test3_T_f);
    if (abort_flag) { goto Label_Abort; }

    call t5 := CopyOrMoveRef(t29);
    if (abort_flag) { goto Label_Abort; }

    call t30 := CopyOrMoveRef(t2);
    if (abort_flag) { goto Label_Abort; }

    call t31 := BorrowField(t30, Test3_T_g);
    if (abort_flag) { goto Label_Abort; }

    call t6 := CopyOrMoveRef(t31);
    if (abort_flag) { goto Label_Abort; }

    call t32 := CopyOrMoveRef(t5);
    if (abort_flag) { goto Label_Abort; }

    call tmp := ReadRef(t32);
    assume is#Integer(tmp);

    m := UpdateLocal(m, old_size + 33, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 33));
    m := UpdateLocal(m, old_size + 7, tmp);
    if (abort_flag) { goto Label_Abort; }

    call t34 := CopyOrMoveRef(t6);
    if (abort_flag) { goto Label_Abort; }

    call tmp := ReadRef(t34);
    assume is#Integer(tmp);

    m := UpdateLocal(m, old_size + 35, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 35));
    m := UpdateLocal(m, old_size + 8, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 36, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := GetLocal(m, old_size + 36);
    if (!b#Boolean(tmp)) { goto Label_60; }
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 37, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(10);
    m := UpdateLocal(m, old_size + 38, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 37), GetLocal(m, old_size + 38)));
    m := UpdateLocal(m, old_size + 39, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Not(GetLocal(m, old_size + 39));
    m := UpdateLocal(m, old_size + 40, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := GetLocal(m, old_size + 40);
    if (!b#Boolean(tmp)) { goto Label_52; }
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 41, tmp);
    if (abort_flag) { goto Label_Abort; }

    goto Label_Abort;
    if (abort_flag) { goto Label_Abort; }

Label_52:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 42, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(20);
    m := UpdateLocal(m, old_size + 43, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 42), GetLocal(m, old_size + 43)));
    m := UpdateLocal(m, old_size + 44, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Not(GetLocal(m, old_size + 44));
    m := UpdateLocal(m, old_size + 45, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := GetLocal(m, old_size + 45);
    if (!b#Boolean(tmp)) { goto Label_59; }
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 46, tmp);
    if (abort_flag) { goto Label_Abort; }

    goto Label_Abort;
    if (abort_flag) { goto Label_Abort; }

Label_59:
    goto Label_74;
    if (abort_flag) { goto Label_Abort; }

Label_60:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 47, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(20);
    m := UpdateLocal(m, old_size + 48, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 47), GetLocal(m, old_size + 48)));
    m := UpdateLocal(m, old_size + 49, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Not(GetLocal(m, old_size + 49));
    m := UpdateLocal(m, old_size + 50, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := GetLocal(m, old_size + 50);
    if (!b#Boolean(tmp)) { goto Label_67; }
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 51, tmp);
    if (abort_flag) { goto Label_Abort; }

    goto Label_Abort;
    if (abort_flag) { goto Label_Abort; }

Label_67:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 52, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(10);
    m := UpdateLocal(m, old_size + 53, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 52), GetLocal(m, old_size + 53)));
    m := UpdateLocal(m, old_size + 54, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Not(GetLocal(m, old_size + 54));
    m := UpdateLocal(m, old_size + 55, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := GetLocal(m, old_size + 55);
    if (!b#Boolean(tmp)) { goto Label_74; }
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 56, tmp);
    if (abort_flag) { goto Label_Abort; }

    goto Label_Abort;
    if (abort_flag) { goto Label_Abort; }

Label_74:
    return;
    if (abort_flag) { goto Label_Abort; }

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure Test3_test3_verify (arg0: Value) returns ()
{
    call Test3_test3(arg0);
}
