

// ** structs of module TestAbortIf



// ** functions of module TestAbortIf

procedure {:inline 1} TestAbortIf_abort1 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y)))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // BooleanType()
    var t5: Value; // BooleanType()
    var t6: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(x);
    assume IsValidU64(y);

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := Gt(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Not(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);

    tmp := GetLocal(m, old_size + 5);
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 6, tmp);

    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestAbortIf_abort1_verify (x: Value, y: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestAbortIf_abort1(x, y);
}

procedure {:inline 1} TestAbortIf_abort2 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y)))) ==> abort_flag;
{
    // declare local variables

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(x);
    assume IsValidU64(y);

    old_size := local_counter;
    local_counter := local_counter + 2;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestAbortIf_abort2_verify (x: Value, y: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestAbortIf_abort2(x, y);
}

procedure {:inline 1} TestAbortIf_abort3 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y)))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // BooleanType()
    var t3: Value; // BooleanType()
    var t4: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(x);
    assume IsValidU64(y);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    call tmp := LdFalse();
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := Not(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 3, tmp);

    tmp := GetLocal(m, old_size + 3);
    if (!b#Boolean(tmp)) { goto Label_5; }

    call tmp := LdConst(2);
    m := UpdateLocal(m, old_size + 4, tmp);

    goto Label_Abort;

Label_5:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestAbortIf_abort3_verify (x: Value, y: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestAbortIf_abort3(x, y);
}

procedure {:inline 1} TestAbortIf_abort4 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) < i#Integer(y))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // BooleanType()
    var t5: Value; // BooleanType()
    var t6: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(x);
    assume IsValidU64(y);

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := Gt(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Not(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);

    tmp := GetLocal(m, old_size + 5);
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 6, tmp);

    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestAbortIf_abort4_verify (x: Value, y: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestAbortIf_abort4(x, y);
}

procedure {:inline 1} TestAbortIf_abort5 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y)))) && (b#Boolean(Boolean(i#Integer(x) > i#Integer(y))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y)))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // BooleanType()
    var t5: Value; // BooleanType()
    var t6: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(x);
    assume IsValidU64(y);

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := Gt(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Not(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);

    tmp := GetLocal(m, old_size + 5);
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 6, tmp);

    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestAbortIf_abort5_verify (x: Value, y: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestAbortIf_abort5(x, y);
}

procedure {:inline 1} TestAbortIf_abort6 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) && (b#Boolean(Boolean(i#Integer(x) > i#Integer(y))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // BooleanType()
    var t5: Value; // BooleanType()
    var t6: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(x);
    assume IsValidU64(y);

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := Gt(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Not(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);

    tmp := GetLocal(m, old_size + 5);
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 6, tmp);

    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestAbortIf_abort6_verify (x: Value, y: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestAbortIf_abort6(x, y);
}

procedure {:inline 1} TestAbortIf_abort7 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) && (b#Boolean(Boolean(i#Integer(x) >= i#Integer(y))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // BooleanType()
    var t5: Value; // BooleanType()
    var t6: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(x);
    assume IsValidU64(y);

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := Gt(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Not(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);

    tmp := GetLocal(m, old_size + 5);
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 6, tmp);

    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestAbortIf_abort7_verify (x: Value, y: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestAbortIf_abort7(x, y);
}

procedure {:inline 1} TestAbortIf_abort8 (x: Value, y: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((ret0) == (Boolean(true))));
ensures old(!(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) && (b#Boolean(Boolean(i#Integer(x) > i#Integer(y))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // BooleanType()
    var t5: Value; // BooleanType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(x);
    assume IsValidU64(y);

    old_size := local_counter;
    local_counter := local_counter + 11;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := Gt(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Not(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);

    tmp := GetLocal(m, old_size + 5);
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 6, tmp);

    goto Label_Abort;

Label_7:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 9, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 8), GetLocal(m, old_size + 9)));
    m := UpdateLocal(m, old_size + 10, tmp);

    ret0 := GetLocal(m, old_size + 10);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestAbortIf_abort8_verify (x: Value, y: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestAbortIf_abort8(x, y);
}

procedure {:inline 1} TestAbortIf_abort9 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((x) == (y)));
ensures old(!(b#Boolean(Boolean(i#Integer(x) > i#Integer(y))) || b#Boolean(Boolean(i#Integer(x) < i#Integer(y))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) > i#Integer(y))) || b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // BooleanType()
    var t5: Value; // BooleanType()
    var t6: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(x);
    assume IsValidU64(y);

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := Gt(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Not(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);

    tmp := GetLocal(m, old_size + 5);
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 6, tmp);

    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestAbortIf_abort9_verify (x: Value, y: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call TestAbortIf_abort9(x, y);
}
