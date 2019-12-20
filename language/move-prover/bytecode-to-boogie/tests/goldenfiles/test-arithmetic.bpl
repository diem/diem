

// ** structs of module TestArithmetic



// ** functions of module TestArithmetic

procedure {:inline 1} TestArithmetic_add_two_number (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#Integer(arg1);

    old_size := local_counter;
    local_counter := local_counter + 10;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Add(GetLocal(m, old_size + 4), GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 6, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 2, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(3);
    m := UpdateLocal(m, old_size + 7, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 3, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 8, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 9, tmp);
    if (abort_flag) { goto Label_Abort; }

    ret0 := GetLocal(m, old_size + 8);
    ret1 := GetLocal(m, old_size + 9);
    return;
    if (abort_flag) { goto Label_Abort; }

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
}

procedure TestArithmetic_add_two_number_verify (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
{
    call ret0, ret1 := TestArithmetic_add_two_number(arg0, arg1);
}

procedure {:inline 1} TestArithmetic_multiple_ops (arg0: Value, arg1: Value, arg2: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#Integer(arg1);
    assume is#Integer(arg2);

    old_size := local_counter;
    local_counter := local_counter + 10;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);
    m := UpdateLocal(m, old_size + 2, arg2);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Add(GetLocal(m, old_size + 4), GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 6, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 7, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Mul(GetLocal(m, old_size + 6), GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 8, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 3, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 9, tmp);
    if (abort_flag) { goto Label_Abort; }

    ret0 := GetLocal(m, old_size + 9);
    return;
    if (abort_flag) { goto Label_Abort; }

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestArithmetic_multiple_ops_verify (arg0: Value, arg1: Value, arg2: Value) returns (ret0: Value)
{
    call ret0 := TestArithmetic_multiple_ops(arg0, arg1, arg2);
}

procedure {:inline 1} TestArithmetic_bool_ops (arg0: Value, arg1: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // BooleanType()
    var t3: Value; // BooleanType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // BooleanType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // BooleanType()
    var t10: Value; // BooleanType()
    var t11: Value; // IntegerType()
    var t12: Value; // IntegerType()
    var t13: Value; // BooleanType()
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Value; // BooleanType()
    var t17: Value; // BooleanType()
    var t18: Value; // BooleanType()
    var t19: Value; // BooleanType()
    var t20: Value; // BooleanType()
    var t21: Value; // BooleanType()
    var t22: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#Integer(arg1);

    old_size := local_counter;
    local_counter := local_counter + 23;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Gt(GetLocal(m, old_size + 4), GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 6, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 7, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 8, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Ge(GetLocal(m, old_size + 7), GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 9, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := And(GetLocal(m, old_size + 6), GetLocal(m, old_size + 9));
    m := UpdateLocal(m, old_size + 10, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 10));
    m := UpdateLocal(m, old_size + 2, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 11, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 12, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Lt(GetLocal(m, old_size + 11), GetLocal(m, old_size + 12));
    m := UpdateLocal(m, old_size + 13, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 14, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 15, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Le(GetLocal(m, old_size + 14), GetLocal(m, old_size + 15));
    m := UpdateLocal(m, old_size + 16, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Or(GetLocal(m, old_size + 13), GetLocal(m, old_size + 16));
    m := UpdateLocal(m, old_size + 17, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 17));
    m := UpdateLocal(m, old_size + 3, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 18, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 19, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := Boolean(!IsEqual(GetLocal(m, old_size + 18), GetLocal(m, old_size + 19)));
    m := UpdateLocal(m, old_size + 20, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Not(GetLocal(m, old_size + 20));
    m := UpdateLocal(m, old_size + 21, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := GetLocal(m, old_size + 21);
    if (!b#Boolean(tmp)) { goto Label_23; }
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 22, tmp);
    if (abort_flag) { goto Label_Abort; }

    goto Label_Abort;
    if (abort_flag) { goto Label_Abort; }

Label_23:
    return;
    if (abort_flag) { goto Label_Abort; }

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestArithmetic_bool_ops_verify (arg0: Value, arg1: Value) returns ()
{
    call TestArithmetic_bool_ops(arg0, arg1);
}

procedure {:inline 1} TestArithmetic_arithmetic_ops (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // IntegerType()
    var t11: Value; // IntegerType()
    var t12: Value; // IntegerType()
    var t13: Value; // IntegerType()
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Value; // BooleanType()
    var t17: Value; // BooleanType()
    var t18: Value; // IntegerType()
    var t19: Value; // IntegerType()
    var t20: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#Integer(arg1);

    old_size := local_counter;
    local_counter := local_counter + 21;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call tmp := LdConst(6);
    m := UpdateLocal(m, old_size + 3, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(4);
    m := UpdateLocal(m, old_size + 4, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Add(GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 6, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Sub(GetLocal(m, old_size + 5), GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 7, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(2);
    m := UpdateLocal(m, old_size + 8, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Mul(GetLocal(m, old_size + 7), GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 9, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(3);
    m := UpdateLocal(m, old_size + 10, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Div(GetLocal(m, old_size + 9), GetLocal(m, old_size + 10));
    m := UpdateLocal(m, old_size + 11, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(4);
    m := UpdateLocal(m, old_size + 12, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Mod(GetLocal(m, old_size + 11), GetLocal(m, old_size + 12));
    m := UpdateLocal(m, old_size + 13, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 13));
    m := UpdateLocal(m, old_size + 2, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 14, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(2);
    m := UpdateLocal(m, old_size + 15, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 14), GetLocal(m, old_size + 15)));
    m := UpdateLocal(m, old_size + 16, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Not(GetLocal(m, old_size + 16));
    m := UpdateLocal(m, old_size + 17, tmp);
    if (abort_flag) { goto Label_Abort; }

    tmp := GetLocal(m, old_size + 17);
    if (!b#Boolean(tmp)) { goto Label_19; }
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 18, tmp);
    if (abort_flag) { goto Label_Abort; }

    goto Label_Abort;
    if (abort_flag) { goto Label_Abort; }

Label_19:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 19, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 20, tmp);
    if (abort_flag) { goto Label_Abort; }

    ret0 := GetLocal(m, old_size + 19);
    ret1 := GetLocal(m, old_size + 20);
    return;
    if (abort_flag) { goto Label_Abort; }

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
}

procedure TestArithmetic_arithmetic_ops_verify (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
{
    call ret0, ret1 := TestArithmetic_arithmetic_ops(arg0, arg1);
}

procedure {:inline 1} TestArithmetic_overflow () returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 6;

    // bytecode translation starts here
    call tmp := LdConst(9223372036854775807);
    m := UpdateLocal(m, old_size + 2, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 0, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 4, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Add(GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 1, tmp);
    if (abort_flag) { goto Label_Abort; }

    return;
    if (abort_flag) { goto Label_Abort; }

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestArithmetic_overflow_verify () returns ()
{
    call TestArithmetic_overflow();
}

procedure {:inline 1} TestArithmetic_underflow () returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 6;

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 2, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 0, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 4, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Sub(GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 1, tmp);
    if (abort_flag) { goto Label_Abort; }

    return;
    if (abort_flag) { goto Label_Abort; }

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestArithmetic_underflow_verify () returns ()
{
    call TestArithmetic_underflow();
}

procedure {:inline 1} TestArithmetic_div_by_zero () returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 6;

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 2, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 0, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 3, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := Div(GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 1, tmp);
    if (abort_flag) { goto Label_Abort; }

    return;
    if (abort_flag) { goto Label_Abort; }

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestArithmetic_div_by_zero_verify () returns ()
{
    call TestArithmetic_div_by_zero();
}
