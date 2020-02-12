

// ** synthetics of module TestSpecs



// ** structs of module TestSpecs

const unique TestSpecs_T: TypeName;
const TestSpecs_T_value: FieldName;
axiom TestSpecs_T_value == 0;
function TestSpecs_T_type_value(): TypeValue {
    StructType(TestSpecs_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
function {:inline 1} $TestSpecs_T_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, TestSpecs_T_value))
}

procedure {:inline 1} Pack_TestSpecs_T(module_idx: int, func_idx: int, var_idx: int, code_idx: int, value: Value) returns (_struct: Value)
{
    assume IsValidU64(value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, value));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
}

procedure {:inline 1} Unpack_TestSpecs_T(_struct: Value) returns (value: Value)
{
    assume is#Vector(_struct);
    value := SelectField(_struct, TestSpecs_T_value);
    assume IsValidU64(value);
}



// ** functions of module TestSpecs

procedure {:inline 1} TestSpecs_value (ref: Reference) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, SelectField(Dereference(__m, ref), TestSpecs_T_value))));
{
    // declare local variables
    var __t1: Reference; // ReferenceType(TestSpecs_T_type_value())
    var __t2: Reference; // ReferenceType(IntegerType())
    var __t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $TestSpecs_T_is_well_formed(Dereference(__m, ref)) && IsValidReferenceParameter(__m, __local_counter, ref);
    assume $TestSpecs_T_is_well_formed(Dereference(__m, ref));
    assume $DebugTrackLocal(0, 0, 0, 66, Dereference(__m, ref));

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(ref);

    call __t2 := BorrowField(__t1, TestSpecs_T_value);

    call __tmp := ReadRef(__t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(0, 0, 1, 141, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestSpecs_value_verify (ref: Reference) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestSpecs_value(ref);
}
