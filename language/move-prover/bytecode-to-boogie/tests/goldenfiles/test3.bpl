

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
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> abort_flag;
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Boolean(flag);

    old_size := local_counter;
    local_counter := local_counter + 57;
    m := UpdateLocal(m, old_size + 0, flag);

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 10, tmp);

    call tmp := Pack_Test3_T(GetLocal(m, old_size + 9), GetLocal(m, old_size + 10));
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 11));
    m := UpdateLocal(m, old_size + 1, tmp);

    call t12 := BorrowLoc(old_size+1);

    call t2 := CopyOrMoveRef(t12);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 13, tmp);

    tmp := GetLocal(m, old_size + 13);
    if (!b#Boolean(tmp)) { goto Label_12; }

    call t14 := CopyOrMoveRef(t2);

    call t15 := BorrowField(t14, Test3_T_f);

    call t3 := CopyOrMoveRef(t15);

    goto Label_15;

Label_12:
    call t16 := CopyOrMoveRef(t2);

    call t17 := BorrowField(t16, Test3_T_g);

    call t3 := CopyOrMoveRef(t17);

Label_15:
    call tmp := LdConst(10);
    m := UpdateLocal(m, old_size + 18, tmp);

    call t19 := CopyOrMoveRef(t3);

    call WriteRef(t19, GetLocal(m, old_size + 18));

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 20, tmp);

    call tmp := Not(GetLocal(m, old_size + 20));
    m := UpdateLocal(m, old_size + 21, tmp);

    tmp := GetLocal(m, old_size + 21);
    if (!b#Boolean(tmp)) { goto Label_25; }

    call t22 := CopyOrMoveRef(t2);

    call t23 := BorrowField(t22, Test3_T_f);

    call t4 := CopyOrMoveRef(t23);

    goto Label_28;

Label_25:
    call t24 := CopyOrMoveRef(t2);

    call t25 := BorrowField(t24, Test3_T_g);

    call t4 := CopyOrMoveRef(t25);

Label_28:
    call tmp := LdConst(20);
    m := UpdateLocal(m, old_size + 26, tmp);

    call t27 := CopyOrMoveRef(t4);

    call WriteRef(t27, GetLocal(m, old_size + 26));

    call t28 := CopyOrMoveRef(t2);

    call t29 := BorrowField(t28, Test3_T_f);

    call t5 := CopyOrMoveRef(t29);

    call t30 := CopyOrMoveRef(t2);

    call t31 := BorrowField(t30, Test3_T_g);

    call t6 := CopyOrMoveRef(t31);

    call t32 := CopyOrMoveRef(t5);

    call tmp := ReadRef(t32);
    assume IsValidU64(tmp);

    m := UpdateLocal(m, old_size + 33, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 33));
    m := UpdateLocal(m, old_size + 7, tmp);

    call t34 := CopyOrMoveRef(t6);

    call tmp := ReadRef(t34);
    assume IsValidU64(tmp);

    m := UpdateLocal(m, old_size + 35, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 35));
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 36, tmp);

    tmp := GetLocal(m, old_size + 36);
    if (!b#Boolean(tmp)) { goto Label_60; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 37, tmp);

    call tmp := LdConst(10);
    m := UpdateLocal(m, old_size + 38, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 37), GetLocal(m, old_size + 38)));
    m := UpdateLocal(m, old_size + 39, tmp);

    call tmp := Not(GetLocal(m, old_size + 39));
    m := UpdateLocal(m, old_size + 40, tmp);

    tmp := GetLocal(m, old_size + 40);
    if (!b#Boolean(tmp)) { goto Label_52; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 41, tmp);

    goto Label_Abort;

Label_52:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 42, tmp);

    call tmp := LdConst(20);
    m := UpdateLocal(m, old_size + 43, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 42), GetLocal(m, old_size + 43)));
    m := UpdateLocal(m, old_size + 44, tmp);

    call tmp := Not(GetLocal(m, old_size + 44));
    m := UpdateLocal(m, old_size + 45, tmp);

    tmp := GetLocal(m, old_size + 45);
    if (!b#Boolean(tmp)) { goto Label_59; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 46, tmp);

    goto Label_Abort;

Label_59:
    goto Label_74;

Label_60:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 47, tmp);

    call tmp := LdConst(20);
    m := UpdateLocal(m, old_size + 48, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 47), GetLocal(m, old_size + 48)));
    m := UpdateLocal(m, old_size + 49, tmp);

    call tmp := Not(GetLocal(m, old_size + 49));
    m := UpdateLocal(m, old_size + 50, tmp);

    tmp := GetLocal(m, old_size + 50);
    if (!b#Boolean(tmp)) { goto Label_67; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 51, tmp);

    goto Label_Abort;

Label_67:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 52, tmp);

    call tmp := LdConst(10);
    m := UpdateLocal(m, old_size + 53, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 52), GetLocal(m, old_size + 53)));
    m := UpdateLocal(m, old_size + 54, tmp);

    call tmp := Not(GetLocal(m, old_size + 54));
    m := UpdateLocal(m, old_size + 55, tmp);

    tmp := GetLocal(m, old_size + 55);
    if (!b#Boolean(tmp)) { goto Label_74; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 56, tmp);

    goto Label_Abort;

Label_74:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure Test3_test3_verify (flag: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call Test3_test3(flag);
}
