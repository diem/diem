

// ** structs of module CastBad



// ** functions of module CastBad

procedure {:inline 1} CastBad_aborting_u8_cast_bad (x: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> abort_flag;
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(x);

    old_size := local_counter;
    local_counter := local_counter + 3;
    m := UpdateLocal(m, old_size + 0, x);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CastU8(GetLocal(m, old_size + 1));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 2, tmp);

    ret0 := GetLocal(m, old_size + 2);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure CastBad_aborting_u8_cast_bad_verify (x: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := CastBad_aborting_u8_cast_bad(x);
}

procedure {:inline 1} CastBad_aborting_u8_cast_ok (x: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) > i#Integer(Integer(255)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) > i#Integer(Integer(255))))) ==> abort_flag;
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(x);

    old_size := local_counter;
    local_counter := local_counter + 3;
    m := UpdateLocal(m, old_size + 0, x);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CastU8(GetLocal(m, old_size + 1));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 2, tmp);

    ret0 := GetLocal(m, old_size + 2);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure CastBad_aborting_u8_cast_ok_verify (x: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := CastBad_aborting_u8_cast_ok(x);
}

procedure {:inline 1} CastBad_aborting_u64_cast_bad (x: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> abort_flag;
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU128(x);

    old_size := local_counter;
    local_counter := local_counter + 3;
    m := UpdateLocal(m, old_size + 0, x);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CastU64(GetLocal(m, old_size + 1));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 2, tmp);

    ret0 := GetLocal(m, old_size + 2);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure CastBad_aborting_u64_cast_bad_verify (x: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := CastBad_aborting_u64_cast_bad(x);
}

procedure {:inline 1} CastBad_aborting_u64_cast_ok (x: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) > i#Integer(Integer(9223372036854775807)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) > i#Integer(Integer(9223372036854775807))))) ==> abort_flag;
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU128(x);

    old_size := local_counter;
    local_counter := local_counter + 3;
    m := UpdateLocal(m, old_size + 0, x);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CastU64(GetLocal(m, old_size + 1));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 2, tmp);

    ret0 := GetLocal(m, old_size + 2);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure CastBad_aborting_u64_cast_ok_verify (x: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := CastBad_aborting_u64_cast_ok(x);
}
