

// ** structs of module TestAddition



// ** functions of module TestAddition

procedure {:inline 1} TestAddition_overflow_u8_add_bad (x: Value, y: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU8(x);
    assume IsValidU8(y);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := AddU8(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 4, tmp);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestAddition_overflow_u8_add_bad_verify (x: Value, y: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestAddition_overflow_u8_add_bad(x, y);
}

procedure {:inline 1} TestAddition_overflow_u8_add_ok (x: Value, y: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(x) + i#Integer(y))) > i#Integer(Integer(255)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(x) + i#Integer(y))) > i#Integer(Integer(255))))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU8(x);
    assume IsValidU8(y);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := AddU8(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 4, tmp);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestAddition_overflow_u8_add_ok_verify (x: Value, y: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestAddition_overflow_u8_add_ok(x, y);
}

procedure {:inline 1} TestAddition_overflow_u64_add_bad (x: Value, y: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
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
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := AddU64(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 4, tmp);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestAddition_overflow_u64_add_bad_verify (x: Value, y: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestAddition_overflow_u64_add_bad(x, y);
}

procedure {:inline 1} TestAddition_overflow_u64_add_ok (x: Value, y: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(x) + i#Integer(y))) > i#Integer(Integer(9223372036854775807)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(x) + i#Integer(y))) > i#Integer(Integer(9223372036854775807))))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
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
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := AddU64(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 4, tmp);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestAddition_overflow_u64_add_ok_verify (x: Value, y: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestAddition_overflow_u64_add_ok(x, y);
}

procedure {:inline 1} TestAddition_overflow_u128_add_bad (x: Value, y: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> abort_flag;
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU128(x);
    assume IsValidU128(y);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, x);
    m := UpdateLocal(m, old_size + 1, y);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := AddU128(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 4, tmp);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestAddition_overflow_u128_add_bad_verify (x: Value, y: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestAddition_overflow_u128_add_bad(x, y);
}
