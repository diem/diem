

// ** structs of module TestSpecs

const unique TestSpecs_T: TypeName;
const TestSpecs_T_value: FieldName;
axiom TestSpecs_T_value == 0;
function TestSpecs_T_type_value(): TypeValue {
    StructType(TestSpecs_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}

procedure {:inline 1} Pack_TestSpecs_T(v0: Value) returns (v: Value)
{
    assume is#Integer(v0);
    v := Vector(ExtendValueArray(EmptyValueArray, v0));

}

procedure {:inline 1} Unpack_TestSpecs_T(v: Value) returns (v0: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, TestSpecs_T_value);
}



// ** functions of module TestSpecs

procedure {:inline 1} TestSpecs_value (arg0: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures b#Boolean(Boolean((ret0) == (SelectField(Dereference(m, arg0), TestSpecs_T_value))));
{
    // declare local variables
    var t0: Reference; // ReferenceType(TestSpecs_T_type_value())
    var t1: Reference; // ReferenceType(TestSpecs_T_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);

    old_size := local_counter;
    local_counter := local_counter + 4;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, TestSpecs_T_value);

    call tmp := ReadRef(t2);
    assume is#Integer(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestSpecs_value_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := TestSpecs_value(arg0);
}
