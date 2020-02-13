

// ** synthetics of module TestStruct



// ** structs of module TestStruct

const unique TestStruct_B: TypeName;
const TestStruct_B_addr: FieldName;
axiom TestStruct_B_addr == 0;
const TestStruct_B_val: FieldName;
axiom TestStruct_B_val == 1;
function TestStruct_B_type_value(): TypeValue {
    StructType(TestStruct_B, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, AddressType()), IntegerType()))
}
function {:inline 1} $TestStruct_B_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && is#Address(SelectField(__this, TestStruct_B_addr))
        && IsValidU64(SelectField(__this, TestStruct_B_val))
}

procedure {:inline 1} Pack_TestStruct_B(module_idx: int, func_idx: int, var_idx: int, code_idx: int, addr: Value, val: Value) returns (_struct: Value)
{
    assume is#Address(addr);
    assume IsValidU64(val);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, addr), val));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
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
function {:inline 1} $TestStruct_A_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, TestStruct_A_val))
        && $TestStruct_B_is_well_formed(SelectField(__this, TestStruct_A_b))
}

procedure {:inline 1} Pack_TestStruct_A(module_idx: int, func_idx: int, var_idx: int, code_idx: int, val: Value, b: Value) returns (_struct: Value)
{
    assume IsValidU64(val);
    assume $TestStruct_B_is_well_formed(b);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, val), b));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_TestStruct_A(_struct: Value) returns (val: Value, b: Value)
{
    assume is#Vector(_struct);
    val := SelectField(_struct, TestStruct_A_val);
    assume IsValidU64(val);
    b := SelectField(_struct, TestStruct_A_b);
    assume $TestStruct_B_is_well_formed(b);
}

const unique TestStruct_C: TypeName;
const TestStruct_C_val: FieldName;
axiom TestStruct_C_val == 0;
const TestStruct_C_b: FieldName;
axiom TestStruct_C_b == 1;
function TestStruct_C_type_value(): TypeValue {
    StructType(TestStruct_C, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), TestStruct_A_type_value()))
}
function {:inline 1} $TestStruct_C_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, TestStruct_C_val))
        && $TestStruct_A_is_well_formed(SelectField(__this, TestStruct_C_b))
}

procedure {:inline 1} Pack_TestStruct_C(module_idx: int, func_idx: int, var_idx: int, code_idx: int, val: Value, b: Value) returns (_struct: Value)
{
    assume IsValidU64(val);
    assume $TestStruct_A_is_well_formed(b);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, val), b));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_TestStruct_C(_struct: Value) returns (val: Value, b: Value)
{
    assume is#Vector(_struct);
    val := SelectField(_struct, TestStruct_C_val);
    assume IsValidU64(val);
    b := SelectField(_struct, TestStruct_C_b);
    assume $TestStruct_A_is_well_formed(b);
}

const unique TestStruct_T: TypeName;
const TestStruct_T_x: FieldName;
axiom TestStruct_T_x == 0;
function TestStruct_T_type_value(): TypeValue {
    StructType(TestStruct_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
function {:inline 1} $TestStruct_T_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, TestStruct_T_x))
}

procedure {:inline 1} Pack_TestStruct_T(module_idx: int, func_idx: int, var_idx: int, code_idx: int, x: Value) returns (_struct: Value)
{
    assume IsValidU64(x);
    _struct := Vector(ExtendValueArray(EmptyValueArray, x));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
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

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $TestStruct_A_is_well_formed(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    assume $DebugTrackLocal(0, 0, 0, 252, a);
    assume $TestStruct_C_is_well_formed(c);
    __m := UpdateLocal(__m, __frame + 1, c);
    assume $DebugTrackLocal(0, 0, 1, 252, c);

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(0, 0, 2, 315, __ret0);
    __ret1 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(0, 0, 3, 315, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
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

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    assume $DebugTrackLocal(0, 1, 0, 351, a);

    // increase the local counter
    __local_counter := __local_counter + 20;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 5), TestStruct_T_type_value());
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 4, __tmp);
    assume $DebugTrackLocal(0, 1, 4, 528, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    __tmp := GetLocal(__m, __frame + 8);
    if (!b#Boolean(__tmp)) { goto Label_8; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    if (true) { assume $DebugTrackAbort(0, 1, 576); }
    goto Label_Abort;

Label_8:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __t11 := BorrowGlobal(GetLocal(__m, __frame + 10), TestStruct_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 1, 598);
      goto Label_Abort;
    }

    call t_ref1 := CopyOrMoveRef(__t11);
    assume $TestStruct_T_is_well_formed(Dereference(__m, t_ref1));
    assume $DebugTrackLocal(0, 1, 2, 589, Dereference(__m, t_ref1));

    call __t12 := CopyOrMoveRef(t_ref1);

    // unimplemented instruction: NoOp

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __t14 := BorrowGlobal(GetLocal(__m, __frame + 13), TestStruct_T_type_value());
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 1, 672);
      goto Label_Abort;
    }

    call t_ref2 := CopyOrMoveRef(__t14);
    assume $TestStruct_T_is_well_formed(Dereference(__m, t_ref2));
    assume $DebugTrackLocal(0, 1, 3, 663, Dereference(__m, t_ref2));

    call __t15 := CopyOrMoveRef(t_ref2);

    // unimplemented instruction: NoOp

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := MoveFrom(GetLocal(__m, __frame + 16), TestStruct_T_type_value());
    __m := UpdateLocal(__m, __frame + 17, __tmp);
    assume $TestStruct_T_is_well_formed(__t17);
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 1, 741);
      goto Label_Abort;
    }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 17));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(0, 1, 1, 737, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call MoveToSender(TestStruct_T_type_value(), GetLocal(__m, __frame + 18));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 1, 772);
      goto Label_Abort;
    }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    __ret0 := GetLocal(__m, __frame + 19);
    assume $DebugTrackLocal(0, 1, 5, 808, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
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

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    assume $DebugTrackLocal(0, 2, 0, 835, a);

    // increase the local counter
    __local_counter := __local_counter + 24;

    // bytecode translation starts here
    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Pack_TestStruct_B(0, 2, 2, 1084, GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(0, 2, 2, 1076, __tmp);

    goto Label_11;

Label_7:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := Pack_TestStruct_B(0, 2, 2, 1150, GetLocal(__m, __frame + 10), GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(0, 2, 2, 1142, __tmp);

Label_11:
    call __t13 := BorrowLoc(__frame + 2, TestStruct_B_type_value());

    call var_b_ref := CopyOrMoveRef(__t13);
    assume $TestStruct_B_is_well_formed(Dereference(__m, var_b_ref));
    assume $DebugTrackLocal(0, 2, 3, 1198, Dereference(__m, var_b_ref));

    call __t14 := CopyOrMoveRef(var_b_ref);

    call __t15 := BorrowField(__t14, TestStruct_B_val);

    call b_val_ref := CopyOrMoveRef(__t15);
    assume IsValidU64(Dereference(__m, b_val_ref));
    assume $DebugTrackLocal(0, 2, 4, 1226, Dereference(__m, b_val_ref));

    call __t16 := CopyOrMoveRef(b_val_ref);

    call __tmp := ReadRef(__t16);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 17));
    __m := UpdateLocal(__m, __frame + 5, __tmp);
    assume $DebugTrackLocal(0, 2, 5, 1268, __tmp);

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

    if (true) { assume $DebugTrackAbort(0, 2, 1328); }
    goto Label_Abort;

Label_26:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    __ret0 := GetLocal(__m, __frame + 23);
    assume $DebugTrackLocal(0, 2, 6, 1341, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
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

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Address(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    assume $DebugTrackLocal(0, 3, 0, 1372, a);

    // increase the local counter
    __local_counter := __local_counter + 16;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Pack_TestStruct_B(0, 3, 2, 1513, GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(0, 3, 2, 1509, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8, __t9 := Unpack_TestStruct_B(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __t8);
    __m := UpdateLocal(__m, __frame + 9, __t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(0, 3, 1, 1559, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(0, 3, 3, 1555, __tmp);

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

    if (true) { assume $DebugTrackAbort(0, 3, 1610); }
    goto Label_Abort;

Label_15:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    __ret0 := GetLocal(__m, __frame + 15);
    assume $DebugTrackLocal(0, 3, 4, 1622, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestStruct_try_unpack_verify (a: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestStruct_try_unpack(a);
}
