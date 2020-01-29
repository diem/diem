

// ** structs of module TestSpecs



// ** functions of module TestSpecs

procedure {:inline 1} TestSpecs_div (x: Value, y: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(b#Boolean(Boolean(i#Integer(y) > i#Integer(Integer(0))))) ==> !__abort_flag;
{
    // declare local variables
    var r: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Div(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __ret0 := GetLocal(__m, __frame + 6);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestSpecs_div_verify (x: Value, y: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call __ret0 := TestSpecs_div(x, y);
}

procedure {:inline 1} TestSpecs_div_by_zero_detected (x: Value, y: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(b#Boolean(Boolean(true))) ==> !__abort_flag;
{
    // declare local variables
    var r: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Div(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __ret0 := GetLocal(__m, __frame + 6);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestSpecs_div_by_zero_detected_verify (x: Value, y: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call __ret0 := TestSpecs_div_by_zero_detected(x, y);
}
