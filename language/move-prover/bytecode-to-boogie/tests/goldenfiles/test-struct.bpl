

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

procedure {:inline 1} TestStruct_identity (a: Value, c: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Value; // TestStruct_A_type_value()
    var t3: Value; // TestStruct_C_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume is#Vector(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    assume is#Vector(c);
    __m := UpdateLocal(__m, __frame + 1, c);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    ret0 := GetLocal(__m, __frame + 2);
    ret1 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
}

procedure TestStruct_identity_verify (a: Value, c: Value) returns (ret0: Value, ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0, ret1 := TestStruct_identity(a, c);
}

procedure {:inline 1} TestStruct_module_builtins (a: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // TestStruct_T_type_value()
    var t2: Reference; // ReferenceType(TestStruct_T_type_value())
    var t3: Reference; // ReferenceType(TestStruct_T_type_value())
    var t4: Value; // BooleanType()
    var t5: Value; // AddressType()
    var t6: Value; // BooleanType()
    var t7: Value; // BooleanType()
    var t8: Value; // BooleanType()
    var t9: Value; // IntegerType()
    var t10: Value; // AddressType()
    var t11: Reference; // ReferenceType(TestStruct_T_type_value())
    var t12: Reference; // ReferenceType(TestStruct_T_type_value())
    var t13: Value; // AddressType()
    var t14: Reference; // ReferenceType(TestStruct_T_type_value())
    var t15: Reference; // ReferenceType(TestStruct_T_type_value())
    var t16: Value; // AddressType()
    var t17: Value; // TestStruct_T_type_value()
    var t18: Value; // TestStruct_T_type_value()
    var t19: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 20;

    // process and type check arguments
    assume is#Address(a);
    __m := UpdateLocal(__m, __frame + 0, a);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 5), TestStruct_T_type_value());
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

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

    call t11 := BorrowGlobal(GetLocal(__m, __frame + 10), TestStruct_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t11);

    call t12 := CopyOrMoveRef(t2);

    // unimplemented instruction: NoOp

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call t14 := BorrowGlobal(GetLocal(__m, __frame + 13), TestStruct_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t3 := CopyOrMoveRef(t14);

    call t15 := CopyOrMoveRef(t3);

    // unimplemented instruction: NoOp

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := MoveFrom(GetLocal(__m, __frame + 16), TestStruct_T_type_value());
    __m := UpdateLocal(__m, __frame + 17, __tmp);
    assume is#Vector(t17);
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 17));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call MoveToSender(TestStruct_T_type_value(), GetLocal(__m, __frame + 18));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    ret0 := GetLocal(__m, __frame + 19);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure TestStruct_module_builtins_verify (a: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := TestStruct_module_builtins(a);
}

procedure {:inline 1} TestStruct_nested_struct (a: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var t1: Value; // TestStruct_A_type_value()
    var t2: Value; // TestStruct_B_type_value()
    var t3: Reference; // ReferenceType(TestStruct_B_type_value())
    var t4: Reference; // ReferenceType(IntegerType())
    var t5: Value; // IntegerType()
    var t6: Value; // BooleanType()
    var t7: Value; // AddressType()
    var t8: Value; // IntegerType()
    var t9: Value; // TestStruct_B_type_value()
    var t10: Value; // AddressType()
    var t11: Value; // IntegerType()
    var t12: Value; // TestStruct_B_type_value()
    var t13: Reference; // ReferenceType(TestStruct_B_type_value())
    var t14: Reference; // ReferenceType(TestStruct_B_type_value())
    var t15: Reference; // ReferenceType(IntegerType())
    var t16: Reference; // ReferenceType(IntegerType())
    var t17: Value; // IntegerType()
    var t18: Value; // IntegerType()
    var t19: Value; // IntegerType()
    var t20: Value; // BooleanType()
    var t21: Value; // BooleanType()
    var t22: Value; // IntegerType()
    var t23: Value; // TestStruct_B_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 24;

    // process and type check arguments
    assume is#Address(a);
    __m := UpdateLocal(__m, __frame + 0, a);

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

Label_11:
    call t13 := BorrowLoc(__frame + 2);

    call t3 := CopyOrMoveRef(t13);

    call t14 := CopyOrMoveRef(t3);

    call t15 := BorrowField(t14, TestStruct_B_val);

    call t4 := CopyOrMoveRef(t15);

    call t16 := CopyOrMoveRef(t4);

    call __tmp := ReadRef(t16);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 17));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

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

    ret0 := GetLocal(__m, __frame + 23);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure TestStruct_nested_struct_verify (a: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := TestStruct_nested_struct(a);
}

procedure {:inline 1} TestStruct_try_unpack (a: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // TestStruct_B_type_value()
    var t3: Value; // AddressType()
    var t4: Value; // AddressType()
    var t5: Value; // IntegerType()
    var t6: Value; // TestStruct_B_type_value()
    var t7: Value; // TestStruct_B_type_value()
    var t8: Value; // AddressType()
    var t9: Value; // IntegerType()
    var t10: Value; // AddressType()
    var t11: Value; // AddressType()
    var t12: Value; // BooleanType()
    var t13: Value; // BooleanType()
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 16;

    // process and type check arguments
    assume is#Address(a);
    __m := UpdateLocal(__m, __frame + 0, a);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Pack_TestStruct_B(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call t8, t9 := Unpack_TestStruct_B(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, t8);
    __m := UpdateLocal(__m, __frame + 9, t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

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

    ret0 := GetLocal(__m, __frame + 15);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure TestStruct_try_unpack_verify (a: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := TestStruct_try_unpack(a);
}
