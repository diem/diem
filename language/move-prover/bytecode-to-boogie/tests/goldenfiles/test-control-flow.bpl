

// ** structs of module TestControlFlow



// ** functions of module TestControlFlow

procedure {:inline 1} TestControlFlow_branch_once (cond: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // BooleanType()
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
    assume is#Boolean(cond);

    old_size := local_counter;
    local_counter := local_counter + 6;
    m := UpdateLocal(m, old_size + 0, cond);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    tmp := GetLocal(m, old_size + 1);
    if (!b#Boolean(tmp)) { goto Label_6; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := LdConst(2);
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := Add(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 4, tmp);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_6:
    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 5, tmp);

    ret0 := GetLocal(m, old_size + 5);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestControlFlow_branch_once_verify (cond: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestControlFlow_branch_once(cond);
}
