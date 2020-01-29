

// ** structs of module TestAddition



// ** functions of module TestAddition

procedure {:inline 1} TestAddition_overflow_u8_add_bad (x: Value, y: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAddition#overflow_u8_add_bad#0#x: [Position]Value;
    var debug#TestAddition#overflow_u8_add_bad#1#y: [Position]Value;
    var debug#TestAddition#overflow_u8_add_bad#2#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;
    debug#TestAddition#overflow_u8_add_bad#0#x := EmptyPositionMap;
    debug#TestAddition#overflow_u8_add_bad#1#y := EmptyPositionMap;
    debug#TestAddition#overflow_u8_add_bad#2#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU8(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAddition#overflow_u8_add_bad#0#x := debug#TestAddition#overflow_u8_add_bad#0#x[Position(72) := x];
    assume IsValidU8(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAddition#overflow_u8_add_bad#1#y := debug#TestAddition#overflow_u8_add_bad#1#y[Position(72) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := AddU8(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#TestAddition#overflow_u8_add_bad#2#__ret := debug#TestAddition#overflow_u8_add_bad#2#__ret[Position(188) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestAddition#overflow_u8_add_bad#2#__ret := debug#TestAddition#overflow_u8_add_bad#2#__ret[Position(218) := __ret0];
}

procedure TestAddition_overflow_u8_add_bad_verify (x: Value, y: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestAddition_overflow_u8_add_bad(x, y);
}

procedure {:inline 1} TestAddition_overflow_u8_add_ok (x: Value, y: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(x) + i#Integer(y))) > i#Integer(Integer(255)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(x) + i#Integer(y))) > i#Integer(Integer(255))))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAddition#overflow_u8_add_ok#0#x: [Position]Value;
    var debug#TestAddition#overflow_u8_add_ok#1#y: [Position]Value;
    var debug#TestAddition#overflow_u8_add_ok#2#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;
    debug#TestAddition#overflow_u8_add_ok#0#x := EmptyPositionMap;
    debug#TestAddition#overflow_u8_add_ok#1#y := EmptyPositionMap;
    debug#TestAddition#overflow_u8_add_ok#2#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU8(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAddition#overflow_u8_add_ok#0#x := debug#TestAddition#overflow_u8_add_ok#0#x[Position(270) := x];
    assume IsValidU8(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAddition#overflow_u8_add_ok#1#y := debug#TestAddition#overflow_u8_add_ok#1#y[Position(270) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := AddU8(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#TestAddition#overflow_u8_add_ok#2#__ret := debug#TestAddition#overflow_u8_add_ok#2#__ret[Position(370) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestAddition#overflow_u8_add_ok#2#__ret := debug#TestAddition#overflow_u8_add_ok#2#__ret[Position(400) := __ret0];
}

procedure TestAddition_overflow_u8_add_ok_verify (x: Value, y: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestAddition_overflow_u8_add_ok(x, y);
}

procedure {:inline 1} TestAddition_overflow_u64_add_bad (x: Value, y: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAddition#overflow_u64_add_bad#0#x: [Position]Value;
    var debug#TestAddition#overflow_u64_add_bad#1#y: [Position]Value;
    var debug#TestAddition#overflow_u64_add_bad#2#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;
    debug#TestAddition#overflow_u64_add_bad#0#x := EmptyPositionMap;
    debug#TestAddition#overflow_u64_add_bad#1#y := EmptyPositionMap;
    debug#TestAddition#overflow_u64_add_bad#2#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAddition#overflow_u64_add_bad#0#x := debug#TestAddition#overflow_u64_add_bad#0#x[Position(452) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAddition#overflow_u64_add_bad#1#y := debug#TestAddition#overflow_u64_add_bad#1#y[Position(452) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#TestAddition#overflow_u64_add_bad#2#__ret := debug#TestAddition#overflow_u64_add_bad#2#__ret[Position(572) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestAddition#overflow_u64_add_bad#2#__ret := debug#TestAddition#overflow_u64_add_bad#2#__ret[Position(602) := __ret0];
}

procedure TestAddition_overflow_u64_add_bad_verify (x: Value, y: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestAddition_overflow_u64_add_bad(x, y);
}

procedure {:inline 1} TestAddition_overflow_u64_add_ok (x: Value, y: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(x) + i#Integer(y))) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(x) + i#Integer(y))) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAddition#overflow_u64_add_ok#0#x: [Position]Value;
    var debug#TestAddition#overflow_u64_add_ok#1#y: [Position]Value;
    var debug#TestAddition#overflow_u64_add_ok#2#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;
    debug#TestAddition#overflow_u64_add_ok#0#x := EmptyPositionMap;
    debug#TestAddition#overflow_u64_add_ok#1#y := EmptyPositionMap;
    debug#TestAddition#overflow_u64_add_ok#2#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAddition#overflow_u64_add_ok#0#x := debug#TestAddition#overflow_u64_add_ok#0#x[Position(654) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAddition#overflow_u64_add_ok#1#y := debug#TestAddition#overflow_u64_add_ok#1#y[Position(654) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#TestAddition#overflow_u64_add_ok#2#__ret := debug#TestAddition#overflow_u64_add_ok#2#__ret[Position(773) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestAddition#overflow_u64_add_ok#2#__ret := debug#TestAddition#overflow_u64_add_ok#2#__ret[Position(803) := __ret0];
}

procedure TestAddition_overflow_u64_add_ok_verify (x: Value, y: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestAddition_overflow_u64_add_ok(x, y);
}

procedure {:inline 1} TestAddition_overflow_u128_add_bad (x: Value, y: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAddition#overflow_u128_add_bad#0#x: [Position]Value;
    var debug#TestAddition#overflow_u128_add_bad#1#y: [Position]Value;
    var debug#TestAddition#overflow_u128_add_bad#2#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;
    debug#TestAddition#overflow_u128_add_bad#0#x := EmptyPositionMap;
    debug#TestAddition#overflow_u128_add_bad#1#y := EmptyPositionMap;
    debug#TestAddition#overflow_u128_add_bad#2#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU128(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAddition#overflow_u128_add_bad#0#x := debug#TestAddition#overflow_u128_add_bad#0#x[Position(855) := x];
    assume IsValidU128(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAddition#overflow_u128_add_bad#1#y := debug#TestAddition#overflow_u128_add_bad#1#y[Position(855) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := AddU128(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#TestAddition#overflow_u128_add_bad#2#__ret := debug#TestAddition#overflow_u128_add_bad#2#__ret[Position(979) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestAddition#overflow_u128_add_bad#2#__ret := debug#TestAddition#overflow_u128_add_bad#2#__ret[Position(1009) := __ret0];
}

procedure TestAddition_overflow_u128_add_bad_verify (x: Value, y: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestAddition_overflow_u128_add_bad(x, y);
}
