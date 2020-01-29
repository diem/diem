

// ** structs of module U64Util



// ** functions of module U64Util

procedure {:inline 1} U64Util_u64_to_bytes (i: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);



// ** structs of module AddressUtil



// ** functions of module AddressUtil

procedure {:inline 1} AddressUtil_address_to_bytes (addr: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);



// ** structs of module BytearrayUtil



// ** functions of module BytearrayUtil

procedure {:inline 1} BytearrayUtil_bytearray_concat (data1: Value, data2: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);



// ** structs of module Hash



// ** functions of module Hash

procedure {:inline 1} Hash_sha2_256 (data: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);

procedure {:inline 1} Hash_sha3_256 (data: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);



// ** structs of module Signature



// ** functions of module Signature

procedure {:inline 1} Signature_ed25519_verify (signature: Value, public_key: Value, message: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);

procedure {:inline 1} Signature_ed25519_threshold_verify (bitmap: Value, signature: Value, public_key: Value, message: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(__m, __txn);



// ** structs of module GasSchedule

const unique GasSchedule_Cost: TypeName;
const GasSchedule_Cost_cpu: FieldName;
axiom GasSchedule_Cost_cpu == 0;
const GasSchedule_Cost_storage: FieldName;
axiom GasSchedule_Cost_storage == 1;
function GasSchedule_Cost_type_value(): TypeValue {
    StructType(GasSchedule_Cost, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), IntegerType()))
}
procedure {:inline 1} Pack_GasSchedule_Cost(cpu: Value, storage: Value) returns (_struct: Value)
{
    assume IsValidU64(cpu);
    assume IsValidU64(storage);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, cpu), storage));
}

procedure {:inline 1} Unpack_GasSchedule_Cost(_struct: Value) returns (cpu: Value, storage: Value)
{
    assume is#Vector(_struct);
    cpu := SelectField(_struct, GasSchedule_Cost_cpu);
    assume IsValidU64(cpu);
    storage := SelectField(_struct, GasSchedule_Cost_storage);
    assume IsValidU64(storage);
}

const unique GasSchedule_T: TypeName;
const GasSchedule_T_instruction_schedule: FieldName;
axiom GasSchedule_T_instruction_schedule == 0;
const GasSchedule_T_native_schedule: FieldName;
axiom GasSchedule_T_native_schedule == 1;
function GasSchedule_T_type_value(): TypeValue {
    StructType(GasSchedule_T, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, Vector_T_type_value(GasSchedule_Cost_type_value())), Vector_T_type_value(GasSchedule_Cost_type_value())))
}
procedure {:inline 1} Pack_GasSchedule_T(instruction_schedule: Value, native_schedule: Value) returns (_struct: Value)
{
    assume is#Vector(instruction_schedule);
    assume is#Vector(native_schedule);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, instruction_schedule), native_schedule));
}

procedure {:inline 1} Unpack_GasSchedule_T(_struct: Value) returns (instruction_schedule: Value, native_schedule: Value)
{
    assume is#Vector(_struct);
    instruction_schedule := SelectField(_struct, GasSchedule_T_instruction_schedule);
    assume is#Vector(instruction_schedule);
    native_schedule := SelectField(_struct, GasSchedule_T_native_schedule);
    assume is#Vector(native_schedule);
}



// ** functions of module GasSchedule

procedure {:inline 1} GasSchedule_initialize (gas_schedule: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Value; // AddressType()
    var t3: Value; // BooleanType()
    var t4: Value; // BooleanType()
    var t5: Value; // IntegerType()
    var t6: Value; // GasSchedule_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments
    assume is#Vector(gas_schedule);
    __m := UpdateLocal(__m, __frame + 0, gas_schedule);

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 1), GetLocal(__m, __frame + 2)));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __tmp := GetLocal(__m, __frame + 4);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    goto Label_Abort;

Label_7:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call MoveToSender(GasSchedule_T_type_value(), GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure GasSchedule_initialize_verify (gas_schedule: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call GasSchedule_initialize(gas_schedule);
}

procedure {:inline 1} GasSchedule_instruction_table_size () returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(GasSchedule_T_type_value())
    var t1: Value; // IntegerType()
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(GasSchedule_T_type_value())
    var t4: Reference; // ReferenceType(GasSchedule_T_type_value())
    var t5: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 8;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call t3 := BorrowGlobal(GetLocal(__m, __frame + 2), GasSchedule_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t0 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, GasSchedule_T_instruction_schedule);

    call t6 := Vector_length(GasSchedule_Cost_type_value(), t5);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(t6);

    __m := UpdateLocal(__m, __frame + 6, t6);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    ret0 := GetLocal(__m, __frame + 7);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure GasSchedule_instruction_table_size_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := GasSchedule_instruction_table_size();
}

procedure {:inline 1} GasSchedule_native_table_size () returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(GasSchedule_T_type_value())
    var t1: Value; // IntegerType()
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(GasSchedule_T_type_value())
    var t4: Reference; // ReferenceType(GasSchedule_T_type_value())
    var t5: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 8;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call t3 := BorrowGlobal(GetLocal(__m, __frame + 2), GasSchedule_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t0 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, GasSchedule_T_native_schedule);

    call t6 := Vector_length(GasSchedule_Cost_type_value(), t5);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(t6);

    __m := UpdateLocal(__m, __frame + 6, t6);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    ret0 := GetLocal(__m, __frame + 7);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure GasSchedule_native_table_size_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := GasSchedule_native_table_size();
}



// ** structs of module ValidatorConfig

const unique ValidatorConfig_Config: TypeName;
const ValidatorConfig_Config_consensus_pubkey: FieldName;
axiom ValidatorConfig_Config_consensus_pubkey == 0;
const ValidatorConfig_Config_validator_network_signing_pubkey: FieldName;
axiom ValidatorConfig_Config_validator_network_signing_pubkey == 1;
const ValidatorConfig_Config_validator_network_identity_pubkey: FieldName;
axiom ValidatorConfig_Config_validator_network_identity_pubkey == 2;
const ValidatorConfig_Config_validator_network_address: FieldName;
axiom ValidatorConfig_Config_validator_network_address == 3;
const ValidatorConfig_Config_fullnodes_network_identity_pubkey: FieldName;
axiom ValidatorConfig_Config_fullnodes_network_identity_pubkey == 4;
const ValidatorConfig_Config_fullnodes_network_address: FieldName;
axiom ValidatorConfig_Config_fullnodes_network_address == 5;
function ValidatorConfig_Config_type_value(): TypeValue {
    StructType(ValidatorConfig_Config, ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, ByteArrayType()), ByteArrayType()), ByteArrayType()), ByteArrayType()), ByteArrayType()), ByteArrayType()))
}
procedure {:inline 1} Pack_ValidatorConfig_Config(consensus_pubkey: Value, validator_network_signing_pubkey: Value, validator_network_identity_pubkey: Value, validator_network_address: Value, fullnodes_network_identity_pubkey: Value, fullnodes_network_address: Value) returns (_struct: Value)
{
    assume is#ByteArray(consensus_pubkey);
    assume is#ByteArray(validator_network_signing_pubkey);
    assume is#ByteArray(validator_network_identity_pubkey);
    assume is#ByteArray(validator_network_address);
    assume is#ByteArray(fullnodes_network_identity_pubkey);
    assume is#ByteArray(fullnodes_network_address);
    _struct := Vector(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(EmptyValueArray, consensus_pubkey), validator_network_signing_pubkey), validator_network_identity_pubkey), validator_network_address), fullnodes_network_identity_pubkey), fullnodes_network_address));
}

procedure {:inline 1} Unpack_ValidatorConfig_Config(_struct: Value) returns (consensus_pubkey: Value, validator_network_signing_pubkey: Value, validator_network_identity_pubkey: Value, validator_network_address: Value, fullnodes_network_identity_pubkey: Value, fullnodes_network_address: Value)
{
    assume is#Vector(_struct);
    consensus_pubkey := SelectField(_struct, ValidatorConfig_Config_consensus_pubkey);
    assume is#ByteArray(consensus_pubkey);
    validator_network_signing_pubkey := SelectField(_struct, ValidatorConfig_Config_validator_network_signing_pubkey);
    assume is#ByteArray(validator_network_signing_pubkey);
    validator_network_identity_pubkey := SelectField(_struct, ValidatorConfig_Config_validator_network_identity_pubkey);
    assume is#ByteArray(validator_network_identity_pubkey);
    validator_network_address := SelectField(_struct, ValidatorConfig_Config_validator_network_address);
    assume is#ByteArray(validator_network_address);
    fullnodes_network_identity_pubkey := SelectField(_struct, ValidatorConfig_Config_fullnodes_network_identity_pubkey);
    assume is#ByteArray(fullnodes_network_identity_pubkey);
    fullnodes_network_address := SelectField(_struct, ValidatorConfig_Config_fullnodes_network_address);
    assume is#ByteArray(fullnodes_network_address);
}

const unique ValidatorConfig_T: TypeName;
const ValidatorConfig_T_config: FieldName;
axiom ValidatorConfig_T_config == 0;
function ValidatorConfig_T_type_value(): TypeValue {
    StructType(ValidatorConfig_T, ExtendTypeValueArray(EmptyTypeValueArray, ValidatorConfig_Config_type_value()))
}
procedure {:inline 1} Pack_ValidatorConfig_T(config: Value) returns (_struct: Value)
{
    assume is#Vector(config);
    _struct := Vector(ExtendValueArray(EmptyValueArray, config));
}

procedure {:inline 1} Unpack_ValidatorConfig_T(_struct: Value) returns (config: Value)
{
    assume is#Vector(_struct);
    config := SelectField(_struct, ValidatorConfig_T_config);
    assume is#Vector(config);
}



// ** functions of module ValidatorConfig

procedure {:inline 1} ValidatorConfig_has (addr: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 3;

    // process and type check arguments
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 1), ValidatorConfig_T_type_value());
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    ret0 := GetLocal(__m, __frame + 2);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_has_verify (addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := ValidatorConfig_has(addr);
}

procedure {:inline 1} ValidatorConfig_config (addr: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t4: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t5: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t6: Value; // ValidatorConfig_Config_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call t3 := BorrowGlobal(GetLocal(__m, __frame + 2), ValidatorConfig_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, ValidatorConfig_T_config);

    call __tmp := ReadRef(t5);
    assume is#Vector(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    ret0 := GetLocal(__m, __frame + 6);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_config_verify (addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := ValidatorConfig_config(addr);
}

procedure {:inline 1} ValidatorConfig_consensus_pubkey (config_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume is#Vector(Dereference(__m, config_ref));
    assume IsValidReferenceParameter(__m, __frame, config_ref);

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(config_ref);

    call t2 := BorrowField(t1, ValidatorConfig_Config_consensus_pubkey);

    call __tmp := ReadRef(t2);
    assume is#ByteArray(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_consensus_pubkey_verify (config_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := ValidatorConfig_consensus_pubkey(config_ref);
}

procedure {:inline 1} ValidatorConfig_validator_network_signing_pubkey (config_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume is#Vector(Dereference(__m, config_ref));
    assume IsValidReferenceParameter(__m, __frame, config_ref);

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(config_ref);

    call t2 := BorrowField(t1, ValidatorConfig_Config_validator_network_signing_pubkey);

    call __tmp := ReadRef(t2);
    assume is#ByteArray(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_validator_network_signing_pubkey_verify (config_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := ValidatorConfig_validator_network_signing_pubkey(config_ref);
}

procedure {:inline 1} ValidatorConfig_validator_network_identity_pubkey (config_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume is#Vector(Dereference(__m, config_ref));
    assume IsValidReferenceParameter(__m, __frame, config_ref);

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(config_ref);

    call t2 := BorrowField(t1, ValidatorConfig_Config_validator_network_identity_pubkey);

    call __tmp := ReadRef(t2);
    assume is#ByteArray(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_validator_network_identity_pubkey_verify (config_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := ValidatorConfig_validator_network_identity_pubkey(config_ref);
}

procedure {:inline 1} ValidatorConfig_validator_network_address (config_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume is#Vector(Dereference(__m, config_ref));
    assume IsValidReferenceParameter(__m, __frame, config_ref);

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(config_ref);

    call t2 := BorrowField(t1, ValidatorConfig_Config_validator_network_address);

    call __tmp := ReadRef(t2);
    assume is#ByteArray(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_validator_network_address_verify (config_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := ValidatorConfig_validator_network_address(config_ref);
}

procedure {:inline 1} ValidatorConfig_fullnodes_network_identity_pubkey (config_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume is#Vector(Dereference(__m, config_ref));
    assume IsValidReferenceParameter(__m, __frame, config_ref);

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(config_ref);

    call t2 := BorrowField(t1, ValidatorConfig_Config_fullnodes_network_identity_pubkey);

    call __tmp := ReadRef(t2);
    assume is#ByteArray(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_fullnodes_network_identity_pubkey_verify (config_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := ValidatorConfig_fullnodes_network_identity_pubkey(config_ref);
}

procedure {:inline 1} ValidatorConfig_fullnodes_network_address (config_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume is#Vector(Dereference(__m, config_ref));
    assume IsValidReferenceParameter(__m, __frame, config_ref);

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(config_ref);

    call t2 := BorrowField(t1, ValidatorConfig_Config_fullnodes_network_address);

    call __tmp := ReadRef(t2);
    assume is#ByteArray(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_fullnodes_network_address_verify (config_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := ValidatorConfig_fullnodes_network_address(config_ref);
}

procedure {:inline 1} ValidatorConfig_register_candidate_validator (consensus_pubkey: Value, validator_network_signing_pubkey: Value, validator_network_identity_pubkey: Value, validator_network_address: Value, fullnodes_network_identity_pubkey: Value, fullnodes_network_address: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t6: Value; // ByteArrayType()
    var t7: Value; // ByteArrayType()
    var t8: Value; // ByteArrayType()
    var t9: Value; // ByteArrayType()
    var t10: Value; // ByteArrayType()
    var t11: Value; // ByteArrayType()
    var t12: Value; // ValidatorConfig_Config_type_value()
    var t13: Value; // ValidatorConfig_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 14;

    // process and type check arguments
    assume is#ByteArray(consensus_pubkey);
    __m := UpdateLocal(__m, __frame + 0, consensus_pubkey);
    assume is#ByteArray(validator_network_signing_pubkey);
    __m := UpdateLocal(__m, __frame + 1, validator_network_signing_pubkey);
    assume is#ByteArray(validator_network_identity_pubkey);
    __m := UpdateLocal(__m, __frame + 2, validator_network_identity_pubkey);
    assume is#ByteArray(validator_network_address);
    __m := UpdateLocal(__m, __frame + 3, validator_network_address);
    assume is#ByteArray(fullnodes_network_identity_pubkey);
    __m := UpdateLocal(__m, __frame + 4, fullnodes_network_identity_pubkey);
    assume is#ByteArray(fullnodes_network_address);
    __m := UpdateLocal(__m, __frame + 5, fullnodes_network_address);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := Pack_ValidatorConfig_Config(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8), GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10), GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Pack_ValidatorConfig_T(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call MoveToSender(ValidatorConfig_T_type_value(), GetLocal(__m, __frame + 13));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure ValidatorConfig_register_candidate_validator_verify (consensus_pubkey: Value, validator_network_signing_pubkey: Value, validator_network_identity_pubkey: Value, validator_network_address: Value, fullnodes_network_identity_pubkey: Value, fullnodes_network_address: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ValidatorConfig_register_candidate_validator(consensus_pubkey, validator_network_signing_pubkey, validator_network_identity_pubkey, validator_network_address, fullnodes_network_identity_pubkey, fullnodes_network_address);
}

procedure {:inline 1} ValidatorConfig_rotate_consensus_pubkey (consensus_pubkey: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t2: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t3: Reference; // ReferenceType(ByteArrayType())
    var t4: Value; // AddressType()
    var t5: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t6: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t7: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t8: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t9: Reference; // ReferenceType(ByteArrayType())
    var t10: Value; // ByteArrayType()
    var t11: Reference; // ReferenceType(ByteArrayType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 12;

    // process and type check arguments
    assume is#ByteArray(consensus_pubkey);
    __m := UpdateLocal(__m, __frame + 0, consensus_pubkey);

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call t5 := BorrowGlobal(GetLocal(__m, __frame + 4), ValidatorConfig_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, ValidatorConfig_T_config);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, ValidatorConfig_Config_consensus_pubkey);

    call t3 := CopyOrMoveRef(t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call t11 := CopyOrMoveRef(t3);

    call WriteRef(t11, GetLocal(__m, __frame + 10));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure ValidatorConfig_rotate_consensus_pubkey_verify (consensus_pubkey: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ValidatorConfig_rotate_consensus_pubkey(consensus_pubkey);
}

procedure {:inline 1} ValidatorConfig_rotate_validator_network_identity_pubkey (validator_network_identity_pubkey: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t2: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t3: Reference; // ReferenceType(ByteArrayType())
    var t4: Value; // AddressType()
    var t5: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t6: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t7: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t8: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t9: Reference; // ReferenceType(ByteArrayType())
    var t10: Value; // ByteArrayType()
    var t11: Reference; // ReferenceType(ByteArrayType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 12;

    // process and type check arguments
    assume is#ByteArray(validator_network_identity_pubkey);
    __m := UpdateLocal(__m, __frame + 0, validator_network_identity_pubkey);

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call t5 := BorrowGlobal(GetLocal(__m, __frame + 4), ValidatorConfig_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, ValidatorConfig_T_config);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, ValidatorConfig_Config_validator_network_identity_pubkey);

    call t3 := CopyOrMoveRef(t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call t11 := CopyOrMoveRef(t3);

    call WriteRef(t11, GetLocal(__m, __frame + 10));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure ValidatorConfig_rotate_validator_network_identity_pubkey_verify (validator_network_identity_pubkey: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ValidatorConfig_rotate_validator_network_identity_pubkey(validator_network_identity_pubkey);
}

procedure {:inline 1} ValidatorConfig_rotate_validator_network_address (validator_network_address: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t2: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t3: Reference; // ReferenceType(ByteArrayType())
    var t4: Value; // AddressType()
    var t5: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t6: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t7: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t8: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t9: Reference; // ReferenceType(ByteArrayType())
    var t10: Value; // ByteArrayType()
    var t11: Reference; // ReferenceType(ByteArrayType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 12;

    // process and type check arguments
    assume is#ByteArray(validator_network_address);
    __m := UpdateLocal(__m, __frame + 0, validator_network_address);

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call t5 := BorrowGlobal(GetLocal(__m, __frame + 4), ValidatorConfig_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, ValidatorConfig_T_config);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, ValidatorConfig_Config_validator_network_address);

    call t3 := CopyOrMoveRef(t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call t11 := CopyOrMoveRef(t3);

    call WriteRef(t11, GetLocal(__m, __frame + 10));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure ValidatorConfig_rotate_validator_network_address_verify (validator_network_address: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ValidatorConfig_rotate_validator_network_address(validator_network_address);
}



// ** structs of module LibraCoin

const unique LibraCoin_T: TypeName;
const LibraCoin_T_value: FieldName;
axiom LibraCoin_T_value == 0;
function LibraCoin_T_type_value(): TypeValue {
    StructType(LibraCoin_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_LibraCoin_T(value: Value) returns (_struct: Value)
{
    assume IsValidU64(value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, value));
}

procedure {:inline 1} Unpack_LibraCoin_T(_struct: Value) returns (value: Value)
{
    assume is#Vector(_struct);
    value := SelectField(_struct, LibraCoin_T_value);
    assume IsValidU64(value);
}

const unique LibraCoin_MintCapability: TypeName;
const LibraCoin_MintCapability__dummy: FieldName;
axiom LibraCoin_MintCapability__dummy == 0;
function LibraCoin_MintCapability_type_value(): TypeValue {
    StructType(LibraCoin_MintCapability, ExtendTypeValueArray(EmptyTypeValueArray, BooleanType()))
}
procedure {:inline 1} Pack_LibraCoin_MintCapability(_dummy: Value) returns (_struct: Value)
{
    assume is#Boolean(_dummy);
    _struct := Vector(ExtendValueArray(EmptyValueArray, _dummy));
}

procedure {:inline 1} Unpack_LibraCoin_MintCapability(_struct: Value) returns (_dummy: Value)
{
    assume is#Vector(_struct);
    _dummy := SelectField(_struct, LibraCoin_MintCapability__dummy);
    assume is#Boolean(_dummy);
}

const unique LibraCoin_MarketCap: TypeName;
const LibraCoin_MarketCap_total_value: FieldName;
axiom LibraCoin_MarketCap_total_value == 0;
function LibraCoin_MarketCap_type_value(): TypeValue {
    StructType(LibraCoin_MarketCap, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_LibraCoin_MarketCap(total_value: Value) returns (_struct: Value)
{
    assume IsValidU128(total_value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, total_value));
}

procedure {:inline 1} Unpack_LibraCoin_MarketCap(_struct: Value) returns (total_value: Value)
{
    assume is#Vector(_struct);
    total_value := SelectField(_struct, LibraCoin_MarketCap_total_value);
    assume IsValidU128(total_value);
}



// ** functions of module LibraCoin

procedure {:inline 1} LibraCoin_mint_with_default_capability (amount: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(LibraCoin_MintCapability_type_value())
    var t4: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 0, amount);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraCoin_MintCapability_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t4 := LibraCoin_mint(GetLocal(__m, __frame + 1), t3);
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t4);

    __m := UpdateLocal(__m, __frame + 4, t4);

    ret0 := GetLocal(__m, __frame + 4);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_mint_with_default_capability_verify (amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_mint_with_default_capability(amount);
}

procedure {:inline 1} LibraCoin_mint (value: Value, capability: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // BooleanType()
    var t8: Value; // BooleanType()
    var t9: Value; // IntegerType()
    var t10: Value; // AddressType()
    var t11: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var t12: Reference; // ReferenceType(IntegerType())
    var t13: Reference; // ReferenceType(IntegerType())
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Value; // IntegerType()
    var t17: Value; // IntegerType()
    var t18: Reference; // ReferenceType(IntegerType())
    var t19: Value; // IntegerType()
    var t20: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 21;

    // process and type check arguments
    assume IsValidU64(value);
    __m := UpdateLocal(__m, __frame + 0, value);
    assume is#Vector(Dereference(__m, capability));
    assume IsValidReferenceParameter(__m, __frame, capability);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(1000000000);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := LdConst(1000000);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Le(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    __tmp := GetLocal(__m, __frame + 8);
    if (!b#Boolean(__tmp)) { goto Label_9; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    goto Label_Abort;

Label_9:
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call t11 := BorrowGlobal(GetLocal(__m, __frame + 10), LibraCoin_MarketCap_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t12 := BorrowField(t11, LibraCoin_MarketCap_total_value);

    call t2 := CopyOrMoveRef(t12);

    call t13 := CopyOrMoveRef(t2);

    call __tmp := ReadRef(t13);
    assume IsValidU128(__tmp);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := CastU128(GetLocal(__m, __frame + 15));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := AddU128(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 16));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call t18 := CopyOrMoveRef(t2);

    call WriteRef(t18, GetLocal(__m, __frame + 17));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __tmp := Pack_LibraCoin_T(GetLocal(__m, __frame + 19));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    ret0 := GetLocal(__m, __frame + 20);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_mint_verify (value: Value, capability: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_mint(value, capability);
}

procedure {:inline 1} LibraCoin_initialize () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Value; // BooleanType()
    var t3: Value; // BooleanType()
    var t4: Value; // IntegerType()
    var t5: Value; // BooleanType()
    var t6: Value; // LibraCoin_MintCapability_type_value()
    var t7: Value; // IntegerType()
    var t8: Value; // LibraCoin_MarketCap_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 9;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 0), GetLocal(__m, __frame + 1)));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __tmp := GetLocal(__m, __frame + 3);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    goto Label_Abort;

Label_7:
    call __tmp := LdTrue();
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Pack_LibraCoin_MintCapability(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call MoveToSender(LibraCoin_MintCapability_type_value(), GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := Pack_LibraCoin_MarketCap(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call MoveToSender(LibraCoin_MarketCap_type_value(), GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraCoin_initialize_verify () returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraCoin_initialize();
}

procedure {:inline 1} LibraCoin_market_cap () returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
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

    // bytecode translation starts here
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call t1 := BorrowGlobal(GetLocal(__m, __frame + 0), LibraCoin_MarketCap_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t2 := BorrowField(t1, LibraCoin_MarketCap_total_value);

    call __tmp := ReadRef(t2);
    assume IsValidU128(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_market_cap_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_market_cap();
}

procedure {:inline 1} LibraCoin_zero () returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 2;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := Pack_LibraCoin_T(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    ret0 := GetLocal(__m, __frame + 1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_zero_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_zero();
}

procedure {:inline 1} LibraCoin_value (coin_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(LibraCoin_T_type_value())
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
    assume is#Vector(Dereference(__m, coin_ref));
    assume IsValidReferenceParameter(__m, __frame, coin_ref);

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(coin_ref);

    call t2 := BorrowField(t1, LibraCoin_T_value);

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

procedure LibraCoin_value_verify (coin_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_value(coin_ref);
}

procedure {:inline 1} LibraCoin_split (coin: Value, amount: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Value; // LibraCoin_T_type_value()
    var t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t4: Value; // IntegerType()
    var t5: Value; // LibraCoin_T_type_value()
    var t6: Value; // LibraCoin_T_type_value()
    var t7: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 8;

    // process and type check arguments
    assume is#Vector(coin);
    __m := UpdateLocal(__m, __frame + 0, coin);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);

    // bytecode translation starts here
    call t3 := BorrowLoc(__frame + 0);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call t5 := LibraCoin_withdraw(t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t5);

    __m := UpdateLocal(__m, __frame + 5, t5);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    ret0 := GetLocal(__m, __frame + 6);
    ret1 := GetLocal(__m, __frame + 7);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
}

procedure LibraCoin_split_verify (coin: Value, amount: Value) returns (ret0: Value, ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0, ret1 := LibraCoin_split(coin, amount);
}

procedure {:inline 1} LibraCoin_withdraw (coin_ref: Reference, amount: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t4: Reference; // ReferenceType(IntegerType())
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // BooleanType()
    var t9: Value; // BooleanType()
    var t10: Value; // IntegerType()
    var t11: Value; // IntegerType()
    var t12: Value; // IntegerType()
    var t13: Value; // IntegerType()
    var t14: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t15: Reference; // ReferenceType(IntegerType())
    var t16: Value; // IntegerType()
    var t17: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 18;

    // process and type check arguments
    assume is#Vector(Dereference(__m, coin_ref));
    assume IsValidReferenceParameter(__m, __frame, coin_ref);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(coin_ref);

    call t4 := BorrowField(t3, LibraCoin_T_value);

    call __tmp := ReadRef(t4);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := Ge(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __tmp := GetLocal(__m, __frame + 9);
    if (!b#Boolean(__tmp)) { goto Label_11; }

    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    goto Label_Abort;

Label_11:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 12));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call t14 := CopyOrMoveRef(coin_ref);

    call t15 := BorrowField(t14, LibraCoin_T_value);

    call WriteRef(t15, GetLocal(__m, __frame + 13));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := Pack_LibraCoin_T(GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    ret0 := GetLocal(__m, __frame + 17);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_withdraw_verify (coin_ref: Reference, amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_withdraw(coin_ref, amount);
}

procedure {:inline 1} LibraCoin_join (coin1: Value, coin2: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t3: Value; // LibraCoin_T_type_value()
    var t4: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume is#Vector(coin1);
    __m := UpdateLocal(__m, __frame + 0, coin1);
    assume is#Vector(coin2);
    __m := UpdateLocal(__m, __frame + 1, coin2);

    // bytecode translation starts here
    call t2 := BorrowLoc(__frame + 0);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call LibraCoin_deposit(t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    ret0 := GetLocal(__m, __frame + 4);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_join_verify (coin1: Value, coin2: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraCoin_join(coin1, coin2);
}

procedure {:inline 1} LibraCoin_deposit (coin_ref: Reference, check: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t5: Reference; // ReferenceType(IntegerType())
    var t6: Value; // IntegerType()
    var t7: Value; // LibraCoin_T_type_value()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // IntegerType()
    var t11: Value; // IntegerType()
    var t12: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t13: Reference; // ReferenceType(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 14;

    // process and type check arguments
    assume is#Vector(Dereference(__m, coin_ref));
    assume IsValidReferenceParameter(__m, __frame, coin_ref);
    assume is#Vector(check);
    __m := UpdateLocal(__m, __frame + 1, check);

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(coin_ref);

    call t5 := BorrowField(t4, LibraCoin_T_value);

    call __tmp := ReadRef(t5);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call t8 := Unpack_LibraCoin_T(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call t12 := CopyOrMoveRef(coin_ref);

    call t13 := BorrowField(t12, LibraCoin_T_value);

    call WriteRef(t13, GetLocal(__m, __frame + 11));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraCoin_deposit_verify (coin_ref: Reference, check: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraCoin_deposit(coin_ref, check);
}

procedure {:inline 1} LibraCoin_destroy_zero (coin: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // LibraCoin_T_type_value()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // BooleanType()
    var t7: Value; // BooleanType()
    var t8: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 9;

    // process and type check arguments
    assume is#Vector(coin);
    __m := UpdateLocal(__m, __frame + 0, coin);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call t3 := Unpack_LibraCoin_T(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5)));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __tmp := GetLocal(__m, __frame + 7);
    if (!b#Boolean(__tmp)) { goto Label_10; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    goto Label_Abort;

Label_10:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraCoin_destroy_zero_verify (coin: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraCoin_destroy_zero(coin);
}



// ** structs of module LibraAccount

const unique LibraAccount_T: TypeName;
const LibraAccount_T_authentication_key: FieldName;
axiom LibraAccount_T_authentication_key == 0;
const LibraAccount_T_balance: FieldName;
axiom LibraAccount_T_balance == 1;
const LibraAccount_T_delegated_key_rotation_capability: FieldName;
axiom LibraAccount_T_delegated_key_rotation_capability == 2;
const LibraAccount_T_delegated_withdrawal_capability: FieldName;
axiom LibraAccount_T_delegated_withdrawal_capability == 3;
const LibraAccount_T_received_events: FieldName;
axiom LibraAccount_T_received_events == 4;
const LibraAccount_T_sent_events: FieldName;
axiom LibraAccount_T_sent_events == 5;
const LibraAccount_T_sequence_number: FieldName;
axiom LibraAccount_T_sequence_number == 6;
const LibraAccount_T_event_generator: FieldName;
axiom LibraAccount_T_event_generator == 7;
axiom LibraAccount_T_type_value() == StructType(LibraAccount_T, ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, ByteArrayType()), LibraCoin_T_type_value()), BooleanType()), BooleanType()), LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())), LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())), IntegerType()), LibraAccount_EventHandleGenerator_type_value()));
procedure {:inline 1} Pack_LibraAccount_T(authentication_key: Value, balance: Value, delegated_key_rotation_capability: Value, delegated_withdrawal_capability: Value, received_events: Value, sent_events: Value, sequence_number: Value, event_generator: Value) returns (_struct: Value)
{
    assume is#ByteArray(authentication_key);
    assume is#Vector(balance);
    assume is#Boolean(delegated_key_rotation_capability);
    assume is#Boolean(delegated_withdrawal_capability);
    assume is#Vector(received_events);
    assume is#Vector(sent_events);
    assume IsValidU64(sequence_number);
    assume is#Vector(event_generator);
    _struct := Vector(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(ExtendValueArray(EmptyValueArray, authentication_key), balance), delegated_key_rotation_capability), delegated_withdrawal_capability), received_events), sent_events), sequence_number), event_generator));
}

procedure {:inline 1} Unpack_LibraAccount_T(_struct: Value) returns (authentication_key: Value, balance: Value, delegated_key_rotation_capability: Value, delegated_withdrawal_capability: Value, received_events: Value, sent_events: Value, sequence_number: Value, event_generator: Value)
{
    assume is#Vector(_struct);
    authentication_key := SelectField(_struct, LibraAccount_T_authentication_key);
    assume is#ByteArray(authentication_key);
    balance := SelectField(_struct, LibraAccount_T_balance);
    assume is#Vector(balance);
    delegated_key_rotation_capability := SelectField(_struct, LibraAccount_T_delegated_key_rotation_capability);
    assume is#Boolean(delegated_key_rotation_capability);
    delegated_withdrawal_capability := SelectField(_struct, LibraAccount_T_delegated_withdrawal_capability);
    assume is#Boolean(delegated_withdrawal_capability);
    received_events := SelectField(_struct, LibraAccount_T_received_events);
    assume is#Vector(received_events);
    sent_events := SelectField(_struct, LibraAccount_T_sent_events);
    assume is#Vector(sent_events);
    sequence_number := SelectField(_struct, LibraAccount_T_sequence_number);
    assume IsValidU64(sequence_number);
    event_generator := SelectField(_struct, LibraAccount_T_event_generator);
    assume is#Vector(event_generator);
}

const unique LibraAccount_WithdrawalCapability: TypeName;
const LibraAccount_WithdrawalCapability_account_address: FieldName;
axiom LibraAccount_WithdrawalCapability_account_address == 0;
function LibraAccount_WithdrawalCapability_type_value(): TypeValue {
    StructType(LibraAccount_WithdrawalCapability, ExtendTypeValueArray(EmptyTypeValueArray, AddressType()))
}
procedure {:inline 1} Pack_LibraAccount_WithdrawalCapability(account_address: Value) returns (_struct: Value)
{
    assume is#Address(account_address);
    _struct := Vector(ExtendValueArray(EmptyValueArray, account_address));
}

procedure {:inline 1} Unpack_LibraAccount_WithdrawalCapability(_struct: Value) returns (account_address: Value)
{
    assume is#Vector(_struct);
    account_address := SelectField(_struct, LibraAccount_WithdrawalCapability_account_address);
    assume is#Address(account_address);
}

const unique LibraAccount_KeyRotationCapability: TypeName;
const LibraAccount_KeyRotationCapability_account_address: FieldName;
axiom LibraAccount_KeyRotationCapability_account_address == 0;
function LibraAccount_KeyRotationCapability_type_value(): TypeValue {
    StructType(LibraAccount_KeyRotationCapability, ExtendTypeValueArray(EmptyTypeValueArray, AddressType()))
}
procedure {:inline 1} Pack_LibraAccount_KeyRotationCapability(account_address: Value) returns (_struct: Value)
{
    assume is#Address(account_address);
    _struct := Vector(ExtendValueArray(EmptyValueArray, account_address));
}

procedure {:inline 1} Unpack_LibraAccount_KeyRotationCapability(_struct: Value) returns (account_address: Value)
{
    assume is#Vector(_struct);
    account_address := SelectField(_struct, LibraAccount_KeyRotationCapability_account_address);
    assume is#Address(account_address);
}

const unique LibraAccount_SentPaymentEvent: TypeName;
const LibraAccount_SentPaymentEvent_amount: FieldName;
axiom LibraAccount_SentPaymentEvent_amount == 0;
const LibraAccount_SentPaymentEvent_payee: FieldName;
axiom LibraAccount_SentPaymentEvent_payee == 1;
const LibraAccount_SentPaymentEvent_metadata: FieldName;
axiom LibraAccount_SentPaymentEvent_metadata == 2;
function LibraAccount_SentPaymentEvent_type_value(): TypeValue {
    StructType(LibraAccount_SentPaymentEvent, ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), AddressType()), ByteArrayType()))
}
procedure {:inline 1} Pack_LibraAccount_SentPaymentEvent(amount: Value, payee: Value, metadata: Value) returns (_struct: Value)
{
    assume IsValidU64(amount);
    assume is#Address(payee);
    assume is#ByteArray(metadata);
    _struct := Vector(ExtendValueArray(ExtendValueArray(ExtendValueArray(EmptyValueArray, amount), payee), metadata));
}

procedure {:inline 1} Unpack_LibraAccount_SentPaymentEvent(_struct: Value) returns (amount: Value, payee: Value, metadata: Value)
{
    assume is#Vector(_struct);
    amount := SelectField(_struct, LibraAccount_SentPaymentEvent_amount);
    assume IsValidU64(amount);
    payee := SelectField(_struct, LibraAccount_SentPaymentEvent_payee);
    assume is#Address(payee);
    metadata := SelectField(_struct, LibraAccount_SentPaymentEvent_metadata);
    assume is#ByteArray(metadata);
}

const unique LibraAccount_ReceivedPaymentEvent: TypeName;
const LibraAccount_ReceivedPaymentEvent_amount: FieldName;
axiom LibraAccount_ReceivedPaymentEvent_amount == 0;
const LibraAccount_ReceivedPaymentEvent_payer: FieldName;
axiom LibraAccount_ReceivedPaymentEvent_payer == 1;
const LibraAccount_ReceivedPaymentEvent_metadata: FieldName;
axiom LibraAccount_ReceivedPaymentEvent_metadata == 2;
function LibraAccount_ReceivedPaymentEvent_type_value(): TypeValue {
    StructType(LibraAccount_ReceivedPaymentEvent, ExtendTypeValueArray(ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), AddressType()), ByteArrayType()))
}
procedure {:inline 1} Pack_LibraAccount_ReceivedPaymentEvent(amount: Value, payer: Value, metadata: Value) returns (_struct: Value)
{
    assume IsValidU64(amount);
    assume is#Address(payer);
    assume is#ByteArray(metadata);
    _struct := Vector(ExtendValueArray(ExtendValueArray(ExtendValueArray(EmptyValueArray, amount), payer), metadata));
}

procedure {:inline 1} Unpack_LibraAccount_ReceivedPaymentEvent(_struct: Value) returns (amount: Value, payer: Value, metadata: Value)
{
    assume is#Vector(_struct);
    amount := SelectField(_struct, LibraAccount_ReceivedPaymentEvent_amount);
    assume IsValidU64(amount);
    payer := SelectField(_struct, LibraAccount_ReceivedPaymentEvent_payer);
    assume is#Address(payer);
    metadata := SelectField(_struct, LibraAccount_ReceivedPaymentEvent_metadata);
    assume is#ByteArray(metadata);
}

const unique LibraAccount_EventHandleGenerator: TypeName;
const LibraAccount_EventHandleGenerator_counter: FieldName;
axiom LibraAccount_EventHandleGenerator_counter == 0;
function LibraAccount_EventHandleGenerator_type_value(): TypeValue {
    StructType(LibraAccount_EventHandleGenerator, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_LibraAccount_EventHandleGenerator(counter: Value) returns (_struct: Value)
{
    assume IsValidU64(counter);
    _struct := Vector(ExtendValueArray(EmptyValueArray, counter));
}

procedure {:inline 1} Unpack_LibraAccount_EventHandleGenerator(_struct: Value) returns (counter: Value)
{
    assume is#Vector(_struct);
    counter := SelectField(_struct, LibraAccount_EventHandleGenerator_counter);
    assume IsValidU64(counter);
}

const unique LibraAccount_EventHandle: TypeName;
const LibraAccount_EventHandle_counter: FieldName;
axiom LibraAccount_EventHandle_counter == 0;
const LibraAccount_EventHandle_guid: FieldName;
axiom LibraAccount_EventHandle_guid == 1;
function LibraAccount_EventHandle_type_value(tv0: TypeValue): TypeValue {
    StructType(LibraAccount_EventHandle, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), ByteArrayType()))
}
procedure {:inline 1} Pack_LibraAccount_EventHandle(tv0: TypeValue, counter: Value, guid: Value) returns (_struct: Value)
{
    assume IsValidU64(counter);
    assume is#ByteArray(guid);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, counter), guid));
}

procedure {:inline 1} Unpack_LibraAccount_EventHandle(_struct: Value) returns (counter: Value, guid: Value)
{
    assume is#Vector(_struct);
    counter := SelectField(_struct, LibraAccount_EventHandle_counter);
    assume IsValidU64(counter);
    guid := SelectField(_struct, LibraAccount_EventHandle_guid);
    assume is#ByteArray(guid);
}



// ** functions of module LibraAccount

procedure {:inline 1} LibraAccount_deposit (payee: Value, to_deposit: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Value; // AddressType()
    var t3: Value; // LibraCoin_T_type_value()
    var t4: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume is#Vector(to_deposit);
    __m := UpdateLocal(__m, __frame + 1, to_deposit);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    // unimplemented instruction: LdByteArray(4, ByteArrayPoolIndex(0))

    call LibraAccount_deposit_with_metadata(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_deposit_verify (payee: Value, to_deposit: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_deposit(payee, to_deposit);
}

procedure {:inline 1} LibraAccount_deposit_with_metadata (payee: Value, to_deposit: Value, metadata: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t3: Value; // AddressType()
    var t4: Value; // AddressType()
    var t5: Value; // LibraCoin_T_type_value()
    var t6: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume is#Vector(to_deposit);
    __m := UpdateLocal(__m, __frame + 1, to_deposit);
    assume is#ByteArray(metadata);
    __m := UpdateLocal(__m, __frame + 2, metadata);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call LibraAccount_deposit_with_sender_and_metadata(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_deposit_with_metadata_verify (payee: Value, to_deposit: Value, metadata: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_deposit_with_metadata(payee, to_deposit, metadata);
}

procedure {:inline 1} LibraAccount_deposit_with_sender_and_metadata (payee: Value, sender: Value, to_deposit: Value, metadata: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t4: Value; // IntegerType()
    var t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // IntegerType()
    var t11: Value; // BooleanType()
    var t12: Value; // BooleanType()
    var t13: Value; // IntegerType()
    var t14: Value; // AddressType()
    var t15: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t16: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t17: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value()))
    var t18: Value; // IntegerType()
    var t19: Value; // AddressType()
    var t20: Value; // ByteArrayType()
    var t21: Value; // LibraAccount_SentPaymentEvent_type_value()
    var t22: Value; // AddressType()
    var t23: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t24: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t25: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t26: Value; // LibraCoin_T_type_value()
    var t27: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t28: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value()))
    var t29: Value; // IntegerType()
    var t30: Value; // AddressType()
    var t31: Value; // ByteArrayType()
    var t32: Value; // LibraAccount_ReceivedPaymentEvent_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 33;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume is#Address(sender);
    __m := UpdateLocal(__m, __frame + 1, sender);
    assume is#Vector(to_deposit);
    __m := UpdateLocal(__m, __frame + 2, to_deposit);
    assume is#ByteArray(metadata);
    __m := UpdateLocal(__m, __frame + 3, metadata);

    // bytecode translation starts here
    call t7 := BorrowLoc(__frame + 2);

    call t8 := LibraCoin_value(t7);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(t8);

    __m := UpdateLocal(__m, __frame + 8, t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    __tmp := GetLocal(__m, __frame + 12);
    if (!b#Boolean(__tmp)) { goto Label_10; }

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    goto Label_Abort;

Label_10:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call t15 := BorrowGlobal(GetLocal(__m, __frame + 14), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t6 := CopyOrMoveRef(t15);

    call t16 := CopyOrMoveRef(t6);

    call t17 := BorrowField(t16, LibraAccount_T_sent_events);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := Pack_LibraAccount_SentPaymentEvent(GetLocal(__m, __frame + 18), GetLocal(__m, __frame + 19), GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    call LibraAccount_emit_event(LibraAccount_SentPaymentEvent_type_value(), t17, GetLocal(__m, __frame + 21));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    call t23 := BorrowGlobal(GetLocal(__m, __frame + 22), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t5 := CopyOrMoveRef(t23);

    call t24 := CopyOrMoveRef(t5);

    call t25 := BorrowField(t24, LibraAccount_T_balance);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 26, __tmp);

    call LibraCoin_deposit(t25, GetLocal(__m, __frame + 26));
    if (__abort_flag) { goto Label_Abort; }

    call t27 := CopyOrMoveRef(t5);

    call t28 := BorrowField(t27, LibraAccount_T_received_events);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 29, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 30, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 31, __tmp);

    call __tmp := Pack_LibraAccount_ReceivedPaymentEvent(GetLocal(__m, __frame + 29), GetLocal(__m, __frame + 30), GetLocal(__m, __frame + 31));
    __m := UpdateLocal(__m, __frame + 32, __tmp);

    call LibraAccount_emit_event(LibraAccount_ReceivedPaymentEvent_type_value(), t28, GetLocal(__m, __frame + 32));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_deposit_with_sender_and_metadata_verify (payee: Value, sender: Value, to_deposit: Value, metadata: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_deposit_with_sender_and_metadata(payee, sender, to_deposit, metadata);
}

procedure {:inline 1} LibraAccount_mint_to_address (payee: Value, amount: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Value; // AddressType()
    var t3: Value; // BooleanType()
    var t4: Value; // BooleanType()
    var t5: Value; // AddressType()
    var t6: Value; // AddressType()
    var t7: Value; // IntegerType()
    var t8: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 9;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 2), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __tmp := GetLocal(__m, __frame + 4);
    if (!b#Boolean(__tmp)) { goto Label_6; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call LibraAccount_create_account(GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }

Label_6:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call t8 := LibraCoin_mint_with_default_capability(GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t8);

    __m := UpdateLocal(__m, __frame + 8, t8);

    call LibraAccount_deposit(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_mint_to_address_verify (payee: Value, amount: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_mint_to_address(payee, amount);
}

procedure {:inline 1} LibraAccount_withdraw_from_account (account: Reference, amount: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Value; // LibraCoin_T_type_value()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t5: Value; // IntegerType()
    var t6: Value; // LibraCoin_T_type_value()
    var t7: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 8;

    // process and type check arguments
    assume is#Vector(Dereference(__m, account));
    assume IsValidReferenceParameter(__m, __frame, account);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(account);

    call t4 := BorrowField(t3, LibraAccount_T_balance);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call t6 := LibraCoin_withdraw(t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t6);

    __m := UpdateLocal(__m, __frame + 6, t6);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    ret0 := GetLocal(__m, __frame + 7);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_from_account_verify (account: Reference, amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_withdraw_from_account(account, amount);
}

procedure {:inline 1} LibraAccount_withdraw_from_sender (amount: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t5: Reference; // ReferenceType(BooleanType())
    var t6: Value; // BooleanType()
    var t7: Value; // IntegerType()
    var t8: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t9: Value; // IntegerType()
    var t10: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 11;

    // process and type check arguments
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 0, amount);

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, LibraAccount_T_delegated_withdrawal_capability);

    call __tmp := ReadRef(t5);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_9; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    goto Label_Abort;

Label_9:
    call t8 := CopyOrMoveRef(t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call t10 := LibraAccount_withdraw_from_account(t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t10);

    __m := UpdateLocal(__m, __frame + 10, t10);

    ret0 := GetLocal(__m, __frame + 10);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_from_sender_verify (amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_withdraw_from_sender(amount);
}

procedure {:inline 1} LibraAccount_withdraw_with_capability (cap: Reference, amount: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t4: Reference; // ReferenceType(AddressType())
    var t5: Value; // AddressType()
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t8: Value; // IntegerType()
    var t9: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 10;

    // process and type check arguments
    assume is#Vector(Dereference(__m, cap));
    assume IsValidReferenceParameter(__m, __frame, cap);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(cap);

    call t4 := BorrowField(t3, LibraAccount_WithdrawalCapability_account_address);

    call __tmp := ReadRef(t4);
    assume is#Address(__tmp);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call t6 := BorrowGlobal(GetLocal(__m, __frame + 5), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t6);

    call t7 := CopyOrMoveRef(t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call t9 := LibraAccount_withdraw_from_account(t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t9);

    __m := UpdateLocal(__m, __frame + 9, t9);

    ret0 := GetLocal(__m, __frame + 9);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_with_capability_verify (cap: Reference, amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_withdraw_with_capability(cap, amount);
}

procedure {:inline 1} LibraAccount_extract_sender_withdrawal_capability () returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t2: Reference; // ReferenceType(BooleanType())
    var t3: Value; // AddressType()
    var t4: Value; // AddressType()
    var t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Reference; // ReferenceType(BooleanType())
    var t8: Reference; // ReferenceType(BooleanType())
    var t9: Value; // BooleanType()
    var t10: Value; // IntegerType()
    var t11: Value; // BooleanType()
    var t12: Reference; // ReferenceType(BooleanType())
    var t13: Value; // AddressType()
    var t14: Value; // LibraAccount_WithdrawalCapability_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 15;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call t5 := BorrowGlobal(GetLocal(__m, __frame + 4), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, LibraAccount_T_delegated_withdrawal_capability);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call __tmp := ReadRef(t8);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __tmp := GetLocal(__m, __frame + 9);
    if (!b#Boolean(__tmp)) { goto Label_13; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    goto Label_Abort;

Label_13:
    call __tmp := LdTrue();
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call t12 := CopyOrMoveRef(t2);

    call WriteRef(t12, GetLocal(__m, __frame + 11));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := Pack_LibraAccount_WithdrawalCapability(GetLocal(__m, __frame + 13));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    ret0 := GetLocal(__m, __frame + 14);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_extract_sender_withdrawal_capability_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_extract_sender_withdrawal_capability();
}

procedure {:inline 1} LibraAccount_restore_withdrawal_capability (cap: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Value; // LibraAccount_WithdrawalCapability_type_value()
    var t4: Value; // AddressType()
    var t5: Value; // AddressType()
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Value; // BooleanType()
    var t8: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t9: Reference; // ReferenceType(BooleanType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 10;

    // process and type check arguments
    assume is#Vector(cap);
    __m := UpdateLocal(__m, __frame + 0, cap);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call t4 := Unpack_LibraAccount_WithdrawalCapability(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call t6 := BorrowGlobal(GetLocal(__m, __frame + 5), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t6);

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, LibraAccount_T_delegated_withdrawal_capability);

    call WriteRef(t9, GetLocal(__m, __frame + 7));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_restore_withdrawal_capability_verify (cap: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_restore_withdrawal_capability(cap);
}

procedure {:inline 1} LibraAccount_pay_from_capability (payee: Value, cap: Reference, amount: Value, metadata: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t4: Value; // AddressType()
    var t5: Value; // BooleanType()
    var t6: Value; // BooleanType()
    var t7: Value; // AddressType()
    var t8: Value; // AddressType()
    var t9: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t10: Reference; // ReferenceType(AddressType())
    var t11: Value; // AddressType()
    var t12: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t13: Value; // IntegerType()
    var t14: Value; // LibraCoin_T_type_value()
    var t15: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 16;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume is#Vector(Dereference(__m, cap));
    assume IsValidReferenceParameter(__m, __frame, cap);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 2, amount);
    assume is#ByteArray(metadata);
    __m := UpdateLocal(__m, __frame + 3, metadata);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 4), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_6; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call LibraAccount_create_account(GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }

Label_6:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call t9 := CopyOrMoveRef(cap);

    call t10 := BorrowField(t9, LibraAccount_WithdrawalCapability_account_address);

    call __tmp := ReadRef(t10);
    assume is#Address(__tmp);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call t12 := CopyOrMoveRef(cap);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call t14 := LibraAccount_withdraw_with_capability(t12, GetLocal(__m, __frame + 13));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t14);

    __m := UpdateLocal(__m, __frame + 14, t14);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call LibraAccount_deposit_with_sender_and_metadata(GetLocal(__m, __frame + 8), GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_pay_from_capability_verify (payee: Value, cap: Reference, amount: Value, metadata: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_pay_from_capability(payee, cap, amount, metadata);
}

procedure {:inline 1} LibraAccount_pay_from_sender_with_metadata (payee: Value, amount: Value, metadata: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t3: Value; // AddressType()
    var t4: Value; // BooleanType()
    var t5: Value; // BooleanType()
    var t6: Value; // AddressType()
    var t7: Value; // AddressType()
    var t8: Value; // IntegerType()
    var t9: Value; // LibraCoin_T_type_value()
    var t10: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 11;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    assume is#ByteArray(metadata);
    __m := UpdateLocal(__m, __frame + 2, metadata);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 3), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_6; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call LibraAccount_create_account(GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

Label_6:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call t9 := LibraAccount_withdraw_from_sender(GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t9);

    __m := UpdateLocal(__m, __frame + 9, t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call LibraAccount_deposit_with_metadata(GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_pay_from_sender_with_metadata_verify (payee: Value, amount: Value, metadata: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_pay_from_sender_with_metadata(payee, amount, metadata);
}

procedure {:inline 1} LibraAccount_pay_from_sender (payee: Value, amount: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Value; // AddressType()
    var t3: Value; // IntegerType()
    var t4: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume is#Address(payee);
    __m := UpdateLocal(__m, __frame + 0, payee);
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    // unimplemented instruction: LdByteArray(4, ByteArrayPoolIndex(0))

    call LibraAccount_pay_from_sender_with_metadata(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_pay_from_sender_verify (payee: Value, amount: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_pay_from_sender(payee, amount);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key_for_account (account: Reference, new_authentication_key: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Value; // ByteArrayType()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(ByteArrayType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume is#Vector(Dereference(__m, account));
    assume IsValidReferenceParameter(__m, __frame, account);
    assume is#ByteArray(new_authentication_key);
    __m := UpdateLocal(__m, __frame + 1, new_authentication_key);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call t3 := CopyOrMoveRef(account);

    call t4 := BorrowField(t3, LibraAccount_T_authentication_key);

    call WriteRef(t4, GetLocal(__m, __frame + 2));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_rotate_authentication_key_for_account_verify (account: Reference, new_authentication_key: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_rotate_authentication_key_for_account(account, new_authentication_key);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key (new_authentication_key: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t5: Reference; // ReferenceType(BooleanType())
    var t6: Value; // BooleanType()
    var t7: Value; // IntegerType()
    var t8: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t9: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 10;

    // process and type check arguments
    assume is#ByteArray(new_authentication_key);
    __m := UpdateLocal(__m, __frame + 0, new_authentication_key);

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, LibraAccount_T_delegated_key_rotation_capability);

    call __tmp := ReadRef(t5);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_9; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    goto Label_Abort;

Label_9:
    call t8 := CopyOrMoveRef(t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call LibraAccount_rotate_authentication_key_for_account(t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_rotate_authentication_key_verify (new_authentication_key: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_rotate_authentication_key(new_authentication_key);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key_with_capability (cap: Reference, new_authentication_key: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var t3: Reference; // ReferenceType(AddressType())
    var t4: Value; // AddressType()
    var t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t6: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments
    assume is#Vector(Dereference(__m, cap));
    assume IsValidReferenceParameter(__m, __frame, cap);
    assume is#ByteArray(new_authentication_key);
    __m := UpdateLocal(__m, __frame + 1, new_authentication_key);

    // bytecode translation starts here
    call t2 := CopyOrMoveRef(cap);

    call t3 := BorrowField(t2, LibraAccount_KeyRotationCapability_account_address);

    call __tmp := ReadRef(t3);
    assume is#Address(__tmp);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call t5 := BorrowGlobal(GetLocal(__m, __frame + 4), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call LibraAccount_rotate_authentication_key_for_account(t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_rotate_authentication_key_with_capability_verify (cap: Reference, new_authentication_key: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_rotate_authentication_key_with_capability(cap, new_authentication_key);
}

procedure {:inline 1} LibraAccount_extract_sender_key_rotation_capability () returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(BooleanType())
    var t2: Value; // AddressType()
    var t3: Value; // AddressType()
    var t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t5: Reference; // ReferenceType(BooleanType())
    var t6: Reference; // ReferenceType(BooleanType())
    var t7: Value; // BooleanType()
    var t8: Value; // IntegerType()
    var t9: Value; // BooleanType()
    var t10: Reference; // ReferenceType(BooleanType())
    var t11: Value; // AddressType()
    var t12: Value; // LibraAccount_KeyRotationCapability_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 13;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call t4 := BorrowGlobal(GetLocal(__m, __frame + 3), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t5 := BorrowField(t4, LibraAccount_T_delegated_key_rotation_capability);

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call __tmp := ReadRef(t6);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __tmp := GetLocal(__m, __frame + 7);
    if (!b#Boolean(__tmp)) { goto Label_11; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    goto Label_Abort;

Label_11:
    call __tmp := LdTrue();
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call t10 := CopyOrMoveRef(t1);

    call WriteRef(t10, GetLocal(__m, __frame + 9));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := Pack_LibraAccount_KeyRotationCapability(GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    ret0 := GetLocal(__m, __frame + 12);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_extract_sender_key_rotation_capability_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_extract_sender_key_rotation_capability();
}

procedure {:inline 1} LibraAccount_restore_key_rotation_capability (cap: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Value; // LibraAccount_KeyRotationCapability_type_value()
    var t4: Value; // AddressType()
    var t5: Value; // AddressType()
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Value; // BooleanType()
    var t8: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t9: Reference; // ReferenceType(BooleanType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 10;

    // process and type check arguments
    assume is#Vector(cap);
    __m := UpdateLocal(__m, __frame + 0, cap);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call t4 := Unpack_LibraAccount_KeyRotationCapability(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call t6 := BorrowGlobal(GetLocal(__m, __frame + 5), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t6);

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, LibraAccount_T_delegated_key_rotation_capability);

    call WriteRef(t9, GetLocal(__m, __frame + 7));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_restore_key_rotation_capability_verify (cap: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_restore_key_rotation_capability(cap);
}

procedure {:inline 1} LibraAccount_create_account (fresh_address: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // LibraAccount_EventHandleGenerator_type_value()
    var t2: Value; // IntegerType()
    var t3: Value; // LibraAccount_EventHandleGenerator_type_value()
    var t4: Value; // AddressType()
    var t5: Value; // AddressType()
    var t6: Value; // ByteArrayType()
    var t7: Value; // LibraCoin_T_type_value()
    var t8: Value; // BooleanType()
    var t9: Value; // BooleanType()
    var t10: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t11: Value; // AddressType()
    var t12: Value; // LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())
    var t13: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t14: Value; // AddressType()
    var t15: Value; // LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())
    var t16: Value; // IntegerType()
    var t17: Value; // LibraAccount_EventHandleGenerator_type_value()
    var t18: Value; // LibraAccount_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 19;

    // process and type check arguments
    assume is#Address(fresh_address);
    __m := UpdateLocal(__m, __frame + 0, fresh_address);

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := Pack_LibraAccount_EventHandleGenerator(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call t6 := AddressUtil_address_to_bytes(GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t6);

    __m := UpdateLocal(__m, __frame + 6, t6);

    call t7 := LibraCoin_zero();
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t7);

    __m := UpdateLocal(__m, __frame + 7, t7);

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call t10 := BorrowLoc(__frame + 1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call t12 := LibraAccount_new_event_handle_impl(LibraAccount_ReceivedPaymentEvent_type_value(), t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t12);

    __m := UpdateLocal(__m, __frame + 12, t12);

    call t13 := BorrowLoc(__frame + 1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call t15 := LibraAccount_new_event_handle_impl(LibraAccount_SentPaymentEvent_type_value(), t13, GetLocal(__m, __frame + 14));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t15);

    __m := UpdateLocal(__m, __frame + 15, t15);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := Pack_LibraAccount_T(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8), GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 12), GetLocal(__m, __frame + 15), GetLocal(__m, __frame + 16), GetLocal(__m, __frame + 17));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call LibraAccount_save_account(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 18));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_create_account_verify (fresh_address: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_create_account(fresh_address);
}

procedure {:inline 1} LibraAccount_create_new_account (fresh_address: Value, initial_balance: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Value; // AddressType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // BooleanType()
    var t6: Value; // AddressType()
    var t7: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 8;

    // process and type check arguments
    assume is#Address(fresh_address);
    __m := UpdateLocal(__m, __frame + 0, fresh_address);
    assume IsValidU64(initial_balance);
    __m := UpdateLocal(__m, __frame + 1, initial_balance);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call LibraAccount_create_account(GetLocal(__m, __frame + 2));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_9; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call LibraAccount_pay_from_sender(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }

Label_9:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_create_new_account_verify (fresh_address: Value, initial_balance: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_create_new_account(fresh_address, initial_balance);
}

procedure {:inline 1} LibraAccount_save_account (addr: Value, account: Value) returns ();
requires ExistsTxnSenderAccount(__m, __txn);

procedure {:inline 1} LibraAccount_balance_for_account (account: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 6;

    // process and type check arguments
    assume is#Vector(Dereference(__m, account));
    assume IsValidReferenceParameter(__m, __frame, account);

    // bytecode translation starts here
    call t2 := CopyOrMoveRef(account);

    call t3 := BorrowField(t2, LibraAccount_T_balance);

    call t4 := LibraCoin_value(t3);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(t4);

    __m := UpdateLocal(__m, __frame + 4, t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    ret0 := GetLocal(__m, __frame + 5);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_balance_for_account_verify (account: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_balance_for_account(account);
}

procedure {:inline 1} LibraAccount_balance (addr: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
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
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t3 := LibraAccount_balance_for_account(t2);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(t3);

    __m := UpdateLocal(__m, __frame + 3, t3);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_balance_verify (addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_balance(addr);
}

procedure {:inline 1} LibraAccount_sequence_number_for_account (account: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(LibraAccount_T_type_value())
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
    assume is#Vector(Dereference(__m, account));
    assume IsValidReferenceParameter(__m, __frame, account);

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(account);

    call t2 := BorrowField(t1, LibraAccount_T_sequence_number);

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

procedure LibraAccount_sequence_number_for_account_verify (account: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_sequence_number_for_account(account);
}

procedure {:inline 1} LibraAccount_sequence_number (addr: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
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
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t3 := LibraAccount_sequence_number_for_account(t2);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(t3);

    __m := UpdateLocal(__m, __frame + 3, t3);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_sequence_number_verify (addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_sequence_number(addr);
}

procedure {:inline 1} LibraAccount_delegated_key_rotation_capability (addr: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(BooleanType())
    var t4: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t3 := BorrowField(t2, LibraAccount_T_delegated_key_rotation_capability);

    call __tmp := ReadRef(t3);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    ret0 := GetLocal(__m, __frame + 4);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_delegated_key_rotation_capability_verify (addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_delegated_key_rotation_capability(addr);
}

procedure {:inline 1} LibraAccount_delegated_withdrawal_capability (addr: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(BooleanType())
    var t4: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;

    // process and type check arguments
    assume is#Address(addr);
    __m := UpdateLocal(__m, __frame + 0, addr);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call t2 := BorrowGlobal(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t3 := BorrowField(t2, LibraAccount_T_delegated_withdrawal_capability);

    call __tmp := ReadRef(t3);
    assume is#Boolean(__tmp);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    ret0 := GetLocal(__m, __frame + 4);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_delegated_withdrawal_capability_verify (addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_delegated_withdrawal_capability(addr);
}

procedure {:inline 1} LibraAccount_withdrawal_capability_address (cap: Reference) returns (ret0: Reference)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t2: Reference; // ReferenceType(AddressType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 3;

    // process and type check arguments
    assume is#Vector(Dereference(__m, cap));
    assume IsValidReferenceParameter(__m, __frame, cap);

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(cap);

    call t2 := BorrowField(t1, LibraAccount_WithdrawalCapability_account_address);

    ret0 := t2;
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultReference;
}

procedure LibraAccount_withdrawal_capability_address_verify (cap: Reference) returns (ret0: Reference)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_withdrawal_capability_address(cap);
}

procedure {:inline 1} LibraAccount_key_rotation_capability_address (cap: Reference) returns (ret0: Reference)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var t2: Reference; // ReferenceType(AddressType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 3;

    // process and type check arguments
    assume is#Vector(Dereference(__m, cap));
    assume IsValidReferenceParameter(__m, __frame, cap);

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(cap);

    call t2 := BorrowField(t1, LibraAccount_KeyRotationCapability_account_address);

    ret0 := t2;
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultReference;
}

procedure LibraAccount_key_rotation_capability_address_verify (cap: Reference) returns (ret0: Reference)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_key_rotation_capability_address(cap);
}

procedure {:inline 1} LibraAccount_exists (check_addr: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 3;

    // process and type check arguments
    assume is#Address(check_addr);
    __m := UpdateLocal(__m, __frame + 0, check_addr);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 1), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    ret0 := GetLocal(__m, __frame + 2);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_exists_verify (check_addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_exists(check_addr);
}

procedure {:inline 1} LibraAccount_prologue (txn_sequence_number: Value, txn_public_key: Value, txn_gas_price: Value, txn_max_gas_units: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t4: Value; // AddressType()
    var t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // AddressType()
    var t11: Value; // AddressType()
    var t12: Value; // BooleanType()
    var t13: Value; // BooleanType()
    var t14: Value; // IntegerType()
    var t15: Value; // AddressType()
    var t16: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t17: Value; // ByteArrayType()
    var t18: Value; // ByteArrayType()
    var t19: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t20: Reference; // ReferenceType(ByteArrayType())
    var t21: Value; // ByteArrayType()
    var t22: Value; // BooleanType()
    var t23: Value; // BooleanType()
    var t24: Value; // IntegerType()
    var t25: Value; // IntegerType()
    var t26: Value; // IntegerType()
    var t27: Value; // IntegerType()
    var t28: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t29: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t30: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t31: Value; // IntegerType()
    var t32: Value; // IntegerType()
    var t33: Value; // IntegerType()
    var t34: Value; // BooleanType()
    var t35: Value; // BooleanType()
    var t36: Value; // IntegerType()
    var t37: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t38: Reference; // ReferenceType(IntegerType())
    var t39: Value; // IntegerType()
    var t40: Value; // IntegerType()
    var t41: Value; // IntegerType()
    var t42: Value; // BooleanType()
    var t43: Value; // BooleanType()
    var t44: Value; // IntegerType()
    var t45: Value; // IntegerType()
    var t46: Value; // IntegerType()
    var t47: Value; // BooleanType()
    var t48: Value; // BooleanType()
    var t49: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 50;

    // process and type check arguments
    assume IsValidU64(txn_sequence_number);
    __m := UpdateLocal(__m, __frame + 0, txn_sequence_number);
    assume is#ByteArray(txn_public_key);
    __m := UpdateLocal(__m, __frame + 1, txn_public_key);
    assume IsValidU64(txn_gas_price);
    __m := UpdateLocal(__m, __frame + 2, txn_gas_price);
    assume IsValidU64(txn_max_gas_units);
    __m := UpdateLocal(__m, __frame + 3, txn_max_gas_units);

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 10));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := Exists(GetLocal(__m, __frame + 11), LibraAccount_T_type_value());
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    __tmp := GetLocal(__m, __frame + 13);
    if (!b#Boolean(__tmp)) { goto Label_8; }

    call __tmp := LdConst(5);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    goto Label_Abort;

Label_8:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call t16 := BorrowGlobal(GetLocal(__m, __frame + 15), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t5 := CopyOrMoveRef(t16);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call t18 := Hash_sha3_256(GetLocal(__m, __frame + 17));
    if (__abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t18);

    __m := UpdateLocal(__m, __frame + 18, t18);

    call t19 := CopyOrMoveRef(t5);

    call t20 := BorrowField(t19, LibraAccount_T_authentication_key);

    call __tmp := ReadRef(t20);
    assume is#ByteArray(__tmp);
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 18), GetLocal(__m, __frame + 21)));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 22));
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    __tmp := GetLocal(__m, __frame + 23);
    if (!b#Boolean(__tmp)) { goto Label_21; }

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 24, __tmp);

    goto Label_Abort;

Label_21:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 25, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 26, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 25), GetLocal(__m, __frame + 26));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 27, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 27));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call t28 := CopyOrMoveRef(t5);

    call t29 := FreezeRef(t28);

    call t6 := CopyOrMoveRef(t29);

    call t30 := CopyOrMoveRef(t6);

    call t31 := LibraAccount_balance_for_account(t30);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(t31);

    __m := UpdateLocal(__m, __frame + 31, t31);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 31));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 32, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 33, __tmp);

    call __tmp := Ge(GetLocal(__m, __frame + 32), GetLocal(__m, __frame + 33));
    __m := UpdateLocal(__m, __frame + 34, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 34));
    __m := UpdateLocal(__m, __frame + 35, __tmp);

    __tmp := GetLocal(__m, __frame + 35);
    if (!b#Boolean(__tmp)) { goto Label_38; }

    call __tmp := LdConst(6);
    __m := UpdateLocal(__m, __frame + 36, __tmp);

    goto Label_Abort;

Label_38:
    call t37 := CopyOrMoveRef(t5);

    call t38 := BorrowField(t37, LibraAccount_T_sequence_number);

    call __tmp := ReadRef(t38);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 39, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 39));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 40, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 41, __tmp);

    call __tmp := Ge(GetLocal(__m, __frame + 40), GetLocal(__m, __frame + 41));
    __m := UpdateLocal(__m, __frame + 42, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 42));
    __m := UpdateLocal(__m, __frame + 43, __tmp);

    __tmp := GetLocal(__m, __frame + 43);
    if (!b#Boolean(__tmp)) { goto Label_49; }

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 44, __tmp);

    goto Label_Abort;

Label_49:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 45, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 46, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 45), GetLocal(__m, __frame + 46)));
    __m := UpdateLocal(__m, __frame + 47, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 47));
    __m := UpdateLocal(__m, __frame + 48, __tmp);

    __tmp := GetLocal(__m, __frame + 48);
    if (!b#Boolean(__tmp)) { goto Label_56; }

    call __tmp := LdConst(4);
    __m := UpdateLocal(__m, __frame + 49, __tmp);

    goto Label_Abort;

Label_56:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_prologue_verify (txn_sequence_number: Value, txn_public_key: Value, txn_gas_price: Value, txn_max_gas_units: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_prologue(txn_sequence_number, txn_public_key, txn_gas_price, txn_max_gas_units);
}

procedure {:inline 1} LibraAccount_epilogue (txn_sequence_number: Value, txn_gas_price: Value, txn_max_gas_units: Value, gas_units_remaining: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Value; // IntegerType()
    var t8: Value; // LibraCoin_T_type_value()
    var t9: Value; // AddressType()
    var t10: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t11: Value; // IntegerType()
    var t12: Value; // IntegerType()
    var t13: Value; // IntegerType()
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t17: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t18: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t19: Value; // IntegerType()
    var t20: Value; // IntegerType()
    var t21: Value; // BooleanType()
    var t22: Value; // BooleanType()
    var t23: Value; // IntegerType()
    var t24: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t25: Value; // IntegerType()
    var t26: Value; // LibraCoin_T_type_value()
    var t27: Value; // IntegerType()
    var t28: Value; // IntegerType()
    var t29: Value; // IntegerType()
    var t30: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t31: Reference; // ReferenceType(IntegerType())
    var t32: Value; // AddressType()
    var t33: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t34: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t35: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t36: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 37;

    // process and type check arguments
    assume IsValidU64(txn_sequence_number);
    __m := UpdateLocal(__m, __frame + 0, txn_sequence_number);
    assume IsValidU64(txn_gas_price);
    __m := UpdateLocal(__m, __frame + 1, txn_gas_price);
    assume IsValidU64(txn_max_gas_units);
    __m := UpdateLocal(__m, __frame + 2, txn_max_gas_units);
    assume IsValidU64(gas_units_remaining);
    __m := UpdateLocal(__m, __frame + 3, gas_units_remaining);

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call t10 := BorrowGlobal(GetLocal(__m, __frame + 9), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t4 := CopyOrMoveRef(t10);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 12), GetLocal(__m, __frame + 13));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 14));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 15));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call t16 := CopyOrMoveRef(t4);

    call t17 := FreezeRef(t16);

    call t6 := CopyOrMoveRef(t17);

    call t18 := CopyOrMoveRef(t6);

    call t19 := LibraAccount_balance_for_account(t18);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(t19);

    __m := UpdateLocal(__m, __frame + 19, t19);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := Ge(GetLocal(__m, __frame + 19), GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 21));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    __tmp := GetLocal(__m, __frame + 22);
    if (!b#Boolean(__tmp)) { goto Label_20; }

    call __tmp := LdConst(6);
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    goto Label_Abort;

Label_20:
    call t24 := CopyOrMoveRef(t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 25, __tmp);

    call t26 := LibraAccount_withdraw_from_account(t24, GetLocal(__m, __frame + 25));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t26);

    __m := UpdateLocal(__m, __frame + 26, t26);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 26));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 27, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 28, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 27), GetLocal(__m, __frame + 28));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 29, __tmp);

    call t30 := CopyOrMoveRef(t4);

    call t31 := BorrowField(t30, LibraAccount_T_sequence_number);

    call WriteRef(t31, GetLocal(__m, __frame + 29));

    call __tmp := LdAddr(4078);
    __m := UpdateLocal(__m, __frame + 32, __tmp);

    call t33 := BorrowGlobal(GetLocal(__m, __frame + 32), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t5 := CopyOrMoveRef(t33);

    call t34 := CopyOrMoveRef(t5);

    call t35 := BorrowField(t34, LibraAccount_T_balance);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 36, __tmp);

    call LibraCoin_deposit(t35, GetLocal(__m, __frame + 36));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_epilogue_verify (txn_sequence_number: Value, txn_gas_price: Value, txn_max_gas_units: Value, gas_units_remaining: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_epilogue(txn_sequence_number, txn_gas_price, txn_max_gas_units, gas_units_remaining);
}

procedure {:inline 1} LibraAccount_fresh_guid (counter: Reference, sender: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // ByteArrayType()
    var t4: Value; // ByteArrayType()
    var t5: Value; // ByteArrayType()
    var t6: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t7: Reference; // ReferenceType(IntegerType())
    var t8: Value; // AddressType()
    var t9: Value; // ByteArrayType()
    var t10: Reference; // ReferenceType(IntegerType())
    var t11: Value; // IntegerType()
    var t12: Value; // ByteArrayType()
    var t13: Reference; // ReferenceType(IntegerType())
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Value; // IntegerType()
    var t17: Reference; // ReferenceType(IntegerType())
    var t18: Value; // ByteArrayType()
    var t19: Value; // ByteArrayType()
    var t20: Value; // ByteArrayType()
    var t21: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 22;

    // process and type check arguments
    assume is#Vector(Dereference(__m, counter));
    assume IsValidReferenceParameter(__m, __frame, counter);
    assume is#Address(sender);
    __m := UpdateLocal(__m, __frame + 1, sender);

    // bytecode translation starts here
    call t6 := CopyOrMoveRef(counter);

    call t7 := BorrowField(t6, LibraAccount_EventHandleGenerator_counter);

    call t2 := CopyOrMoveRef(t7);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call t9 := AddressUtil_address_to_bytes(GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t9);

    __m := UpdateLocal(__m, __frame + 9, t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call t10 := CopyOrMoveRef(t2);

    call __tmp := ReadRef(t10);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call t12 := U64Util_u64_to_bytes(GetLocal(__m, __frame + 11));
    if (__abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t12);

    __m := UpdateLocal(__m, __frame + 12, t12);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call t13 := CopyOrMoveRef(t2);

    call __tmp := ReadRef(t13);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call t17 := CopyOrMoveRef(t2);

    call WriteRef(t17, GetLocal(__m, __frame + 16));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call t20 := BytearrayUtil_bytearray_concat(GetLocal(__m, __frame + 18), GetLocal(__m, __frame + 19));
    if (__abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t20);

    __m := UpdateLocal(__m, __frame + 20, t20);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    ret0 := GetLocal(__m, __frame + 21);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_fresh_guid_verify (counter: Reference, sender: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_fresh_guid(counter, sender);
}

procedure {:inline 1} LibraAccount_new_event_handle_impl (tv0: TypeValue, counter: Reference, sender: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t4: Value; // AddressType()
    var t5: Value; // ByteArrayType()
    var t6: Value; // LibraAccount_EventHandle_type_value(tv0)
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments
    assume is#Vector(Dereference(__m, counter));
    assume IsValidReferenceParameter(__m, __frame, counter);
    assume is#Address(sender);
    __m := UpdateLocal(__m, __frame + 1, sender);

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call t3 := CopyOrMoveRef(counter);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call t5 := LibraAccount_fresh_guid(t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t5);

    __m := UpdateLocal(__m, __frame + 5, t5);

    call __tmp := Pack_LibraAccount_EventHandle(tv0, GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    ret0 := GetLocal(__m, __frame + 6);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_new_event_handle_impl_verify (tv0: TypeValue, counter: Reference, sender: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_new_event_handle_impl(tv0, counter, sender);
}

procedure {:inline 1} LibraAccount_new_event_handle (tv0: TypeValue) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t1: Value; // ByteArrayType()
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t5: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t6: Value; // AddressType()
    var t7: Value; // LibraAccount_EventHandle_type_value(tv0)
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 8;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraAccount_T_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call t0 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, LibraAccount_T_event_generator);

    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call t7 := LibraAccount_new_event_handle_impl(tv0, t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t7);

    __m := UpdateLocal(__m, __frame + 7, t7);

    ret0 := GetLocal(__m, __frame + 7);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_new_event_handle_verify (tv0: TypeValue) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := LibraAccount_new_event_handle(tv0);
}

procedure {:inline 1} LibraAccount_emit_event (tv0: TypeValue, handle_ref: Reference, msg: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // ByteArrayType()
    var t4: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(tv0))
    var t5: Reference; // ReferenceType(ByteArrayType())
    var t6: Value; // ByteArrayType()
    var t7: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(tv0))
    var t8: Reference; // ReferenceType(IntegerType())
    var t9: Value; // ByteArrayType()
    var t10: Reference; // ReferenceType(IntegerType())
    var t11: Value; // IntegerType()
    var t12: Value; // tv0
    var t13: Reference; // ReferenceType(IntegerType())
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Value; // IntegerType()
    var t17: Reference; // ReferenceType(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 18;

    // process and type check arguments
    assume is#Vector(Dereference(__m, handle_ref));
    assume IsValidReferenceParameter(__m, __frame, handle_ref);
    __m := UpdateLocal(__m, __frame + 1, msg);

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(handle_ref);

    call t5 := BorrowField(t4, LibraAccount_EventHandle_guid);

    call __tmp := ReadRef(t5);
    assume is#ByteArray(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call t7 := CopyOrMoveRef(handle_ref);

    call t8 := BorrowField(t7, LibraAccount_EventHandle_counter);

    call t2 := CopyOrMoveRef(t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call t10 := CopyOrMoveRef(t2);

    call __tmp := ReadRef(t10);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call LibraAccount_write_to_event_store(tv0, GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 12));
    if (__abort_flag) { goto Label_Abort; }

    call t13 := CopyOrMoveRef(t2);

    call __tmp := ReadRef(t13);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call t17 := CopyOrMoveRef(t2);

    call WriteRef(t17, GetLocal(__m, __frame + 16));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_emit_event_verify (tv0: TypeValue, handle_ref: Reference, msg: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_emit_event(tv0, handle_ref, msg);
}

procedure {:inline 1} LibraAccount_write_to_event_store (tv0: TypeValue, guid: Value, count: Value, msg: Value) returns ();
requires ExistsTxnSenderAccount(__m, __txn);

procedure {:inline 1} LibraAccount_destroy_handle (tv0: TypeValue, handle: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // ByteArrayType()
    var t2: Value; // IntegerType()
    var t3: Value; // LibraAccount_EventHandle_type_value(tv0)
    var t4: Value; // IntegerType()
    var t5: Value; // ByteArrayType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 6;

    // process and type check arguments
    assume is#Vector(handle);
    __m := UpdateLocal(__m, __frame + 0, handle);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call t4, t5 := Unpack_LibraAccount_EventHandle(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, t4);
    __m := UpdateLocal(__m, __frame + 5, t5);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraAccount_destroy_handle_verify (tv0: TypeValue, handle: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call LibraAccount_destroy_handle(tv0, handle);
}



// ** structs of module TestLib



// ** functions of module TestLib
