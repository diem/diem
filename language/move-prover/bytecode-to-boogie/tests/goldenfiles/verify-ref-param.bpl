

// ** structs of module TestSpecs

const unique TestSpecs_T: TypeName;
const TestSpecs_T_value: FieldName;
axiom TestSpecs_T_value == 0;
function TestSpecs_T_type_value(): TypeValue {
    StructType(TestSpecs_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_TestSpecs_T(value: Value) returns (_struct: Value)
{
    assume IsValidU64(value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, value));
}

procedure {:inline 1} Unpack_TestSpecs_T(_struct: Value) returns (value: Value)
{
    assume is#Vector(_struct);
    value := SelectField(_struct, TestSpecs_T_value);
    assume IsValidU64(value);
}



// ** functions of module TestSpecs

procedure {:inline 1} TestSpecs_value (ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean((ret0) == (SelectField(Dereference(__m, ref), TestSpecs_T_value))));
{
    // declare local variables
    var t1: Reference; // ReferenceType(TestSpecs_T_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume is#Vector(Dereference(__m, ref));
    assume IsValidReferenceParameter(__m, __frame, ref);

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(ref);

    call t2 := BorrowField(t1, TestSpecs_T_value);

    call __tmp := ReadRef(t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure TestSpecs_value_verify (ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := TestSpecs_value(ref);
}
