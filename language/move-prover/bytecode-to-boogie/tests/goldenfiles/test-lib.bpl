

// ** structs of module U64Util



// ** functions of module U64Util

procedure {:inline 1} U64Util_u64_to_bytes (i: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(m, txn);



// ** structs of module AddressUtil



// ** functions of module AddressUtil

procedure {:inline 1} AddressUtil_address_to_bytes (addr: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(m, txn);



// ** structs of module BytearrayUtil



// ** functions of module BytearrayUtil

procedure {:inline 1} BytearrayUtil_bytearray_concat (data1: Value, data2: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(m, txn);



// ** structs of module Hash



// ** functions of module Hash

procedure {:inline 1} Hash_sha2_256 (data: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(m, txn);

procedure {:inline 1} Hash_sha3_256 (data: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(m, txn);



// ** structs of module Signature



// ** functions of module Signature

procedure {:inline 1} Signature_ed25519_verify (signature: Value, public_key: Value, message: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(m, txn);

procedure {:inline 1} Signature_ed25519_threshold_verify (bitmap: Value, signature: Value, public_key: Value, message: Value) returns (ret0: Value);
requires ExistsTxnSenderAccount(m, txn);



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
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Value; // AddressType()
    var t3: Value; // BooleanType()
    var t4: Value; // BooleanType()
    var t5: Value; // IntegerType()
    var t6: Value; // GasSchedule_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(gas_schedule);

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 0, gas_schedule);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := LdAddr(173345816);
    m := UpdateLocal(m, old_size + 2, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 1), GetLocal(m, old_size + 2)));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := Not(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, tmp);

    tmp := GetLocal(m, old_size + 4);
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 5, tmp);

    goto Label_Abort;

Label_7:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 6, tmp);

    call MoveToSender(GasSchedule_T_type_value(), GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure GasSchedule_initialize_verify (gas_schedule: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call GasSchedule_initialize(gas_schedule);
}

procedure {:inline 1} GasSchedule_instruction_table_size () returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 8;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := BorrowGlobal(GetLocal(m, old_size + 2), GasSchedule_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t0 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, GasSchedule_T_instruction_schedule);

    call t6 := Vector_length(GasSchedule_Cost_type_value(), t5);
    if (abort_flag) { goto Label_Abort; }
    assume IsValidU64(t6);

    m := UpdateLocal(m, old_size + 6, t6);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    ret0 := GetLocal(m, old_size + 7);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure GasSchedule_instruction_table_size_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := GasSchedule_instruction_table_size();
}

procedure {:inline 1} GasSchedule_native_table_size () returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 8;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := BorrowGlobal(GetLocal(m, old_size + 2), GasSchedule_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t0 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, GasSchedule_T_native_schedule);

    call t6 := Vector_length(GasSchedule_Cost_type_value(), t5);
    if (abort_flag) { goto Label_Abort; }
    assume IsValidU64(t6);

    m := UpdateLocal(m, old_size + 6, t6);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    ret0 := GetLocal(m, old_size + 7);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure GasSchedule_native_table_size_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
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
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(addr);

    old_size := local_counter;
    local_counter := local_counter + 3;
    m := UpdateLocal(m, old_size + 0, addr);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := Exists(GetLocal(m, old_size + 1), ValidatorConfig_T_type_value());
    m := UpdateLocal(m, old_size + 2, tmp);

    ret0 := GetLocal(m, old_size + 2);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_has_verify (addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := ValidatorConfig_has(addr);
}

procedure {:inline 1} ValidatorConfig_config (addr: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t4: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t5: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t6: Value; // ValidatorConfig_Config_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(addr);

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 0, addr);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := BorrowGlobal(GetLocal(m, old_size + 2), ValidatorConfig_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, ValidatorConfig_T_config);

    call tmp := ReadRef(t5);
    assume is#Vector(tmp);

    m := UpdateLocal(m, old_size + 6, tmp);

    ret0 := GetLocal(m, old_size + 6);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_config_verify (addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := ValidatorConfig_config(addr);
}

procedure {:inline 1} ValidatorConfig_consensus_pubkey (config_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, config_ref));
    assume IsValidReferenceParameter(m, local_counter, config_ref);

    old_size := local_counter;
    local_counter := local_counter + 4;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(config_ref);

    call t2 := BorrowField(t1, ValidatorConfig_Config_consensus_pubkey);

    call tmp := ReadRef(t2);
    assume is#ByteArray(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_consensus_pubkey_verify (config_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := ValidatorConfig_consensus_pubkey(config_ref);
}

procedure {:inline 1} ValidatorConfig_validator_network_signing_pubkey (config_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, config_ref));
    assume IsValidReferenceParameter(m, local_counter, config_ref);

    old_size := local_counter;
    local_counter := local_counter + 4;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(config_ref);

    call t2 := BorrowField(t1, ValidatorConfig_Config_validator_network_signing_pubkey);

    call tmp := ReadRef(t2);
    assume is#ByteArray(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_validator_network_signing_pubkey_verify (config_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := ValidatorConfig_validator_network_signing_pubkey(config_ref);
}

procedure {:inline 1} ValidatorConfig_validator_network_identity_pubkey (config_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, config_ref));
    assume IsValidReferenceParameter(m, local_counter, config_ref);

    old_size := local_counter;
    local_counter := local_counter + 4;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(config_ref);

    call t2 := BorrowField(t1, ValidatorConfig_Config_validator_network_identity_pubkey);

    call tmp := ReadRef(t2);
    assume is#ByteArray(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_validator_network_identity_pubkey_verify (config_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := ValidatorConfig_validator_network_identity_pubkey(config_ref);
}

procedure {:inline 1} ValidatorConfig_validator_network_address (config_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, config_ref));
    assume IsValidReferenceParameter(m, local_counter, config_ref);

    old_size := local_counter;
    local_counter := local_counter + 4;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(config_ref);

    call t2 := BorrowField(t1, ValidatorConfig_Config_validator_network_address);

    call tmp := ReadRef(t2);
    assume is#ByteArray(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_validator_network_address_verify (config_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := ValidatorConfig_validator_network_address(config_ref);
}

procedure {:inline 1} ValidatorConfig_fullnodes_network_identity_pubkey (config_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, config_ref));
    assume IsValidReferenceParameter(m, local_counter, config_ref);

    old_size := local_counter;
    local_counter := local_counter + 4;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(config_ref);

    call t2 := BorrowField(t1, ValidatorConfig_Config_fullnodes_network_identity_pubkey);

    call tmp := ReadRef(t2);
    assume is#ByteArray(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_fullnodes_network_identity_pubkey_verify (config_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := ValidatorConfig_fullnodes_network_identity_pubkey(config_ref);
}

procedure {:inline 1} ValidatorConfig_fullnodes_network_address (config_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, config_ref));
    assume IsValidReferenceParameter(m, local_counter, config_ref);

    old_size := local_counter;
    local_counter := local_counter + 4;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(config_ref);

    call t2 := BorrowField(t1, ValidatorConfig_Config_fullnodes_network_address);

    call tmp := ReadRef(t2);
    assume is#ByteArray(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure ValidatorConfig_fullnodes_network_address_verify (config_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := ValidatorConfig_fullnodes_network_address(config_ref);
}

procedure {:inline 1} ValidatorConfig_register_candidate_validator (consensus_pubkey: Value, validator_network_signing_pubkey: Value, validator_network_identity_pubkey: Value, validator_network_address: Value, fullnodes_network_identity_pubkey: Value, fullnodes_network_address: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#ByteArray(consensus_pubkey);
    assume is#ByteArray(validator_network_signing_pubkey);
    assume is#ByteArray(validator_network_identity_pubkey);
    assume is#ByteArray(validator_network_address);
    assume is#ByteArray(fullnodes_network_identity_pubkey);
    assume is#ByteArray(fullnodes_network_address);

    old_size := local_counter;
    local_counter := local_counter + 14;
    m := UpdateLocal(m, old_size + 0, consensus_pubkey);
    m := UpdateLocal(m, old_size + 1, validator_network_signing_pubkey);
    m := UpdateLocal(m, old_size + 2, validator_network_identity_pubkey);
    m := UpdateLocal(m, old_size + 3, validator_network_address);
    m := UpdateLocal(m, old_size + 4, fullnodes_network_identity_pubkey);
    m := UpdateLocal(m, old_size + 5, fullnodes_network_address);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 10, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := Pack_ValidatorConfig_Config(GetLocal(m, old_size + 6), GetLocal(m, old_size + 7), GetLocal(m, old_size + 8), GetLocal(m, old_size + 9), GetLocal(m, old_size + 10), GetLocal(m, old_size + 11));
    m := UpdateLocal(m, old_size + 12, tmp);

    call tmp := Pack_ValidatorConfig_T(GetLocal(m, old_size + 12));
    m := UpdateLocal(m, old_size + 13, tmp);

    call MoveToSender(ValidatorConfig_T_type_value(), GetLocal(m, old_size + 13));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure ValidatorConfig_register_candidate_validator_verify (consensus_pubkey: Value, validator_network_signing_pubkey: Value, validator_network_identity_pubkey: Value, validator_network_address: Value, fullnodes_network_identity_pubkey: Value, fullnodes_network_address: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call ValidatorConfig_register_candidate_validator(consensus_pubkey, validator_network_signing_pubkey, validator_network_identity_pubkey, validator_network_address, fullnodes_network_identity_pubkey, fullnodes_network_address);
}

procedure {:inline 1} ValidatorConfig_rotate_consensus_pubkey (consensus_pubkey: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#ByteArray(consensus_pubkey);

    old_size := local_counter;
    local_counter := local_counter + 12;
    m := UpdateLocal(m, old_size + 0, consensus_pubkey);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 4, tmp);

    call t5 := BorrowGlobal(GetLocal(m, old_size + 4), ValidatorConfig_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, ValidatorConfig_T_config);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, ValidatorConfig_Config_consensus_pubkey);

    call t3 := CopyOrMoveRef(t9);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 10, tmp);

    call t11 := CopyOrMoveRef(t3);

    call WriteRef(t11, GetLocal(m, old_size + 10));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure ValidatorConfig_rotate_consensus_pubkey_verify (consensus_pubkey: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call ValidatorConfig_rotate_consensus_pubkey(consensus_pubkey);
}

procedure {:inline 1} ValidatorConfig_rotate_validator_network_identity_pubkey (validator_network_identity_pubkey: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#ByteArray(validator_network_identity_pubkey);

    old_size := local_counter;
    local_counter := local_counter + 12;
    m := UpdateLocal(m, old_size + 0, validator_network_identity_pubkey);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 4, tmp);

    call t5 := BorrowGlobal(GetLocal(m, old_size + 4), ValidatorConfig_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, ValidatorConfig_T_config);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, ValidatorConfig_Config_validator_network_identity_pubkey);

    call t3 := CopyOrMoveRef(t9);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 10, tmp);

    call t11 := CopyOrMoveRef(t3);

    call WriteRef(t11, GetLocal(m, old_size + 10));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure ValidatorConfig_rotate_validator_network_identity_pubkey_verify (validator_network_identity_pubkey: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call ValidatorConfig_rotate_validator_network_identity_pubkey(validator_network_identity_pubkey);
}

procedure {:inline 1} ValidatorConfig_rotate_validator_network_address (validator_network_address: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#ByteArray(validator_network_address);

    old_size := local_counter;
    local_counter := local_counter + 12;
    m := UpdateLocal(m, old_size + 0, validator_network_address);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 4, tmp);

    call t5 := BorrowGlobal(GetLocal(m, old_size + 4), ValidatorConfig_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, ValidatorConfig_T_config);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, ValidatorConfig_Config_validator_network_address);

    call t3 := CopyOrMoveRef(t9);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 10, tmp);

    call t11 := CopyOrMoveRef(t3);

    call WriteRef(t11, GetLocal(m, old_size + 10));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure ValidatorConfig_rotate_validator_network_address_verify (validator_network_address: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
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
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(LibraCoin_MintCapability_type_value())
    var t4: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(amount);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, amount);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := BorrowGlobal(GetLocal(m, old_size + 2), LibraCoin_MintCapability_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t4 := LibraCoin_mint(GetLocal(m, old_size + 1), t3);
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t4);

    m := UpdateLocal(m, old_size + 4, t4);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_mint_with_default_capability_verify (amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_mint_with_default_capability(amount);
}

procedure {:inline 1} LibraCoin_mint (value: Value, capability: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(value);
    assume is#Vector(Dereference(m, capability));
    assume IsValidReferenceParameter(m, local_counter, capability);

    old_size := local_counter;
    local_counter := local_counter + 21;
    m := UpdateLocal(m, old_size + 0, value);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := LdConst(1000000000);
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := LdConst(1000000);
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := MulU64(GetLocal(m, old_size + 4), GetLocal(m, old_size + 5));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := Le(GetLocal(m, old_size + 3), GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := Not(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 8, tmp);

    tmp := GetLocal(m, old_size + 8);
    if (!b#Boolean(tmp)) { goto Label_9; }

    call tmp := LdConst(11);
    m := UpdateLocal(m, old_size + 9, tmp);

    goto Label_Abort;

Label_9:
    call tmp := LdAddr(173345816);
    m := UpdateLocal(m, old_size + 10, tmp);

    call t11 := BorrowGlobal(GetLocal(m, old_size + 10), LibraCoin_MarketCap_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t12 := BorrowField(t11, LibraCoin_MarketCap_total_value);

    call t2 := CopyOrMoveRef(t12);

    call t13 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t13);
    assume IsValidU128(tmp);

    m := UpdateLocal(m, old_size + 14, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 15, tmp);

    call tmp := CastU128(GetLocal(m, old_size + 15));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 16, tmp);

    call tmp := AddU128(GetLocal(m, old_size + 14), GetLocal(m, old_size + 16));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 17, tmp);

    call t18 := CopyOrMoveRef(t2);

    call WriteRef(t18, GetLocal(m, old_size + 17));

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 19, tmp);

    call tmp := Pack_LibraCoin_T(GetLocal(m, old_size + 19));
    m := UpdateLocal(m, old_size + 20, tmp);

    ret0 := GetLocal(m, old_size + 20);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_mint_verify (value: Value, capability: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_mint(value, capability);
}

procedure {:inline 1} LibraCoin_initialize () returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 9;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := LdAddr(173345816);
    m := UpdateLocal(m, old_size + 1, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 0), GetLocal(m, old_size + 1)));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := Not(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 3, tmp);

    tmp := GetLocal(m, old_size + 3);
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 4, tmp);

    goto Label_Abort;

Label_7:
    call tmp := LdTrue();
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := Pack_LibraCoin_MintCapability(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 6, tmp);

    call MoveToSender(LibraCoin_MintCapability_type_value(), GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := Pack_LibraCoin_MarketCap(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 8, tmp);

    call MoveToSender(LibraCoin_MarketCap_type_value(), GetLocal(m, old_size + 8));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraCoin_initialize_verify () returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraCoin_initialize();
}

procedure {:inline 1} LibraCoin_market_cap () returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 4;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := UpdateLocal(m, old_size + 0, tmp);

    call t1 := BorrowGlobal(GetLocal(m, old_size + 0), LibraCoin_MarketCap_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t2 := BorrowField(t1, LibraCoin_MarketCap_total_value);

    call tmp := ReadRef(t2);
    assume IsValidU128(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_market_cap_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_market_cap();
}

procedure {:inline 1} LibraCoin_zero () returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 2;

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := Pack_LibraCoin_T(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    ret0 := GetLocal(m, old_size + 1);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_zero_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_zero();
}

procedure {:inline 1} LibraCoin_value (coin_ref: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, coin_ref));
    assume IsValidReferenceParameter(m, local_counter, coin_ref);

    old_size := local_counter;
    local_counter := local_counter + 4;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(coin_ref);

    call t2 := BorrowField(t1, LibraCoin_T_value);

    call tmp := ReadRef(t2);
    assume IsValidU64(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_value_verify (coin_ref: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_value(coin_ref);
}

procedure {:inline 1} LibraCoin_split (coin: Value, amount: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t2: Value; // LibraCoin_T_type_value()
    var t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t4: Value; // IntegerType()
    var t5: Value; // LibraCoin_T_type_value()
    var t6: Value; // LibraCoin_T_type_value()
    var t7: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(coin);
    assume IsValidU64(amount);

    old_size := local_counter;
    local_counter := local_counter + 8;
    m := UpdateLocal(m, old_size + 0, coin);
    m := UpdateLocal(m, old_size + 1, amount);

    // bytecode translation starts here
    call t3 := BorrowLoc(old_size+0);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 4, tmp);

    call t5 := LibraCoin_withdraw(t3, GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t5);

    m := UpdateLocal(m, old_size + 5, t5);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 7, tmp);

    ret0 := GetLocal(m, old_size + 6);
    ret1 := GetLocal(m, old_size + 7);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
}

procedure LibraCoin_split_verify (coin: Value, amount: Value) returns (ret0: Value, ret1: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0, ret1 := LibraCoin_split(coin, amount);
}

procedure {:inline 1} LibraCoin_withdraw (coin_ref: Reference, amount: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, coin_ref));
    assume IsValidReferenceParameter(m, local_counter, coin_ref);
    assume IsValidU64(amount);

    old_size := local_counter;
    local_counter := local_counter + 18;
    m := UpdateLocal(m, old_size + 1, amount);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(coin_ref);

    call t4 := BorrowField(t3, LibraCoin_T_value);

    call tmp := ReadRef(t4);
    assume IsValidU64(tmp);

    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := Ge(GetLocal(m, old_size + 6), GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := Not(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 9, tmp);

    tmp := GetLocal(m, old_size + 9);
    if (!b#Boolean(tmp)) { goto Label_11; }

    call tmp := LdConst(10);
    m := UpdateLocal(m, old_size + 10, tmp);

    goto Label_Abort;

Label_11:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 12, tmp);

    call tmp := Sub(GetLocal(m, old_size + 11), GetLocal(m, old_size + 12));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 13, tmp);

    call t14 := CopyOrMoveRef(coin_ref);

    call t15 := BorrowField(t14, LibraCoin_T_value);

    call WriteRef(t15, GetLocal(m, old_size + 13));

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 16, tmp);

    call tmp := Pack_LibraCoin_T(GetLocal(m, old_size + 16));
    m := UpdateLocal(m, old_size + 17, tmp);

    ret0 := GetLocal(m, old_size + 17);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_withdraw_verify (coin_ref: Reference, amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_withdraw(coin_ref, amount);
}

procedure {:inline 1} LibraCoin_join (coin1: Value, coin2: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t2: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t3: Value; // LibraCoin_T_type_value()
    var t4: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(coin1);
    assume is#Vector(coin2);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, coin1);
    m := UpdateLocal(m, old_size + 1, coin2);

    // bytecode translation starts here
    call t2 := BorrowLoc(old_size+0);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call LibraCoin_deposit(t2, GetLocal(m, old_size + 3));
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraCoin_join_verify (coin1: Value, coin2: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraCoin_join(coin1, coin2);
}

procedure {:inline 1} LibraCoin_deposit (coin_ref: Reference, check: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, coin_ref));
    assume IsValidReferenceParameter(m, local_counter, coin_ref);
    assume is#Vector(check);

    old_size := local_counter;
    local_counter := local_counter + 14;
    m := UpdateLocal(m, old_size + 1, check);

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(coin_ref);

    call t5 := BorrowField(t4, LibraCoin_T_value);

    call tmp := ReadRef(t5);
    assume IsValidU64(tmp);

    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    call t8 := Unpack_LibraCoin_T(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 8, t8);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 10, tmp);

    call tmp := AddU64(GetLocal(m, old_size + 9), GetLocal(m, old_size + 10));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 11, tmp);

    call t12 := CopyOrMoveRef(coin_ref);

    call t13 := BorrowField(t12, LibraCoin_T_value);

    call WriteRef(t13, GetLocal(m, old_size + 11));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraCoin_deposit_verify (coin_ref: Reference, check: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraCoin_deposit(coin_ref, check);
}

procedure {:inline 1} LibraCoin_destroy_zero (coin: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(coin);

    old_size := local_counter;
    local_counter := local_counter + 9;
    m := UpdateLocal(m, old_size + 0, coin);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := Unpack_LibraCoin_T(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 3, t3);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 5, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 4), GetLocal(m, old_size + 5)));
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := Not(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 7, tmp);

    tmp := GetLocal(m, old_size + 7);
    if (!b#Boolean(tmp)) { goto Label_10; }

    call tmp := LdConst(11);
    m := UpdateLocal(m, old_size + 8, tmp);

    goto Label_Abort;

Label_10:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraCoin_destroy_zero_verify (coin: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
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
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t2: Value; // AddressType()
    var t3: Value; // LibraCoin_T_type_value()
    var t4: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(payee);
    assume is#Vector(to_deposit);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, payee);
    m := UpdateLocal(m, old_size + 1, to_deposit);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    // unimplemented instruction

    call LibraAccount_deposit_with_metadata(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_deposit_verify (payee: Value, to_deposit: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_deposit(payee, to_deposit);
}

procedure {:inline 1} LibraAccount_deposit_with_metadata (payee: Value, to_deposit: Value, metadata: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t3: Value; // AddressType()
    var t4: Value; // AddressType()
    var t5: Value; // LibraCoin_T_type_value()
    var t6: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(payee);
    assume is#Vector(to_deposit);
    assume is#ByteArray(metadata);

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 0, payee);
    m := UpdateLocal(m, old_size + 1, to_deposit);
    m := UpdateLocal(m, old_size + 2, metadata);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 6, tmp);

    call LibraAccount_deposit_with_sender_and_metadata(GetLocal(m, old_size + 3), GetLocal(m, old_size + 4), GetLocal(m, old_size + 5), GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_deposit_with_metadata_verify (payee: Value, to_deposit: Value, metadata: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_deposit_with_metadata(payee, to_deposit, metadata);
}

procedure {:inline 1} LibraAccount_deposit_with_sender_and_metadata (payee: Value, sender: Value, to_deposit: Value, metadata: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(payee);
    assume is#Address(sender);
    assume is#Vector(to_deposit);
    assume is#ByteArray(metadata);

    old_size := local_counter;
    local_counter := local_counter + 33;
    m := UpdateLocal(m, old_size + 0, payee);
    m := UpdateLocal(m, old_size + 1, sender);
    m := UpdateLocal(m, old_size + 2, to_deposit);
    m := UpdateLocal(m, old_size + 3, metadata);

    // bytecode translation starts here
    call t7 := BorrowLoc(old_size+2);

    call t8 := LibraCoin_value(t7);
    if (abort_flag) { goto Label_Abort; }
    assume IsValidU64(t8);

    m := UpdateLocal(m, old_size + 8, t8);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 10, tmp);

    call tmp := Gt(GetLocal(m, old_size + 9), GetLocal(m, old_size + 10));
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := Not(GetLocal(m, old_size + 11));
    m := UpdateLocal(m, old_size + 12, tmp);

    tmp := GetLocal(m, old_size + 12);
    if (!b#Boolean(tmp)) { goto Label_10; }

    call tmp := LdConst(7);
    m := UpdateLocal(m, old_size + 13, tmp);

    goto Label_Abort;

Label_10:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 14, tmp);

    call t15 := BorrowGlobal(GetLocal(m, old_size + 14), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t6 := CopyOrMoveRef(t15);

    call t16 := CopyOrMoveRef(t6);

    call t17 := BorrowField(t16, LibraAccount_T_sent_events);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 18, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 19, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 20, tmp);

    call tmp := Pack_LibraAccount_SentPaymentEvent(GetLocal(m, old_size + 18), GetLocal(m, old_size + 19), GetLocal(m, old_size + 20));
    m := UpdateLocal(m, old_size + 21, tmp);

    call LibraAccount_emit_event(LibraAccount_SentPaymentEvent_type_value(), t17, GetLocal(m, old_size + 21));
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 22, tmp);

    call t23 := BorrowGlobal(GetLocal(m, old_size + 22), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t5 := CopyOrMoveRef(t23);

    call t24 := CopyOrMoveRef(t5);

    call t25 := BorrowField(t24, LibraAccount_T_balance);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 26, tmp);

    call LibraCoin_deposit(t25, GetLocal(m, old_size + 26));
    if (abort_flag) { goto Label_Abort; }

    call t27 := CopyOrMoveRef(t5);

    call t28 := BorrowField(t27, LibraAccount_T_received_events);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 29, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 30, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 31, tmp);

    call tmp := Pack_LibraAccount_ReceivedPaymentEvent(GetLocal(m, old_size + 29), GetLocal(m, old_size + 30), GetLocal(m, old_size + 31));
    m := UpdateLocal(m, old_size + 32, tmp);

    call LibraAccount_emit_event(LibraAccount_ReceivedPaymentEvent_type_value(), t28, GetLocal(m, old_size + 32));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_deposit_with_sender_and_metadata_verify (payee: Value, sender: Value, to_deposit: Value, metadata: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_deposit_with_sender_and_metadata(payee, sender, to_deposit, metadata);
}

procedure {:inline 1} LibraAccount_mint_to_address (payee: Value, amount: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t2: Value; // AddressType()
    var t3: Value; // BooleanType()
    var t4: Value; // BooleanType()
    var t5: Value; // AddressType()
    var t6: Value; // AddressType()
    var t7: Value; // IntegerType()
    var t8: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(payee);
    assume IsValidU64(amount);

    old_size := local_counter;
    local_counter := local_counter + 9;
    m := UpdateLocal(m, old_size + 0, payee);
    m := UpdateLocal(m, old_size + 1, amount);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := Exists(GetLocal(m, old_size + 2), LibraAccount_T_type_value());
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := Not(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, tmp);

    tmp := GetLocal(m, old_size + 4);
    if (!b#Boolean(tmp)) { goto Label_6; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 5, tmp);

    call LibraAccount_create_account(GetLocal(m, old_size + 5));
    if (abort_flag) { goto Label_Abort; }

Label_6:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    call t8 := LibraCoin_mint_with_default_capability(GetLocal(m, old_size + 7));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t8);

    m := UpdateLocal(m, old_size + 8, t8);

    call LibraAccount_deposit(GetLocal(m, old_size + 6), GetLocal(m, old_size + 8));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_mint_to_address_verify (payee: Value, amount: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_mint_to_address(payee, amount);
}

procedure {:inline 1} LibraAccount_withdraw_from_account (account: Reference, amount: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t2: Value; // LibraCoin_T_type_value()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t5: Value; // IntegerType()
    var t6: Value; // LibraCoin_T_type_value()
    var t7: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, account));
    assume IsValidReferenceParameter(m, local_counter, account);
    assume IsValidU64(amount);

    old_size := local_counter;
    local_counter := local_counter + 8;
    m := UpdateLocal(m, old_size + 1, amount);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(account);

    call t4 := BorrowField(t3, LibraAccount_T_balance);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    call t6 := LibraCoin_withdraw(t4, GetLocal(m, old_size + 5));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t6);

    m := UpdateLocal(m, old_size + 6, t6);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 7, tmp);

    ret0 := GetLocal(m, old_size + 7);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_from_account_verify (account: Reference, amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_withdraw_from_account(account, amount);
}

procedure {:inline 1} LibraAccount_withdraw_from_sender (amount: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(amount);

    old_size := local_counter;
    local_counter := local_counter + 11;
    m := UpdateLocal(m, old_size + 0, amount);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := BorrowGlobal(GetLocal(m, old_size + 2), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, LibraAccount_T_delegated_withdrawal_capability);

    call tmp := ReadRef(t5);
    assume is#Boolean(tmp);

    m := UpdateLocal(m, old_size + 6, tmp);

    tmp := GetLocal(m, old_size + 6);
    if (!b#Boolean(tmp)) { goto Label_9; }

    call tmp := LdConst(11);
    m := UpdateLocal(m, old_size + 7, tmp);

    goto Label_Abort;

Label_9:
    call t8 := CopyOrMoveRef(t1);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 9, tmp);

    call t10 := LibraAccount_withdraw_from_account(t8, GetLocal(m, old_size + 9));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t10);

    m := UpdateLocal(m, old_size + 10, t10);

    ret0 := GetLocal(m, old_size + 10);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_from_sender_verify (amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_withdraw_from_sender(amount);
}

procedure {:inline 1} LibraAccount_withdraw_with_capability (cap: Reference, amount: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, cap));
    assume IsValidReferenceParameter(m, local_counter, cap);
    assume IsValidU64(amount);

    old_size := local_counter;
    local_counter := local_counter + 10;
    m := UpdateLocal(m, old_size + 1, amount);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(cap);

    call t4 := BorrowField(t3, LibraAccount_WithdrawalCapability_account_address);

    call tmp := ReadRef(t4);
    assume is#Address(tmp);

    m := UpdateLocal(m, old_size + 5, tmp);

    call t6 := BorrowGlobal(GetLocal(m, old_size + 5), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t6);

    call t7 := CopyOrMoveRef(t2);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 8, tmp);

    call t9 := LibraAccount_withdraw_from_account(t7, GetLocal(m, old_size + 8));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t9);

    m := UpdateLocal(m, old_size + 9, t9);

    ret0 := GetLocal(m, old_size + 9);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_withdraw_with_capability_verify (cap: Reference, amount: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_withdraw_with_capability(cap, amount);
}

procedure {:inline 1} LibraAccount_extract_sender_withdrawal_capability () returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 15;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    call t5 := BorrowGlobal(GetLocal(m, old_size + 4), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, LibraAccount_T_delegated_withdrawal_capability);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t8);
    assume is#Boolean(tmp);

    m := UpdateLocal(m, old_size + 9, tmp);

    tmp := GetLocal(m, old_size + 9);
    if (!b#Boolean(tmp)) { goto Label_13; }

    call tmp := LdConst(11);
    m := UpdateLocal(m, old_size + 10, tmp);

    goto Label_Abort;

Label_13:
    call tmp := LdTrue();
    m := UpdateLocal(m, old_size + 11, tmp);

    call t12 := CopyOrMoveRef(t2);

    call WriteRef(t12, GetLocal(m, old_size + 11));

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 13, tmp);

    call tmp := Pack_LibraAccount_WithdrawalCapability(GetLocal(m, old_size + 13));
    m := UpdateLocal(m, old_size + 14, tmp);

    ret0 := GetLocal(m, old_size + 14);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_extract_sender_withdrawal_capability_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_extract_sender_withdrawal_capability();
}

procedure {:inline 1} LibraAccount_restore_withdrawal_capability (cap: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(cap);

    old_size := local_counter;
    local_counter := local_counter + 10;
    m := UpdateLocal(m, old_size + 0, cap);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call t4 := Unpack_LibraAccount_WithdrawalCapability(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, t4);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    call t6 := BorrowGlobal(GetLocal(m, old_size + 5), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t6);

    call tmp := LdFalse();
    m := UpdateLocal(m, old_size + 7, tmp);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, LibraAccount_T_delegated_withdrawal_capability);

    call WriteRef(t9, GetLocal(m, old_size + 7));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_restore_withdrawal_capability_verify (cap: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_restore_withdrawal_capability(cap);
}

procedure {:inline 1} LibraAccount_pay_from_capability (payee: Value, cap: Reference, amount: Value, metadata: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(payee);
    assume is#Vector(Dereference(m, cap));
    assume IsValidReferenceParameter(m, local_counter, cap);
    assume IsValidU64(amount);
    assume is#ByteArray(metadata);

    old_size := local_counter;
    local_counter := local_counter + 16;
    m := UpdateLocal(m, old_size + 0, payee);
    m := UpdateLocal(m, old_size + 2, amount);
    m := UpdateLocal(m, old_size + 3, metadata);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Exists(GetLocal(m, old_size + 4), LibraAccount_T_type_value());
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := Not(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 6, tmp);

    tmp := GetLocal(m, old_size + 6);
    if (!b#Boolean(tmp)) { goto Label_6; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 7, tmp);

    call LibraAccount_create_account(GetLocal(m, old_size + 7));
    if (abort_flag) { goto Label_Abort; }

Label_6:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 8, tmp);

    call t9 := CopyOrMoveRef(cap);

    call t10 := BorrowField(t9, LibraAccount_WithdrawalCapability_account_address);

    call tmp := ReadRef(t10);
    assume is#Address(tmp);

    m := UpdateLocal(m, old_size + 11, tmp);

    call t12 := CopyOrMoveRef(cap);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 13, tmp);

    call t14 := LibraAccount_withdraw_with_capability(t12, GetLocal(m, old_size + 13));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t14);

    m := UpdateLocal(m, old_size + 14, t14);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 15, tmp);

    call LibraAccount_deposit_with_sender_and_metadata(GetLocal(m, old_size + 8), GetLocal(m, old_size + 11), GetLocal(m, old_size + 14), GetLocal(m, old_size + 15));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_pay_from_capability_verify (payee: Value, cap: Reference, amount: Value, metadata: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_pay_from_capability(payee, cap, amount, metadata);
}

procedure {:inline 1} LibraAccount_pay_from_sender_with_metadata (payee: Value, amount: Value, metadata: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(payee);
    assume IsValidU64(amount);
    assume is#ByteArray(metadata);

    old_size := local_counter;
    local_counter := local_counter + 11;
    m := UpdateLocal(m, old_size + 0, payee);
    m := UpdateLocal(m, old_size + 1, amount);
    m := UpdateLocal(m, old_size + 2, metadata);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := Exists(GetLocal(m, old_size + 3), LibraAccount_T_type_value());
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Not(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);

    tmp := GetLocal(m, old_size + 5);
    if (!b#Boolean(tmp)) { goto Label_6; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 6, tmp);

    call LibraAccount_create_account(GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }

Label_6:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 7, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 8, tmp);

    call t9 := LibraAccount_withdraw_from_sender(GetLocal(m, old_size + 8));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t9);

    m := UpdateLocal(m, old_size + 9, t9);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 10, tmp);

    call LibraAccount_deposit_with_metadata(GetLocal(m, old_size + 7), GetLocal(m, old_size + 9), GetLocal(m, old_size + 10));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_pay_from_sender_with_metadata_verify (payee: Value, amount: Value, metadata: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_pay_from_sender_with_metadata(payee, amount, metadata);
}

procedure {:inline 1} LibraAccount_pay_from_sender (payee: Value, amount: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t2: Value; // AddressType()
    var t3: Value; // IntegerType()
    var t4: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(payee);
    assume IsValidU64(amount);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, payee);
    m := UpdateLocal(m, old_size + 1, amount);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    // unimplemented instruction

    call LibraAccount_pay_from_sender_with_metadata(GetLocal(m, old_size + 2), GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_pay_from_sender_verify (payee: Value, amount: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_pay_from_sender(payee, amount);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key_for_account (account: Reference, new_authentication_key: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t2: Value; // ByteArrayType()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(ByteArrayType())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, account));
    assume IsValidReferenceParameter(m, local_counter, account);
    assume is#ByteArray(new_authentication_key);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 1, new_authentication_key);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := CopyOrMoveRef(account);

    call t4 := BorrowField(t3, LibraAccount_T_authentication_key);

    call WriteRef(t4, GetLocal(m, old_size + 2));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_rotate_authentication_key_for_account_verify (account: Reference, new_authentication_key: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_rotate_authentication_key_for_account(account, new_authentication_key);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key (new_authentication_key: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#ByteArray(new_authentication_key);

    old_size := local_counter;
    local_counter := local_counter + 10;
    m := UpdateLocal(m, old_size + 0, new_authentication_key);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := BorrowGlobal(GetLocal(m, old_size + 2), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, LibraAccount_T_delegated_key_rotation_capability);

    call tmp := ReadRef(t5);
    assume is#Boolean(tmp);

    m := UpdateLocal(m, old_size + 6, tmp);

    tmp := GetLocal(m, old_size + 6);
    if (!b#Boolean(tmp)) { goto Label_9; }

    call tmp := LdConst(11);
    m := UpdateLocal(m, old_size + 7, tmp);

    goto Label_Abort;

Label_9:
    call t8 := CopyOrMoveRef(t1);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 9, tmp);

    call LibraAccount_rotate_authentication_key_for_account(t8, GetLocal(m, old_size + 9));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_rotate_authentication_key_verify (new_authentication_key: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_rotate_authentication_key(new_authentication_key);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key_with_capability (cap: Reference, new_authentication_key: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t2: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var t3: Reference; // ReferenceType(AddressType())
    var t4: Value; // AddressType()
    var t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t6: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, cap));
    assume IsValidReferenceParameter(m, local_counter, cap);
    assume is#ByteArray(new_authentication_key);

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 1, new_authentication_key);

    // bytecode translation starts here
    call t2 := CopyOrMoveRef(cap);

    call t3 := BorrowField(t2, LibraAccount_KeyRotationCapability_account_address);

    call tmp := ReadRef(t3);
    assume is#Address(tmp);

    m := UpdateLocal(m, old_size + 4, tmp);

    call t5 := BorrowGlobal(GetLocal(m, old_size + 4), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 6, tmp);

    call LibraAccount_rotate_authentication_key_for_account(t5, GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_rotate_authentication_key_with_capability_verify (cap: Reference, new_authentication_key: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_rotate_authentication_key_with_capability(cap, new_authentication_key);
}

procedure {:inline 1} LibraAccount_extract_sender_key_rotation_capability () returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 13;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call t4 := BorrowGlobal(GetLocal(m, old_size + 3), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t5 := BorrowField(t4, LibraAccount_T_delegated_key_rotation_capability);

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call tmp := ReadRef(t6);
    assume is#Boolean(tmp);

    m := UpdateLocal(m, old_size + 7, tmp);

    tmp := GetLocal(m, old_size + 7);
    if (!b#Boolean(tmp)) { goto Label_11; }

    call tmp := LdConst(11);
    m := UpdateLocal(m, old_size + 8, tmp);

    goto Label_Abort;

Label_11:
    call tmp := LdTrue();
    m := UpdateLocal(m, old_size + 9, tmp);

    call t10 := CopyOrMoveRef(t1);

    call WriteRef(t10, GetLocal(m, old_size + 9));

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := Pack_LibraAccount_KeyRotationCapability(GetLocal(m, old_size + 11));
    m := UpdateLocal(m, old_size + 12, tmp);

    ret0 := GetLocal(m, old_size + 12);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_extract_sender_key_rotation_capability_verify () returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_extract_sender_key_rotation_capability();
}

procedure {:inline 1} LibraAccount_restore_key_rotation_capability (cap: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(cap);

    old_size := local_counter;
    local_counter := local_counter + 10;
    m := UpdateLocal(m, old_size + 0, cap);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call t4 := Unpack_LibraAccount_KeyRotationCapability(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, t4);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    call t6 := BorrowGlobal(GetLocal(m, old_size + 5), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t2 := CopyOrMoveRef(t6);

    call tmp := LdFalse();
    m := UpdateLocal(m, old_size + 7, tmp);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, LibraAccount_T_delegated_key_rotation_capability);

    call WriteRef(t9, GetLocal(m, old_size + 7));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_restore_key_rotation_capability_verify (cap: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_restore_key_rotation_capability(cap);
}

procedure {:inline 1} LibraAccount_create_account (fresh_address: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(fresh_address);

    old_size := local_counter;
    local_counter := local_counter + 19;
    m := UpdateLocal(m, old_size + 0, fresh_address);

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := Pack_LibraAccount_EventHandleGenerator(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 5, tmp);

    call t6 := AddressUtil_address_to_bytes(GetLocal(m, old_size + 5));
    if (abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t6);

    m := UpdateLocal(m, old_size + 6, t6);

    call t7 := LibraCoin_zero();
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t7);

    m := UpdateLocal(m, old_size + 7, t7);

    call tmp := LdFalse();
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := LdFalse();
    m := UpdateLocal(m, old_size + 9, tmp);

    call t10 := BorrowLoc(old_size+1);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 11, tmp);

    call t12 := LibraAccount_new_event_handle_impl(LibraAccount_ReceivedPaymentEvent_type_value(), t10, GetLocal(m, old_size + 11));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t12);

    m := UpdateLocal(m, old_size + 12, t12);

    call t13 := BorrowLoc(old_size+1);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 14, tmp);

    call t15 := LibraAccount_new_event_handle_impl(LibraAccount_SentPaymentEvent_type_value(), t13, GetLocal(m, old_size + 14));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t15);

    m := UpdateLocal(m, old_size + 15, t15);

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 16, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 17, tmp);

    call tmp := Pack_LibraAccount_T(GetLocal(m, old_size + 6), GetLocal(m, old_size + 7), GetLocal(m, old_size + 8), GetLocal(m, old_size + 9), GetLocal(m, old_size + 12), GetLocal(m, old_size + 15), GetLocal(m, old_size + 16), GetLocal(m, old_size + 17));
    m := UpdateLocal(m, old_size + 18, tmp);

    call LibraAccount_save_account(GetLocal(m, old_size + 4), GetLocal(m, old_size + 18));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_create_account_verify (fresh_address: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_create_account(fresh_address);
}

procedure {:inline 1} LibraAccount_create_new_account (fresh_address: Value, initial_balance: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t2: Value; // AddressType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // BooleanType()
    var t6: Value; // AddressType()
    var t7: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(fresh_address);
    assume IsValidU64(initial_balance);

    old_size := local_counter;
    local_counter := local_counter + 8;
    m := UpdateLocal(m, old_size + 0, fresh_address);
    m := UpdateLocal(m, old_size + 1, initial_balance);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 2, tmp);

    call LibraAccount_create_account(GetLocal(m, old_size + 2));
    if (abort_flag) { goto Label_Abort; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Gt(GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 5, tmp);

    tmp := GetLocal(m, old_size + 5);
    if (!b#Boolean(tmp)) { goto Label_9; }

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 7, tmp);

    call LibraAccount_pay_from_sender(GetLocal(m, old_size + 6), GetLocal(m, old_size + 7));
    if (abort_flag) { goto Label_Abort; }

Label_9:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_create_new_account_verify (fresh_address: Value, initial_balance: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_create_new_account(fresh_address, initial_balance);
}

procedure {:inline 1} LibraAccount_save_account (addr: Value, account: Value) returns ();
requires ExistsTxnSenderAccount(m, txn);

procedure {:inline 1} LibraAccount_balance_for_account (account: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, account));
    assume IsValidReferenceParameter(m, local_counter, account);

    old_size := local_counter;
    local_counter := local_counter + 6;

    // bytecode translation starts here
    call t2 := CopyOrMoveRef(account);

    call t3 := BorrowField(t2, LibraAccount_T_balance);

    call t4 := LibraCoin_value(t3);
    if (abort_flag) { goto Label_Abort; }
    assume IsValidU64(t4);

    m := UpdateLocal(m, old_size + 4, t4);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 5, tmp);

    ret0 := GetLocal(m, old_size + 5);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_balance_for_account_verify (account: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_balance_for_account(account);
}

procedure {:inline 1} LibraAccount_balance (addr: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(addr);

    old_size := local_counter;
    local_counter := local_counter + 4;
    m := UpdateLocal(m, old_size + 0, addr);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call t2 := BorrowGlobal(GetLocal(m, old_size + 1), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t3 := LibraAccount_balance_for_account(t2);
    if (abort_flag) { goto Label_Abort; }
    assume IsValidU64(t3);

    m := UpdateLocal(m, old_size + 3, t3);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_balance_verify (addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_balance(addr);
}

procedure {:inline 1} LibraAccount_sequence_number_for_account (account: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, account));
    assume IsValidReferenceParameter(m, local_counter, account);

    old_size := local_counter;
    local_counter := local_counter + 4;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(account);

    call t2 := BorrowField(t1, LibraAccount_T_sequence_number);

    call tmp := ReadRef(t2);
    assume IsValidU64(tmp);

    m := UpdateLocal(m, old_size + 3, tmp);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_sequence_number_for_account_verify (account: Reference) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_sequence_number_for_account(account);
}

procedure {:inline 1} LibraAccount_sequence_number (addr: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(addr);

    old_size := local_counter;
    local_counter := local_counter + 4;
    m := UpdateLocal(m, old_size + 0, addr);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call t2 := BorrowGlobal(GetLocal(m, old_size + 1), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t3 := LibraAccount_sequence_number_for_account(t2);
    if (abort_flag) { goto Label_Abort; }
    assume IsValidU64(t3);

    m := UpdateLocal(m, old_size + 3, t3);

    ret0 := GetLocal(m, old_size + 3);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_sequence_number_verify (addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_sequence_number(addr);
}

procedure {:inline 1} LibraAccount_delegated_key_rotation_capability (addr: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(BooleanType())
    var t4: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(addr);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, addr);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call t2 := BorrowGlobal(GetLocal(m, old_size + 1), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t3 := BorrowField(t2, LibraAccount_T_delegated_key_rotation_capability);

    call tmp := ReadRef(t3);
    assume is#Boolean(tmp);

    m := UpdateLocal(m, old_size + 4, tmp);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_delegated_key_rotation_capability_verify (addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_delegated_key_rotation_capability(addr);
}

procedure {:inline 1} LibraAccount_delegated_withdrawal_capability (addr: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(BooleanType())
    var t4: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(addr);

    old_size := local_counter;
    local_counter := local_counter + 5;
    m := UpdateLocal(m, old_size + 0, addr);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call t2 := BorrowGlobal(GetLocal(m, old_size + 1), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t3 := BorrowField(t2, LibraAccount_T_delegated_withdrawal_capability);

    call tmp := ReadRef(t3);
    assume is#Boolean(tmp);

    m := UpdateLocal(m, old_size + 4, tmp);

    ret0 := GetLocal(m, old_size + 4);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_delegated_withdrawal_capability_verify (addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_delegated_withdrawal_capability(addr);
}

procedure {:inline 1} LibraAccount_withdrawal_capability_address (cap: Reference) returns (ret0: Reference)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t2: Reference; // ReferenceType(AddressType())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, cap));
    assume IsValidReferenceParameter(m, local_counter, cap);

    old_size := local_counter;
    local_counter := local_counter + 3;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(cap);

    call t2 := BorrowField(t1, LibraAccount_WithdrawalCapability_account_address);

    ret0 := t2;
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultReference;
}

procedure LibraAccount_withdrawal_capability_address_verify (cap: Reference) returns (ret0: Reference)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_withdrawal_capability_address(cap);
}

procedure {:inline 1} LibraAccount_key_rotation_capability_address (cap: Reference) returns (ret0: Reference)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var t2: Reference; // ReferenceType(AddressType())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, cap));
    assume IsValidReferenceParameter(m, local_counter, cap);

    old_size := local_counter;
    local_counter := local_counter + 3;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(cap);

    call t2 := BorrowField(t1, LibraAccount_KeyRotationCapability_account_address);

    ret0 := t2;
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultReference;
}

procedure LibraAccount_key_rotation_capability_address_verify (cap: Reference) returns (ret0: Reference)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_key_rotation_capability_address(cap);
}

procedure {:inline 1} LibraAccount_exists (check_addr: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // AddressType()
    var t2: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Address(check_addr);

    old_size := local_counter;
    local_counter := local_counter + 3;
    m := UpdateLocal(m, old_size + 0, check_addr);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := Exists(GetLocal(m, old_size + 1), LibraAccount_T_type_value());
    m := UpdateLocal(m, old_size + 2, tmp);

    ret0 := GetLocal(m, old_size + 2);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_exists_verify (check_addr: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_exists(check_addr);
}

procedure {:inline 1} LibraAccount_prologue (txn_sequence_number: Value, txn_public_key: Value, txn_gas_price: Value, txn_max_gas_units: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(txn_sequence_number);
    assume is#ByteArray(txn_public_key);
    assume IsValidU64(txn_gas_price);
    assume IsValidU64(txn_max_gas_units);

    old_size := local_counter;
    local_counter := local_counter + 50;
    m := UpdateLocal(m, old_size + 0, txn_sequence_number);
    m := UpdateLocal(m, old_size + 1, txn_public_key);
    m := UpdateLocal(m, old_size + 2, txn_gas_price);
    m := UpdateLocal(m, old_size + 3, txn_max_gas_units);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 10, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 10));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := Exists(GetLocal(m, old_size + 11), LibraAccount_T_type_value());
    m := UpdateLocal(m, old_size + 12, tmp);

    call tmp := Not(GetLocal(m, old_size + 12));
    m := UpdateLocal(m, old_size + 13, tmp);

    tmp := GetLocal(m, old_size + 13);
    if (!b#Boolean(tmp)) { goto Label_8; }

    call tmp := LdConst(5);
    m := UpdateLocal(m, old_size + 14, tmp);

    goto Label_Abort;

Label_8:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 15, tmp);

    call t16 := BorrowGlobal(GetLocal(m, old_size + 15), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t5 := CopyOrMoveRef(t16);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 17, tmp);

    call t18 := Hash_sha3_256(GetLocal(m, old_size + 17));
    if (abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t18);

    m := UpdateLocal(m, old_size + 18, t18);

    call t19 := CopyOrMoveRef(t5);

    call t20 := BorrowField(t19, LibraAccount_T_authentication_key);

    call tmp := ReadRef(t20);
    assume is#ByteArray(tmp);

    m := UpdateLocal(m, old_size + 21, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 18), GetLocal(m, old_size + 21)));
    m := UpdateLocal(m, old_size + 22, tmp);

    call tmp := Not(GetLocal(m, old_size + 22));
    m := UpdateLocal(m, old_size + 23, tmp);

    tmp := GetLocal(m, old_size + 23);
    if (!b#Boolean(tmp)) { goto Label_21; }

    call tmp := LdConst(2);
    m := UpdateLocal(m, old_size + 24, tmp);

    goto Label_Abort;

Label_21:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 25, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 26, tmp);

    call tmp := MulU64(GetLocal(m, old_size + 25), GetLocal(m, old_size + 26));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 27, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 27));
    m := UpdateLocal(m, old_size + 7, tmp);

    call t28 := CopyOrMoveRef(t5);

    call t29 := FreezeRef(t28);

    call t6 := CopyOrMoveRef(t29);

    call t30 := CopyOrMoveRef(t6);

    call t31 := LibraAccount_balance_for_account(t30);
    if (abort_flag) { goto Label_Abort; }
    assume IsValidU64(t31);

    m := UpdateLocal(m, old_size + 31, t31);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 31));
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 32, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 33, tmp);

    call tmp := Ge(GetLocal(m, old_size + 32), GetLocal(m, old_size + 33));
    m := UpdateLocal(m, old_size + 34, tmp);

    call tmp := Not(GetLocal(m, old_size + 34));
    m := UpdateLocal(m, old_size + 35, tmp);

    tmp := GetLocal(m, old_size + 35);
    if (!b#Boolean(tmp)) { goto Label_38; }

    call tmp := LdConst(6);
    m := UpdateLocal(m, old_size + 36, tmp);

    goto Label_Abort;

Label_38:
    call t37 := CopyOrMoveRef(t5);

    call t38 := BorrowField(t37, LibraAccount_T_sequence_number);

    call tmp := ReadRef(t38);
    assume IsValidU64(tmp);

    m := UpdateLocal(m, old_size + 39, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 39));
    m := UpdateLocal(m, old_size + 9, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 40, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 9));
    m := UpdateLocal(m, old_size + 41, tmp);

    call tmp := Ge(GetLocal(m, old_size + 40), GetLocal(m, old_size + 41));
    m := UpdateLocal(m, old_size + 42, tmp);

    call tmp := Not(GetLocal(m, old_size + 42));
    m := UpdateLocal(m, old_size + 43, tmp);

    tmp := GetLocal(m, old_size + 43);
    if (!b#Boolean(tmp)) { goto Label_49; }

    call tmp := LdConst(3);
    m := UpdateLocal(m, old_size + 44, tmp);

    goto Label_Abort;

Label_49:
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 45, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 9));
    m := UpdateLocal(m, old_size + 46, tmp);

    tmp := Boolean(IsEqual(GetLocal(m, old_size + 45), GetLocal(m, old_size + 46)));
    m := UpdateLocal(m, old_size + 47, tmp);

    call tmp := Not(GetLocal(m, old_size + 47));
    m := UpdateLocal(m, old_size + 48, tmp);

    tmp := GetLocal(m, old_size + 48);
    if (!b#Boolean(tmp)) { goto Label_56; }

    call tmp := LdConst(4);
    m := UpdateLocal(m, old_size + 49, tmp);

    goto Label_Abort;

Label_56:
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_prologue_verify (txn_sequence_number: Value, txn_public_key: Value, txn_gas_price: Value, txn_max_gas_units: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_prologue(txn_sequence_number, txn_public_key, txn_gas_price, txn_max_gas_units);
}

procedure {:inline 1} LibraAccount_epilogue (txn_sequence_number: Value, txn_gas_price: Value, txn_max_gas_units: Value, gas_units_remaining: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidU64(txn_sequence_number);
    assume IsValidU64(txn_gas_price);
    assume IsValidU64(txn_max_gas_units);
    assume IsValidU64(gas_units_remaining);

    old_size := local_counter;
    local_counter := local_counter + 37;
    m := UpdateLocal(m, old_size + 0, txn_sequence_number);
    m := UpdateLocal(m, old_size + 1, txn_gas_price);
    m := UpdateLocal(m, old_size + 2, txn_max_gas_units);
    m := UpdateLocal(m, old_size + 3, gas_units_remaining);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 9, tmp);

    call t10 := BorrowGlobal(GetLocal(m, old_size + 9), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t4 := CopyOrMoveRef(t10);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 12, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 13, tmp);

    call tmp := Sub(GetLocal(m, old_size + 12), GetLocal(m, old_size + 13));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 14, tmp);

    call tmp := MulU64(GetLocal(m, old_size + 11), GetLocal(m, old_size + 14));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 15, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 15));
    m := UpdateLocal(m, old_size + 7, tmp);

    call t16 := CopyOrMoveRef(t4);

    call t17 := FreezeRef(t16);

    call t6 := CopyOrMoveRef(t17);

    call t18 := CopyOrMoveRef(t6);

    call t19 := LibraAccount_balance_for_account(t18);
    if (abort_flag) { goto Label_Abort; }
    assume IsValidU64(t19);

    m := UpdateLocal(m, old_size + 19, t19);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 20, tmp);

    call tmp := Ge(GetLocal(m, old_size + 19), GetLocal(m, old_size + 20));
    m := UpdateLocal(m, old_size + 21, tmp);

    call tmp := Not(GetLocal(m, old_size + 21));
    m := UpdateLocal(m, old_size + 22, tmp);

    tmp := GetLocal(m, old_size + 22);
    if (!b#Boolean(tmp)) { goto Label_20; }

    call tmp := LdConst(6);
    m := UpdateLocal(m, old_size + 23, tmp);

    goto Label_Abort;

Label_20:
    call t24 := CopyOrMoveRef(t4);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 7));
    m := UpdateLocal(m, old_size + 25, tmp);

    call t26 := LibraAccount_withdraw_from_account(t24, GetLocal(m, old_size + 25));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t26);

    m := UpdateLocal(m, old_size + 26, t26);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 26));
    m := UpdateLocal(m, old_size + 8, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 27, tmp);

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 28, tmp);

    call tmp := AddU64(GetLocal(m, old_size + 27), GetLocal(m, old_size + 28));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 29, tmp);

    call t30 := CopyOrMoveRef(t4);

    call t31 := BorrowField(t30, LibraAccount_T_sequence_number);

    call WriteRef(t31, GetLocal(m, old_size + 29));

    call tmp := LdAddr(4078);
    m := UpdateLocal(m, old_size + 32, tmp);

    call t33 := BorrowGlobal(GetLocal(m, old_size + 32), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t5 := CopyOrMoveRef(t33);

    call t34 := CopyOrMoveRef(t5);

    call t35 := BorrowField(t34, LibraAccount_T_balance);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 8));
    m := UpdateLocal(m, old_size + 36, tmp);

    call LibraCoin_deposit(t35, GetLocal(m, old_size + 36));
    if (abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_epilogue_verify (txn_sequence_number: Value, txn_gas_price: Value, txn_max_gas_units: Value, gas_units_remaining: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_epilogue(txn_sequence_number, txn_gas_price, txn_max_gas_units, gas_units_remaining);
}

procedure {:inline 1} LibraAccount_fresh_guid (counter: Reference, sender: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, counter));
    assume IsValidReferenceParameter(m, local_counter, counter);
    assume is#Address(sender);

    old_size := local_counter;
    local_counter := local_counter + 22;
    m := UpdateLocal(m, old_size + 1, sender);

    // bytecode translation starts here
    call t6 := CopyOrMoveRef(counter);

    call t7 := BorrowField(t6, LibraAccount_EventHandleGenerator_counter);

    call t2 := CopyOrMoveRef(t7);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 8, tmp);

    call t9 := AddressUtil_address_to_bytes(GetLocal(m, old_size + 8));
    if (abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t9);

    m := UpdateLocal(m, old_size + 9, t9);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 9));
    m := UpdateLocal(m, old_size + 5, tmp);

    call t10 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t10);
    assume IsValidU64(tmp);

    m := UpdateLocal(m, old_size + 11, tmp);

    call t12 := U64Util_u64_to_bytes(GetLocal(m, old_size + 11));
    if (abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t12);

    m := UpdateLocal(m, old_size + 12, t12);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 12));
    m := UpdateLocal(m, old_size + 3, tmp);

    call t13 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t13);
    assume IsValidU64(tmp);

    m := UpdateLocal(m, old_size + 14, tmp);

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 15, tmp);

    call tmp := AddU64(GetLocal(m, old_size + 14), GetLocal(m, old_size + 15));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 16, tmp);

    call t17 := CopyOrMoveRef(t2);

    call WriteRef(t17, GetLocal(m, old_size + 16));

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 18, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 19, tmp);

    call t20 := BytearrayUtil_bytearray_concat(GetLocal(m, old_size + 18), GetLocal(m, old_size + 19));
    if (abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t20);

    m := UpdateLocal(m, old_size + 20, t20);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 20));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 21, tmp);

    ret0 := GetLocal(m, old_size + 21);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_fresh_guid_verify (counter: Reference, sender: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_fresh_guid(counter, sender);
}

procedure {:inline 1} LibraAccount_new_event_handle_impl (tv0: TypeValue, counter: Reference, sender: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t2: Value; // IntegerType()
    var t3: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t4: Value; // AddressType()
    var t5: Value; // ByteArrayType()
    var t6: Value; // LibraAccount_EventHandle_type_value(tv0)

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, counter));
    assume IsValidReferenceParameter(m, local_counter, counter);
    assume is#Address(sender);

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 1, sender);

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := CopyOrMoveRef(counter);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 4, tmp);

    call t5 := LibraAccount_fresh_guid(t3, GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }
    assume is#ByteArray(t5);

    m := UpdateLocal(m, old_size + 5, t5);

    call tmp := Pack_LibraAccount_EventHandle(tv0, GetLocal(m, old_size + 2), GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 6, tmp);

    ret0 := GetLocal(m, old_size + 6);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_new_event_handle_impl_verify (tv0: TypeValue, counter: Reference, sender: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_new_event_handle_impl(tv0, counter, sender);
}

procedure {:inline 1} LibraAccount_new_event_handle (tv0: TypeValue) returns (ret0: Value)
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 8;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 2, tmp);

    call t3 := BorrowGlobal(GetLocal(m, old_size + 2), LibraAccount_T_type_value());
    if (abort_flag) { goto Label_Abort; }

    call t0 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, LibraAccount_T_event_generator);

    call tmp := GetTxnSenderAddress();
    m := UpdateLocal(m, old_size + 6, tmp);

    call t7 := LibraAccount_new_event_handle_impl(tv0, t5, GetLocal(m, old_size + 6));
    if (abort_flag) { goto Label_Abort; }
    assume is#Vector(t7);

    m := UpdateLocal(m, old_size + 7, t7);

    ret0 := GetLocal(m, old_size + 7);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure LibraAccount_new_event_handle_verify (tv0: TypeValue) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(m, txn);
    call ret0 := LibraAccount_new_event_handle(tv0);
}

procedure {:inline 1} LibraAccount_emit_event (tv0: TypeValue, handle_ref: Reference, msg: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
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

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(Dereference(m, handle_ref));
    assume IsValidReferenceParameter(m, local_counter, handle_ref);

    old_size := local_counter;
    local_counter := local_counter + 18;
    m := UpdateLocal(m, old_size + 1, msg);

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(handle_ref);

    call t5 := BorrowField(t4, LibraAccount_EventHandle_guid);

    call tmp := ReadRef(t5);
    assume is#ByteArray(tmp);

    m := UpdateLocal(m, old_size + 6, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 6));
    m := UpdateLocal(m, old_size + 3, tmp);

    call t7 := CopyOrMoveRef(handle_ref);

    call t8 := BorrowField(t7, LibraAccount_EventHandle_counter);

    call t2 := CopyOrMoveRef(t8);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 9, tmp);

    call t10 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t10);
    assume IsValidU64(tmp);

    m := UpdateLocal(m, old_size + 11, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 12, tmp);

    call LibraAccount_write_to_event_store(tv0, GetLocal(m, old_size + 9), GetLocal(m, old_size + 11), GetLocal(m, old_size + 12));
    if (abort_flag) { goto Label_Abort; }

    call t13 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t13);
    assume IsValidU64(tmp);

    m := UpdateLocal(m, old_size + 14, tmp);

    call tmp := LdConst(1);
    m := UpdateLocal(m, old_size + 15, tmp);

    call tmp := AddU64(GetLocal(m, old_size + 14), GetLocal(m, old_size + 15));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 16, tmp);

    call t17 := CopyOrMoveRef(t2);

    call WriteRef(t17, GetLocal(m, old_size + 16));

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_emit_event_verify (tv0: TypeValue, handle_ref: Reference, msg: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_emit_event(tv0, handle_ref, msg);
}

procedure {:inline 1} LibraAccount_write_to_event_store (tv0: TypeValue, guid: Value, count: Value, msg: Value) returns ();
requires ExistsTxnSenderAccount(m, txn);

procedure {:inline 1} LibraAccount_destroy_handle (tv0: TypeValue, handle: Value) returns ()
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t1: Value; // ByteArrayType()
    var t2: Value; // IntegerType()
    var t3: Value; // LibraAccount_EventHandle_type_value(tv0)
    var t4: Value; // IntegerType()
    var t5: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(handle);

    old_size := local_counter;
    local_counter := local_counter + 6;
    m := UpdateLocal(m, old_size + 0, handle);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call t4, t5 := Unpack_LibraAccount_EventHandle(GetLocal(m, old_size + 3));
    m := UpdateLocal(m, old_size + 4, t4);
    m := UpdateLocal(m, old_size + 5, t5);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 4));
    m := UpdateLocal(m, old_size + 2, tmp);

    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure LibraAccount_destroy_handle_verify (tv0: TypeValue, handle: Value) returns ()
{
    assume ExistsTxnSenderAccount(m, txn);
    call LibraAccount_destroy_handle(tv0, handle);
}



// ** structs of module TestLib



// ** functions of module TestLib
