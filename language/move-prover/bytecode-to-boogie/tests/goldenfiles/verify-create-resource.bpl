

// ** structs of module TestSpecs

const unique TestSpecs_R: TypeName;
const TestSpecs_R_x: FieldName;
axiom TestSpecs_R_x == 0;
function TestSpecs_R_type_value(): TypeValue {
    StructType(TestSpecs_R, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_TestSpecs_R(x: Value) returns (_struct: Value)
{
    assume IsValidU64(x);
    _struct := Vector(ExtendValueArray(EmptyValueArray, x));
}

procedure {:inline 1} Unpack_TestSpecs_R(_struct: Value) returns (x: Value)
{
    assume is#Vector(_struct);
    x := SelectField(_struct, TestSpecs_R_x);
    assume IsValidU64(x);
}



// ** functions of module TestSpecs

procedure {:inline 1} TestSpecs_create_resource () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, TestSpecs_R_type_value(), a#Address(Address(TxnSenderAddress(__txn)))));
ensures old(!(b#Boolean(ExistsResource(__m, TestSpecs_R_type_value(), a#Address(Address(TxnSenderAddress(__txn))))))) ==> !__abort_flag;
ensures old(b#Boolean(ExistsResource(__m, TestSpecs_R_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))) ==> __abort_flag;

{
    // declare local variables
    var __t0: Value; // AddressType()
    var __t1: Value; // BooleanType()
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // TestSpecs_R_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 0), TestSpecs_R_type_value());
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __tmp := GetLocal(__m, __frame + 1);
    if (!b#Boolean(__tmp)) { goto Label_5; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    goto Label_Abort;

Label_5:
    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Pack_TestSpecs_R(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call MoveToSender(TestSpecs_R_type_value(), GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 0, 264);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestSpecs_create_resource_verify () returns ()
{
    call InitVerification();
    call TestSpecs_create_resource();
}

procedure {:inline 1} TestSpecs_create_resource_error () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, TestSpecs_R_type_value(), a#Address(Address(TxnSenderAddress(__txn)))));
ensures old(!(b#Boolean(ExistsResource(__m, TestSpecs_R_type_value(), a#Address(Address(TxnSenderAddress(__txn))))))) ==> !__abort_flag;
ensures old(b#Boolean(ExistsResource(__m, TestSpecs_R_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))) ==> __abort_flag;

{
    // declare local variables
    var __t0: Value; // AddressType()
    var __t1: Value; // BooleanType()
    var __t2: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 0), TestSpecs_R_type_value());
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __tmp := GetLocal(__m, __frame + 1);
    if (!b#Boolean(__tmp)) { goto Label_5; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    goto Label_Abort;

Label_5:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestSpecs_create_resource_error_verify () returns ()
{
    call InitVerification();
    call TestSpecs_create_resource_error();
}
