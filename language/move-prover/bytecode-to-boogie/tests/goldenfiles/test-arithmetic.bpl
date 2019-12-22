

// ** structs of module TestArithmetic



// ** functions of module TestArithmetic

procedure {:inline 1} TestArithmetic_add_two_number (x: Value, y: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
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
    assume IsValidInteger(x);
    assume IsValidInteger(y);

    old_size := local_counter;
    local_counter := local_counter + 10;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := Add(GetLocal(m, old_size + 4), GetLocal(m, old_size + 5));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := LdConst(3);
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 9, tmp);

    ret0 := GetLocal(m, old_size + 8);
    ret1 := GetLocal(m, old_size + 9);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
}

procedure TestArithmetic_add_two_number_verify (x: Value, y: Value) returns (ret0: Value, ret1: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0, ret1 := TestArithmetic_add_two_number(x, y);
}

procedure {:inline 1} TestArithmetic_multiple_ops (x: Value, y: Value, z: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
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
    assume IsValidInteger(x);
    assume IsValidInteger(y);
    assume IsValidInteger(z);

    old_size := local_counter;
    local_counter := local_counter + 10;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);
    m := UpdateLocal(m, old_size + 2, z);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := Add(GetLocal(m, old_size + 4), GetLocal(m, old_size + 5));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := Mul(GetLocal(m, old_size + 6), GetLocal(m, old_size + 7));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 9, tmp);

    ret0 := GetLocal(m, old_size + 9);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestArithmetic_multiple_ops_verify (x: Value, y: Value, z: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestArithmetic_multiple_ops(x, y, z);
}

procedure {:inline 1} TestArithmetic_bool_ops (a: Value, b: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
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
    assume IsValidInteger(a);
    assume IsValidInteger(b);

    old_size := local_counter;
    local_counter := local_counter + 23;
    m := UpdateLocal(m, old_size + 0, a);
    m := UpdateLocal(m, old_size + 1, b);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := Gt(GetLocal(m, old_size + 4), GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := Ge(GetLocal(m, old_size + 7), GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := And(GetLocal(m, old_size + 6), GetLocal(m, old_size + 9));
    m := UpdateLocal(m, old_size + 10, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 10));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 12, tmp);

    call tmp := Lt(GetLocal(m, old_size + 11), GetLocal(m, old_size + 12));
    m := UpdateLocal(m, old_size + 13, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 14, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 15, tmp);

    call tmp := Le(GetLocal(m, old_size + 14), GetLocal(m, old_size + 15));
    m := UpdateLocal(m, old_size + 16, tmp);

    call tmp := Or(GetLocal(m, old_size + 13), GetLocal(m, old_size + 16));
    m := UpdateLocal(m, old_size + 17, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 17));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 18, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 19, tmp);

    tmp := Boolean(!IsEqual(GetLocal(m, old_size + 18), GetLocal(m, old_size + 19)));
    m := UpdateLocal(m, old_size + 20, tmp);

    call tmp := Not(GetLocal(m, old_size + 20));
    m := UpdateLocal(m, old_size + 21, tmp);

    tmp := GetLocal(m, old_size + 21);
    if (!b#Boolean(tmp)) { goto Label_23; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 22, tmp);

    goto Label_Abort;

Label_23:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestArithmetic_bool_ops_verify (a: Value, b: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestArithmetic_bool_ops(a, b);
}

procedure {:inline 1} TestArithmetic_arithmetic_ops (a: Value, b: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
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
    assume IsValidInteger(a);
    assume IsValidInteger(b);

    old_size := local_counter;
    local_counter := local_counter + 21;
    m := UpdateLocal(m, old_size + 0, a);
    m := UpdateLocal(m, old_size + 1, b);

    // bytecode translation starts here
    call tmp := LdConst(6);
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := LdConst(4);
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Add(GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := Sub(GetLocal(m, old_size + 5), GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := LdConst(2);
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := Mul(GetLocal(m, old_size + 7), GetLocal(m, old_size + 8));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := LdConst(3);
    m := UpdateLocal(m, old_size + 10, tmp);

    call tmp := Div(GetLocal(m, old_size + 9), GetLocal(m, old_size + 10));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := LdConst(4);
    m := UpdateLocal(m, old_size + 12, tmp);

    call tmp := Mod(GetLocal(m, old_size + 11), GetLocal(m, old_size + 12));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 13, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 13));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 14, tmp);

    call tmp := LdConst(2);
    m := UpdateLocal(m, old_size + 15, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 14), GetLocal(m, old_size + 15)));
    m := UpdateLocal(m, old_size + 16, tmp);

    call tmp := Not(GetLocal(m, old_size + 16));
    m := UpdateLocal(m, old_size + 17, tmp);

    tmp := GetLocal(m, old_size + 17);
    if (!b#Boolean(tmp)) { goto Label_19; }

    call tmp := LdConst(42);
    m := UpdateLocal(m, old_size + 18, tmp);

    goto Label_Abort;

Label_19:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 19, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 20, tmp);

    ret0 := GetLocal(m, old_size + 19);
    ret1 := GetLocal(m, old_size + 20);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
}

procedure TestArithmetic_arithmetic_ops_verify (a: Value, b: Value) returns (ret0: Value, ret1: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0, ret1 := TestArithmetic_arithmetic_ops(a, b);
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

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Add(GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 1, tmp);

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestArithmetic_overflow_verify () returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
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

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Sub(GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 1, tmp);

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestArithmetic_underflow_verify () returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
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

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Div(GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 1, tmp);

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestArithmetic_div_by_zero_verify () returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestArithmetic_div_by_zero();
}
