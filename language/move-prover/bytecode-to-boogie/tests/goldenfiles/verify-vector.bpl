

// ** structs of module VerifyVector



// ** functions of module VerifyVector

procedure {:inline 1} VerifyVector_test_empty1 () returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures b#Boolean(Boolean((ret0) == (ret1)));
{
    // declare local variables
    var t0: Value; // Vector_T_type_value(IntegerType())
    var t1: Value; // Vector_T_type_value(IntegerType())
    var t2: Value; // Vector_T_type_value(IntegerType())
    var t3: Value; // Vector_T_type_value(IntegerType())
    var t4: Value; // Vector_T_type_value(IntegerType())
    var t5: Value; // Vector_T_type_value(IntegerType())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 6;

    // bytecode translation starts here
    call t2 := Vector_empty(IntegerType());
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t2);

    m := UpdateLocal(m, old_size + 2, t2);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 0, tmp);

    call t3 := Vector_empty(IntegerType());
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t3);

    m := UpdateLocal(m, old_size + 3, t3);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    ret0 := GetLocal(m, old_size + 4);
    ret1 := GetLocal(m, old_size + 5);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
}

procedure VerifyVector_test_empty1_verify () returns (ret0: Value, ret1: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0, ret1 := VerifyVector_test_empty1();
}

procedure {:inline 1} VerifyVector_test_empty2 () returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures b#Boolean(Boolean((ret0) == (ret1)));
{
    // declare local variables
    var t0: Value; // Vector_T_type_value(IntegerType())
    var t1: Value; // Vector_T_type_value(IntegerType())
    var t2: Value; // IntegerType()
    var t3: Value; // Vector_T_type_value(IntegerType())
    var t4: Value; // Vector_T_type_value(IntegerType())
    var t5: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var t6: Value; // IntegerType()
    var t7: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var t8: Value; // IntegerType()
    var t9: Value; // Vector_T_type_value(IntegerType())
    var t10: Value; // Vector_T_type_value(IntegerType())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 11;

    // bytecode translation starts here
    call t3 := Vector_empty(IntegerType());
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t3);

    m := UpdateLocal(m, old_size + 3, t3);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 0, tmp);

    call t4 := Vector_empty(IntegerType());
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t4);

    m := UpdateLocal(m, old_size + 4, t4);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 1, tmp);

    call t5 := BorrowLoc(old_size+0);

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 6, tmp);

    call Vector_push_back(IntegerType(), t5, GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }

    call t7 := BorrowLoc(old_size+0);

    call t8 := Vector_pop_back(IntegerType(), t7);
    if (abort_flag) { goto Label_Abort; }
    assume IsValidU64(t8);

    m := UpdateLocal(m, old_size + 8, t8);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 10, tmp);

    ret0 := GetLocal(m, old_size + 9);
    ret1 := GetLocal(m, old_size + 10);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
}

procedure VerifyVector_test_empty2_verify () returns (ret0: Value, ret1: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0, ret1 := VerifyVector_test_empty2();
}
