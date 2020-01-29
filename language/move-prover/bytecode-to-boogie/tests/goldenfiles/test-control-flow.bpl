

// ** structs of module TestControlFlow



// ** functions of module TestControlFlow

procedure {:inline 1} TestControlFlow_branch_once (cond: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var __t1: Value; // BooleanType()
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestControlFlow#branch_once#0#cond: [Position]Value;
    var debug#TestControlFlow#branch_once#1#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 6;
    debug#TestControlFlow#branch_once#0#cond := EmptyPositionMap;
    debug#TestControlFlow#branch_once#1#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Boolean(cond);
    __m := UpdateLocal(__m, __frame + 0, cond);
    debug#TestControlFlow#branch_once#0#cond := debug#TestControlFlow#branch_once#0#cond[Position(117) := cond];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __tmp := GetLocal(__m, __frame + 1);
    if (!b#Boolean(__tmp)) { goto Label_6; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#TestControlFlow#branch_once#1#__ret := debug#TestControlFlow#branch_once#1#__ret[Position(180) := __ret0];
    return;

Label_6:
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    debug#TestControlFlow#branch_once#1#__ret := debug#TestControlFlow#branch_once#1#__ret[Position(206) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestControlFlow#branch_once#1#__ret := debug#TestControlFlow#branch_once#1#__ret[Position(221) := __ret0];
}

procedure TestControlFlow_branch_once_verify (cond: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestControlFlow_branch_once(cond);
}
