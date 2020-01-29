

// ** structs of module TestArithmetic



// ** functions of module TestArithmetic

procedure {:inline 1} TestArithmetic_add_two_number (x: Value, y: Value) returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var res: Value; // IntegerType()
    var z: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestArithmetic#add_two_number#0#x: [Position]Value;
    var debug#TestArithmetic#add_two_number#1#y: [Position]Value;
    var debug#TestArithmetic#add_two_number#2#res: [Position]Value;
    var debug#TestArithmetic#add_two_number#3#z: [Position]Value;
    var debug#TestArithmetic#add_two_number#4#__ret: [Position]Value;
    var debug#TestArithmetic#add_two_number#5#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 10;
    debug#TestArithmetic#add_two_number#0#x := EmptyPositionMap;
    debug#TestArithmetic#add_two_number#1#y := EmptyPositionMap;
    debug#TestArithmetic#add_two_number#2#res := EmptyPositionMap;
    debug#TestArithmetic#add_two_number#3#z := EmptyPositionMap;
    debug#TestArithmetic#add_two_number#4#__ret := EmptyPositionMap;
    debug#TestArithmetic#add_two_number#5#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestArithmetic#add_two_number#0#x := debug#TestArithmetic#add_two_number#0#x[Position(25) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestArithmetic#add_two_number#1#y := debug#TestArithmetic#add_two_number#1#y[Position(25) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#TestArithmetic#add_two_number#2#res := debug#TestArithmetic#add_two_number#2#res[Position(106) := __tmp];

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    debug#TestArithmetic#add_two_number#3#z := debug#TestArithmetic#add_two_number#3#z[Position(133) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    debug#TestArithmetic#add_two_number#4#__ret := debug#TestArithmetic#add_two_number#4#__ret[Position(142) := __ret0];
    __ret1 := GetLocal(__m, __frame + 9);
    debug#TestArithmetic#add_two_number#5#__ret := debug#TestArithmetic#add_two_number#5#__ret[Position(142) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestArithmetic#add_two_number#4#__ret := debug#TestArithmetic#add_two_number#4#__ret[Position(169) := __ret0];
    __ret1 := DefaultValue;
    debug#TestArithmetic#add_two_number#5#__ret := debug#TestArithmetic#add_two_number#5#__ret[Position(169) := __ret1];
}

procedure TestArithmetic_add_two_number_verify (x: Value, y: Value) returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := TestArithmetic_add_two_number(x, y);
}

procedure {:inline 1} TestArithmetic_multiple_ops (x: Value, y: Value, z: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var res: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestArithmetic#multiple_ops#0#x: [Position]Value;
    var debug#TestArithmetic#multiple_ops#1#y: [Position]Value;
    var debug#TestArithmetic#multiple_ops#2#z: [Position]Value;
    var debug#TestArithmetic#multiple_ops#3#res: [Position]Value;
    var debug#TestArithmetic#multiple_ops#4#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 10;
    debug#TestArithmetic#multiple_ops#0#x := EmptyPositionMap;
    debug#TestArithmetic#multiple_ops#1#y := EmptyPositionMap;
    debug#TestArithmetic#multiple_ops#2#z := EmptyPositionMap;
    debug#TestArithmetic#multiple_ops#3#res := EmptyPositionMap;
    debug#TestArithmetic#multiple_ops#4#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestArithmetic#multiple_ops#0#x := debug#TestArithmetic#multiple_ops#0#x[Position(173) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestArithmetic#multiple_ops#1#y := debug#TestArithmetic#multiple_ops#1#y[Position(173) := y];
    assume IsValidU64(z);
    __m := UpdateLocal(__m, __frame + 2, z);
    debug#TestArithmetic#multiple_ops#2#z := debug#TestArithmetic#multiple_ops#2#z[Position(173) := z];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    debug#TestArithmetic#multiple_ops#3#res := debug#TestArithmetic#multiple_ops#3#res[Position(242) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 9);
    debug#TestArithmetic#multiple_ops#4#__ret := debug#TestArithmetic#multiple_ops#4#__ret[Position(281) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestArithmetic#multiple_ops#4#__ret := debug#TestArithmetic#multiple_ops#4#__ret[Position(300) := __ret0];
}

procedure TestArithmetic_multiple_ops_verify (x: Value, y: Value, z: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestArithmetic_multiple_ops(x, y, z);
}

procedure {:inline 1} TestArithmetic_bool_ops (a: Value, b: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var c: Value; // BooleanType()
    var d: Value; // BooleanType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // BooleanType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // BooleanType()
    var __t10: Value; // BooleanType()
    var __t11: Value; // IntegerType()
    var __t12: Value; // IntegerType()
    var __t13: Value; // BooleanType()
    var __t14: Value; // IntegerType()
    var __t15: Value; // IntegerType()
    var __t16: Value; // BooleanType()
    var __t17: Value; // BooleanType()
    var __t18: Value; // BooleanType()
    var __t19: Value; // BooleanType()
    var __t20: Value; // BooleanType()
    var __t21: Value; // BooleanType()
    var __t22: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestArithmetic#bool_ops#0#a: [Position]Value;
    var debug#TestArithmetic#bool_ops#1#b: [Position]Value;
    var debug#TestArithmetic#bool_ops#2#c: [Position]Value;
    var debug#TestArithmetic#bool_ops#3#d: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 23;
    debug#TestArithmetic#bool_ops#0#a := EmptyPositionMap;
    debug#TestArithmetic#bool_ops#1#b := EmptyPositionMap;
    debug#TestArithmetic#bool_ops#2#c := EmptyPositionMap;
    debug#TestArithmetic#bool_ops#3#d := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    debug#TestArithmetic#bool_ops#0#a := debug#TestArithmetic#bool_ops#0#a[Position(304) := a];
    assume IsValidU64(b);
    __m := UpdateLocal(__m, __frame + 1, b);
    debug#TestArithmetic#bool_ops#1#b := debug#TestArithmetic#bool_ops#1#b[Position(304) := b];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Ge(GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := And(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 10));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#TestArithmetic#bool_ops#2#c := debug#TestArithmetic#bool_ops#2#c[Position(382) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Lt(GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := Le(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := Or(GetLocal(__m, __frame + 13), GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 17));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    debug#TestArithmetic#bool_ops#3#d := debug#TestArithmetic#bool_ops#3#d[Position(437) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    __tmp := Boolean(!IsEqual(GetLocal(__m, __frame + 18), GetLocal(__m, __frame + 19)));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    __tmp := GetLocal(__m, __frame + 21);
    if (!b#Boolean(__tmp)) { goto Label_23; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    goto Label_Abort;

Label_23:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestArithmetic_bool_ops_verify (a: Value, b: Value) returns ()
{
    call InitVerification();
    call TestArithmetic_bool_ops(a, b);
}

procedure {:inline 1} TestArithmetic_arithmetic_ops (a: Value, b: Value) returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var c: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // IntegerType()
    var __t10: Value; // IntegerType()
    var __t11: Value; // IntegerType()
    var __t12: Value; // IntegerType()
    var __t13: Value; // IntegerType()
    var __t14: Value; // IntegerType()
    var __t15: Value; // IntegerType()
    var __t16: Value; // BooleanType()
    var __t17: Value; // BooleanType()
    var __t18: Value; // IntegerType()
    var __t19: Value; // IntegerType()
    var __t20: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestArithmetic#arithmetic_ops#0#a: [Position]Value;
    var debug#TestArithmetic#arithmetic_ops#1#b: [Position]Value;
    var debug#TestArithmetic#arithmetic_ops#2#c: [Position]Value;
    var debug#TestArithmetic#arithmetic_ops#3#__ret: [Position]Value;
    var debug#TestArithmetic#arithmetic_ops#4#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 21;
    debug#TestArithmetic#arithmetic_ops#0#a := EmptyPositionMap;
    debug#TestArithmetic#arithmetic_ops#1#b := EmptyPositionMap;
    debug#TestArithmetic#arithmetic_ops#2#c := EmptyPositionMap;
    debug#TestArithmetic#arithmetic_ops#3#__ret := EmptyPositionMap;
    debug#TestArithmetic#arithmetic_ops#4#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    debug#TestArithmetic#arithmetic_ops#0#a := debug#TestArithmetic#arithmetic_ops#0#a[Position(547) := a];
    assume IsValidU64(b);
    __m := UpdateLocal(__m, __frame + 1, b);
    debug#TestArithmetic#arithmetic_ops#1#b := debug#TestArithmetic#arithmetic_ops#1#b[Position(547) := b];

    // bytecode translation starts here
    call __tmp := LdConst(6);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(4);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := Div(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := LdConst(4);
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Mod(GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 12));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 13));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#TestArithmetic#arithmetic_ops#2#c := debug#TestArithmetic#arithmetic_ops#2#c[Position(622) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15)));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    __tmp := GetLocal(__m, __frame + 17);
    if (!b#Boolean(__tmp)) { goto Label_19; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    goto Label_Abort;

Label_19:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    __ret0 := GetLocal(__m, __frame + 19);
    debug#TestArithmetic#arithmetic_ops#3#__ret := debug#TestArithmetic#arithmetic_ops#3#__ret[Position(686) := __ret0];
    __ret1 := GetLocal(__m, __frame + 20);
    debug#TestArithmetic#arithmetic_ops#4#__ret := debug#TestArithmetic#arithmetic_ops#4#__ret[Position(686) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestArithmetic#arithmetic_ops#3#__ret := debug#TestArithmetic#arithmetic_ops#3#__ret[Position(713) := __ret0];
    __ret1 := DefaultValue;
    debug#TestArithmetic#arithmetic_ops#4#__ret := debug#TestArithmetic#arithmetic_ops#4#__ret[Position(713) := __ret1];
}

procedure TestArithmetic_arithmetic_ops_verify (a: Value, b: Value) returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := TestArithmetic_arithmetic_ops(a, b);
}

procedure {:inline 1} TestArithmetic_overflow () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var x: Value; // IntegerType()
    var y: Value; // IntegerType()
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestArithmetic#overflow#0#x: [Position]Value;
    var debug#TestArithmetic#overflow#1#y: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 6;
    debug#TestArithmetic#overflow#0#x := EmptyPositionMap;
    debug#TestArithmetic#overflow#1#y := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdConst(9223372036854775807);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#TestArithmetic#overflow#0#x := debug#TestArithmetic#overflow#0#x[Position(771) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#TestArithmetic#overflow#1#y := debug#TestArithmetic#overflow#1#y[Position(799) := __tmp];

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestArithmetic_overflow_verify () returns ()
{
    call InitVerification();
    call TestArithmetic_overflow();
}

procedure {:inline 1} TestArithmetic_underflow () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var x: Value; // IntegerType()
    var y: Value; // IntegerType()
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestArithmetic#underflow#0#x: [Position]Value;
    var debug#TestArithmetic#underflow#1#y: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 6;
    debug#TestArithmetic#underflow#0#x := EmptyPositionMap;
    debug#TestArithmetic#underflow#1#y := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#TestArithmetic#underflow#0#x := debug#TestArithmetic#underflow#0#x[Position(887) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#TestArithmetic#underflow#1#y := debug#TestArithmetic#underflow#1#y[Position(897) := __tmp];

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestArithmetic_underflow_verify () returns ()
{
    call InitVerification();
    call TestArithmetic_underflow();
}

procedure {:inline 1} TestArithmetic_div_by_zero () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var x: Value; // IntegerType()
    var y: Value; // IntegerType()
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestArithmetic#div_by_zero#0#x: [Position]Value;
    var debug#TestArithmetic#div_by_zero#1#y: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 6;
    debug#TestArithmetic#div_by_zero#0#x := EmptyPositionMap;
    debug#TestArithmetic#div_by_zero#1#y := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#TestArithmetic#div_by_zero#0#x := debug#TestArithmetic#div_by_zero#0#x[Position(987) := __tmp];

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Div(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#TestArithmetic#div_by_zero#1#y := debug#TestArithmetic#div_by_zero#1#y[Position(997) := __tmp];

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestArithmetic_div_by_zero_verify () returns ()
{
    call InitVerification();
    call TestArithmetic_div_by_zero();
}
