

// ** structs of module TestStruct

const unique TestStruct_B: TypeName;
const TestStruct_B_addr: FieldName;
axiom TestStruct_B_addr == 0;
const TestStruct_B_val: FieldName;
axiom TestStruct_B_val == 1;
function TestStruct_B_type_value(): TypeValue {
    StructType(TestStruct_B, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, AddressType()), IntegerType()))
}
procedure {:inline 1} Pack_TestStruct_B(addr: Value, val: Value) returns (_struct: Value)
{
    assume is#Address(addr);
    assume IsValidU64(val);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, addr), val));
}

procedure {:inline 1} Unpack_TestStruct_B(_struct: Value) returns (addr: Value, val: Value)
{
    assume is#Vector(_struct);
    addr := SelectField(_struct, TestStruct_B_addr);
    assume is#Address(addr);
    val := SelectField(_struct, TestStruct_B_val);
    assume IsValidU64(val);
}

const unique TestStruct_A: TypeName;
const TestStruct_A_val: FieldName;
axiom TestStruct_A_val == 0;
const TestStruct_A_b: FieldName;
axiom TestStruct_A_b == 1;
function TestStruct_A_type_value(): TypeValue {
    StructType(TestStruct_A, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), TestStruct_B_type_value()))
}
procedure {:inline 1} Pack_TestStruct_A(val: Value, b: Value) returns (_struct: Value)
{
    assume IsValidU64(val);
    assume is#Vector(b);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, val), b));
}

procedure {:inline 1} Unpack_TestStruct_A(_struct: Value) returns (val: Value, b: Value)
{
    assume is#Vector(_struct);
    val := SelectField(_struct, TestStruct_A_val);
    assume IsValidU64(val);
    b := SelectField(_struct, TestStruct_A_b);
    assume is#Vector(b);
}

const unique TestStruct_C: TypeName;
const TestStruct_C_val: FieldName;
axiom TestStruct_C_val == 0;
const TestStruct_C_b: FieldName;
axiom TestStruct_C_b == 1;
function TestStruct_C_type_value(): TypeValue {
    StructType(TestStruct_C, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), TestStruct_A_type_value()))
}
procedure {:inline 1} Pack_TestStruct_C(val: Value, b: Value) returns (_struct: Value)
{
    assume IsValidU64(val);
    assume is#Vector(b);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, val), b));
}

procedure {:inline 1} Unpack_TestStruct_C(_struct: Value) returns (val: Value, b: Value)
{
    assume is#Vector(_struct);
    val := SelectField(_struct, TestStruct_C_val);
    assume IsValidU64(val);
    b := SelectField(_struct, TestStruct_C_b);
    assume is#Vector(b);
}

const unique TestStruct_T: TypeName;
const TestStruct_T_x: FieldName;
axiom TestStruct_T_x == 0;
function TestStruct_T_type_value(): TypeValue {
    StructType(TestStruct_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_TestStruct_T(x: Value) returns (_struct: Value)
{
    assume IsValidU64(x);
    _struct := Vector(ExtendValueArray(EmptyValueArray, x));
}

procedure {:inline 1} Unpack_TestStruct_T(_struct: Value) returns (x: Value)
{
    assume is#Vector(_struct);
    x := SelectField(_struct, TestStruct_T_x);
    assume IsValidU64(x);
}



// ** functions of module TestStruct

procedure {:inline 1} TestStruct_identity (a: Value, c: Value) returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var __t2: Value; // TestStruct_A_type_value()
    var __t3: Value; // TestStruct_C_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestStruct#identity#0#a: [Position]Value;
    var debug#TestStruct#identity#1#c: [Position]Value;
    var debug#TestStruct#identity#2#__ret: [Position]Value;
    var debug#TestStruct#identity#3#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;
    debug#TestStruct#identity#0#a := EmptyPositionMap;
    debug#TestStruct#identity#1#c := EmptyPositionMap;
    debug#TestStruct#identity#2#__ret := EmptyPositionMap;
    debug#TestStruct#identity#3#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    debug#TestStruct#identity#0#a := debug#TestStruct#identity#0#a[Position(252) := a];
    assume is#Vector(c);
    __m := UpdateLocal(__m, __frame + 1, c);
    debug#TestStruct#identity#1#c := debug#TestStruct#identity#1#c[Position(252) := c];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 2);
    debug#TestStruct#identity#2#__ret := debug#TestStruct#identity#2#__ret[Position(315) := __ret0];
    __ret1 := GetLocal(__m, __frame + 3);
    debug#TestStruct#identity#3#__ret := debug#TestStruct#identity#3#__ret[Position(315) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestStruct#identity#2#__ret := debug#TestStruct#identity#2#__ret[Position(344) := __ret0];
    __ret1 := DefaultValue;
    debug#TestStruct#identity#3#__ret := debug#TestStruct#identity#3#__ret[Position(344) := __ret1];
}

procedure TestStruct_identity_verify (a: Value, c: Value) returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := TestStruct_identity(a, c);
}

procedure {:inline 1} TestStruct_module_builtins (a: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t: Value; // TestStruct_T_type_value()
    var t_ref1: Reference; // ReferenceType(TestStruct_T_type_value())
    var t_ref2: Reference; // ReferenceType(TestStruct_T_type_value())
    var b: Value; // BooleanType()
    var __t5: Value; // AddressType()
    var __t6: Value; // BooleanType()
    var __t7: Value; // BooleanType()
    var __t8: Value; // BooleanType()
    var __t9: Value; // IntegerType()
    var __t10: Value; // AddressType()
    var __t11: Reference; // ReferenceType(TestStruct_T_type_value())
    var __t12: Reference; // ReferenceType(TestStruct_T_type_value())
    var __t13: Value; // AddressType()
    var __t14: Reference; // ReferenceType(TestStruct_T_type_value())
    var __t15: Reference; // ReferenceType(TestStruct_T_type_value())
    var __t16: Value; // AddressType()
    var __t17: Value; // TestStruct_T_type_value()
    var __t18: Value; // TestStruct_T_type_value()
    var __t19: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestStruct#module_builtins#0#a: [Position]Value;
    var debug#TestStruct#module_builtins#1#t: [Position]Value;
    var debug#TestStruct#module_builtins#2#t_ref1: [Position]Value;
    var debug#TestStruct#module_builtins#3#t_ref2: [Position]Value;
    var debug#TestStruct#module_builtins#4#b: [Position]Value;
    var debug#TestStruct#module_builtins#5#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 20;
    debug#TestStruct#module_builtins#0#a := EmptyPositionMap;
    debug#TestStruct#module_builtins#1#t := EmptyPositionMap;
    debug#TestStruct#module_builtins#2#t_ref1 := EmptyPositionMap;
    debug#TestStruct#module_builtins#3#t_ref2 := EmptyPositionMap;
    debug#TestStruct#module_builtins#4#b := EmptyPositionMap;
    debug#TestStruct#module_builtins#5#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Address(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    debug#TestStruct#module_builtins#0#a := debug#TestStruct#module_builtins#0#a[Position(351) := a];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 5), TestStruct_T_type_value());
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 4, __tmp);
    debug#TestStruct#module_builtins#4#b := debug#TestStruct#module_builtins#4#b[Position(528) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    __tmp := GetLocal(__m, __frame + 8);
    if (!b#Boolean(__tmp)) { goto Label_8; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    goto Label_Abort;

Label_8:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __t11 := BorrowGlobal(GetLocal(__m, __frame + 10), TestStruct_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t_ref1 := CopyOrMoveRef(__t11);

    call __t12 := CopyOrMoveRef(t_ref1);

    // unimplemented instruction: NoOp

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __t14 := BorrowGlobal(GetLocal(__m, __frame + 13), TestStruct_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t_ref2 := CopyOrMoveRef(__t14);

    call __t15 := CopyOrMoveRef(t_ref2);

    // unimplemented instruction: NoOp

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := MoveFrom(GetLocal(__m, __frame + 16), TestStruct_T_type_value());
    __m := UpdateLocal(__m, __frame + 17, __tmp);
    assume is#Vector(__t17);
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 17));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#TestStruct#module_builtins#1#t := debug#TestStruct#module_builtins#1#t[Position(737) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call MoveToSender(TestStruct_T_type_value(), GetLocal(__m, __frame + 18));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    __ret0 := GetLocal(__m, __frame + 19);
    debug#TestStruct#module_builtins#5#__ret := debug#TestStruct#module_builtins#5#__ret[Position(808) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestStruct#module_builtins#5#__ret := debug#TestStruct#module_builtins#5#__ret[Position(828) := __ret0];
}

procedure TestStruct_module_builtins_verify (a: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestStruct_module_builtins(a);
}

procedure {:inline 1} TestStruct_nested_struct (a: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var var_a: Value; // TestStruct_A_type_value()
    var var_b: Value; // TestStruct_B_type_value()
    var var_b_ref: Reference; // ReferenceType(TestStruct_B_type_value())
    var b_val_ref: Reference; // ReferenceType(IntegerType())
    var b_val: Value; // IntegerType()
    var __t6: Value; // BooleanType()
    var __t7: Value; // AddressType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // TestStruct_B_type_value()
    var __t10: Value; // AddressType()
    var __t11: Value; // IntegerType()
    var __t12: Value; // TestStruct_B_type_value()
    var __t13: Reference; // ReferenceType(TestStruct_B_type_value())
    var __t14: Reference; // ReferenceType(TestStruct_B_type_value())
    var __t15: Reference; // ReferenceType(IntegerType())
    var __t16: Reference; // ReferenceType(IntegerType())
    var __t17: Value; // IntegerType()
    var __t18: Value; // IntegerType()
    var __t19: Value; // IntegerType()
    var __t20: Value; // BooleanType()
    var __t21: Value; // BooleanType()
    var __t22: Value; // IntegerType()
    var __t23: Value; // TestStruct_B_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestStruct#nested_struct#0#a: [Position]Value;
    var debug#TestStruct#nested_struct#1#var_a: [Position]Value;
    var debug#TestStruct#nested_struct#2#var_b: [Position]Value;
    var debug#TestStruct#nested_struct#3#var_b_ref: [Position]Value;
    var debug#TestStruct#nested_struct#4#b_val_ref: [Position]Value;
    var debug#TestStruct#nested_struct#5#b_val: [Position]Value;
    var debug#TestStruct#nested_struct#6#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 24;
    debug#TestStruct#nested_struct#0#a := EmptyPositionMap;
    debug#TestStruct#nested_struct#1#var_a := EmptyPositionMap;
    debug#TestStruct#nested_struct#2#var_b := EmptyPositionMap;
    debug#TestStruct#nested_struct#3#var_b_ref := EmptyPositionMap;
    debug#TestStruct#nested_struct#4#b_val_ref := EmptyPositionMap;
    debug#TestStruct#nested_struct#5#b_val := EmptyPositionMap;
    debug#TestStruct#nested_struct#6#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Address(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    debug#TestStruct#nested_struct#0#a := debug#TestStruct#nested_struct#0#a[Position(835) := a];

    // bytecode translation starts here
    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Pack_TestStruct_B(GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#TestStruct#nested_struct#2#var_b := debug#TestStruct#nested_struct#2#var_b[Position(1076) := __tmp];

    goto Label_11;

Label_7:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := Pack_TestStruct_B(GetLocal(__m, __frame + 10), GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#TestStruct#nested_struct#2#var_b := debug#TestStruct#nested_struct#2#var_b[Position(1142) := __tmp];

Label_11:
    call __t13 := BorrowLoc(__frame + 2);

    call var_b_ref := CopyOrMoveRef(__t13);

    call __t14 := CopyOrMoveRef(var_b_ref);

    call __t15 := BorrowField(__t14, TestStruct_B_val);

    call b_val_ref := CopyOrMoveRef(__t15);

    call __t16 := CopyOrMoveRef(b_val_ref);

    call __tmp := ReadRef(__t16);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 17));
    __m := UpdateLocal(__m, __frame + 5, __tmp);
    debug#TestStruct#nested_struct#5#b_val := debug#TestStruct#nested_struct#5#b_val[Position(1268) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 18), GetLocal(__m, __frame + 19)));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    __tmp := GetLocal(__m, __frame + 21);
    if (!b#Boolean(__tmp)) { goto Label_26; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    goto Label_Abort;

Label_26:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    __ret0 := GetLocal(__m, __frame + 23);
    debug#TestStruct#nested_struct#6#__ret := debug#TestStruct#nested_struct#6#__ret[Position(1341) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestStruct#nested_struct#6#__ret := debug#TestStruct#nested_struct#6#__ret[Position(1365) := __ret0];
}

procedure TestStruct_nested_struct_verify (a: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestStruct_nested_struct(a);
}

procedure {:inline 1} TestStruct_try_unpack (a: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var v: Value; // IntegerType()
    var b: Value; // TestStruct_B_type_value()
    var aa: Value; // AddressType()
    var __t4: Value; // AddressType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // TestStruct_B_type_value()
    var __t7: Value; // TestStruct_B_type_value()
    var __t8: Value; // AddressType()
    var __t9: Value; // IntegerType()
    var __t10: Value; // AddressType()
    var __t11: Value; // AddressType()
    var __t12: Value; // BooleanType()
    var __t13: Value; // BooleanType()
    var __t14: Value; // IntegerType()
    var __t15: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestStruct#try_unpack#0#a: [Position]Value;
    var debug#TestStruct#try_unpack#1#v: [Position]Value;
    var debug#TestStruct#try_unpack#2#b: [Position]Value;
    var debug#TestStruct#try_unpack#3#aa: [Position]Value;
    var debug#TestStruct#try_unpack#4#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 16;
    debug#TestStruct#try_unpack#0#a := EmptyPositionMap;
    debug#TestStruct#try_unpack#1#v := EmptyPositionMap;
    debug#TestStruct#try_unpack#2#b := EmptyPositionMap;
    debug#TestStruct#try_unpack#3#aa := EmptyPositionMap;
    debug#TestStruct#try_unpack#4#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Address(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    debug#TestStruct#try_unpack#0#a := debug#TestStruct#try_unpack#0#a[Position(1372) := a];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Pack_TestStruct_B(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#TestStruct#try_unpack#2#b := debug#TestStruct#try_unpack#2#b[Position(1509) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8, __t9 := Unpack_TestStruct_B(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __t8);
    __m := UpdateLocal(__m, __frame + 9, __t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#TestStruct#try_unpack#1#v := debug#TestStruct#try_unpack#1#v[Position(1559) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    debug#TestStruct#try_unpack#3#aa := debug#TestStruct#try_unpack#3#aa[Position(1555) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 10), GetLocal(__m, __frame + 11)));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    __tmp := GetLocal(__m, __frame + 13);
    if (!b#Boolean(__tmp)) { goto Label_15; }

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    goto Label_Abort;

Label_15:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    __ret0 := GetLocal(__m, __frame + 15);
    debug#TestStruct#try_unpack#4#__ret := debug#TestStruct#try_unpack#4#__ret[Position(1622) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestStruct#try_unpack#4#__ret := debug#TestStruct#try_unpack#4#__ret[Position(1642) := __ret0];
}

procedure TestStruct_try_unpack_verify (a: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestStruct_try_unpack(a);
}
