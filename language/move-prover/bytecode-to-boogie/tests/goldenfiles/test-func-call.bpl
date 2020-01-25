

// ** structs of module TestFuncCall



// ** functions of module TestFuncCall

procedure {:inline 1} TestFuncCall_f (x: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(x);

    old_size := local_counter;
    local_counter := local_counter + 4;
    m := UpdateLocal(m, old_size + 0, x);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := AddU64(GetLocal(m, old_size + 1), GetLocal(m, old_size + 2));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestFuncCall_f_verify (x: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestFuncCall_f(x);
}

procedure {:inline 1} TestFuncCall_g (x: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(x);

    old_size := local_counter;
    local_counter := local_counter + 4;
    m := UpdateLocal(m, old_size + 0, x);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := LdConst(2);
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := AddU64(GetLocal(m, old_size + 1), GetLocal(m, old_size + 2));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestFuncCall_g_verify (x: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestFuncCall_g(x);
}

procedure {:inline 1} TestFuncCall_h (b: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> abort_flag;
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // BooleanType()
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // BooleanType()
    var t10: Value; // IntegerType()
    var t11: Value; // IntegerType()
    var t12: Value; // BooleanType()
    var t13: Value; // BooleanType()
    var t14: Value; // BooleanType()
    var t15: Value; // BooleanType()
    var t16: Value; // IntegerType()
    var t17: Value; // IntegerType()
    var t18: Value; // BooleanType()
    var t19: Value; // BooleanType()
    var t20: Value; // BooleanType()
    var t21: Value; // BooleanType()
    var t22: Value; // IntegerType()
    var t23: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Boolean(b);

    old_size := local_counter;
    local_counter := local_counter + 24;
    m := UpdateLocal(m, old_size + 0, b);

    // bytecode translation starts here
    call tmp := LdConst(3);
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    tmp := GetLocal(m, old_size + 4);
    if (!b#Boolean(tmp)) { goto Label_8; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    call t6 := TestFuncCall_f(GetLocal(m, old_size + 5));
    if (abort_flag) { goto Label_Abort; }
    assume IsValidU64(t6);

    m := UpdateLocal(m, old_size + 6, t6);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 2, tmp);

    goto Label_11;

Label_8:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    call t8 := TestFuncCall_g(GetLocal(m, old_size + 7));
    if (abort_flag) { goto Label_Abort; }
    assume IsValidU64(t8);

    m := UpdateLocal(m, old_size + 8, t8);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 2, tmp);

Label_11:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 10, tmp);

    call tmp := LdConst(4);
    m := UpdateLocal(m, old_size + 11, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 10), GetLocal(m, old_size + 11)));
    m := UpdateLocal(m, old_size + 12, tmp);

    call tmp := And(GetLocal(m, old_size + 9), GetLocal(m, old_size + 12));
    m := UpdateLocal(m, old_size + 13, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 14, tmp);

    call tmp := Not(GetLocal(m, old_size + 14));
    m := UpdateLocal(m, old_size + 15, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 16, tmp);

    call tmp := LdConst(5);
    m := UpdateLocal(m, old_size + 17, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 16), GetLocal(m, old_size + 17)));
    m := UpdateLocal(m, old_size + 18, tmp);

    call tmp := And(GetLocal(m, old_size + 15), GetLocal(m, old_size + 18));
    m := UpdateLocal(m, old_size + 19, tmp);

    call tmp := Or(GetLocal(m, old_size + 13), GetLocal(m, old_size + 19));
    m := UpdateLocal(m, old_size + 20, tmp);

    call tmp := Not(GetLocal(m, old_size + 20));
    m := UpdateLocal(m, old_size + 21, tmp);

    tmp := GetLocal(m, old_size + 21);
    if (!b#Boolean(tmp)) { goto Label_27; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 22, tmp);

    goto Label_Abort;

Label_27:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 23, tmp);

    ret0 := GetLocal(m, old_size + 23);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestFuncCall_h_verify (b: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestFuncCall_h(b);
}
