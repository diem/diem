

// ** structs of module TestSpecs

const unique TestSpecs_T: TypeName;
const TestSpecs_T_value: FieldName;
axiom TestSpecs_T_value == 0;
function TestSpecs_T_type_value(): TypeValue {
    StructType(TestSpecs_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_TestSpecs_T(value: Value) returns (_struct: Value)
{
    assume IsValidInteger(value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, value));

}

procedure {:inline 1} Unpack_TestSpecs_T(_struct: Value) returns (value: Value)
{
    assume is#Vector(_struct);
    value := SelectField(_struct, TestSpecs_T_value);
    assume IsValidInteger(value);
}



// ** functions of module TestSpecs

procedure {:inline 1} TestSpecs_value (ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures b#Boolean(Boolean((ret0) == (SelectField(Dereference(m, ref), TestSpecs_T_value))));
{
    // declare local variables
    var t1: Reference; // ReferenceType(TestSpecs_T_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, ref));
    assume IsValidReferenceParameter(m, local_counter, ref);

    old_size := local_counter;
    local_counter := local_counter + 4;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(ref);

    call t2 := BorrowField(t1, TestSpecs_T_value);

    call tmp := ReadRef(t2);
    assume IsValidInteger(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestSpecs_value_verify (ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := TestSpecs_value(ref);
}
