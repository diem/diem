

// ** structs of module U64Util



// ** structs of module AddressUtil



// ** structs of module BytearrayUtil



// ** structs of module Hash



// ** structs of module Signature



// ** structs of module GasSchedule

const unique GasSchedule_Cost: TypeName;
const GasSchedule_Cost_cpu: FieldName;
axiom GasSchedule_Cost_cpu == 0;
const GasSchedule_Cost_storage: FieldName;
axiom GasSchedule_Cost_storage == 1;
function GasSchedule_Cost_type_value(): TypeValue {
    StructType(GasSchedule_Cost, TypeValueArray(DefaultTypeMap[0 := IntegerType()][1 := IntegerType()], 2))
}

procedure {:inline 1} Pack_GasSchedule_Cost(v0: Value, v1: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    assume has_type(IntegerType(), v1);
    v := Struct(ValueArray(DefaultIntMap[GasSchedule_Cost_cpu := v0][GasSchedule_Cost_storage := v1], 2));
    assume has_type(GasSchedule_Cost_type_value(), v);
}

procedure {:inline 1} Unpack_GasSchedule_Cost(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[GasSchedule_Cost_cpu];
    v1 := smap(v)[GasSchedule_Cost_storage];
}

const unique GasSchedule_T: TypeName;
const GasSchedule_T_instruction_schedule: FieldName;
axiom GasSchedule_T_instruction_schedule == 0;
const GasSchedule_T_native_schedule: FieldName;
axiom GasSchedule_T_native_schedule == 1;
function GasSchedule_T_type_value(): TypeValue {
    StructType(GasSchedule_T, TypeValueArray(DefaultTypeMap[0 := Vector_T_type_value(GasSchedule_Cost_type_value())][1 := Vector_T_type_value(GasSchedule_Cost_type_value())], 2))
}

procedure {:inline 1} Pack_GasSchedule_T(v0: Value, v1: Value) returns (v: Value)
{
    assume has_type(Vector_T_type_value(GasSchedule_Cost_type_value()), v0);
    assume has_type(Vector_T_type_value(GasSchedule_Cost_type_value()), v1);
    v := Struct(ValueArray(DefaultIntMap[GasSchedule_T_instruction_schedule := v0][GasSchedule_T_native_schedule := v1], 2));
    assume has_type(GasSchedule_T_type_value(), v);
}

procedure {:inline 1} Unpack_GasSchedule_T(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[GasSchedule_T_instruction_schedule];
    v1 := smap(v)[GasSchedule_T_native_schedule];
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
    StructType(ValidatorConfig_Config, TypeValueArray(DefaultTypeMap[0 := ByteArrayType()][1 := ByteArrayType()][2 := ByteArrayType()][3 := ByteArrayType()][4 := ByteArrayType()][5 := ByteArrayType()], 6))
}

procedure {:inline 1} Pack_ValidatorConfig_Config(v0: Value, v1: Value, v2: Value, v3: Value, v4: Value, v5: Value) returns (v: Value)
{
    assume has_type(ByteArrayType(), v0);
    assume has_type(ByteArrayType(), v1);
    assume has_type(ByteArrayType(), v2);
    assume has_type(ByteArrayType(), v3);
    assume has_type(ByteArrayType(), v4);
    assume has_type(ByteArrayType(), v5);
    v := Struct(ValueArray(DefaultIntMap[ValidatorConfig_Config_consensus_pubkey := v0][ValidatorConfig_Config_validator_network_signing_pubkey := v1][ValidatorConfig_Config_validator_network_identity_pubkey := v2][ValidatorConfig_Config_validator_network_address := v3][ValidatorConfig_Config_fullnodes_network_identity_pubkey := v4][ValidatorConfig_Config_fullnodes_network_address := v5], 6));
    assume has_type(ValidatorConfig_Config_type_value(), v);
}

procedure {:inline 1} Unpack_ValidatorConfig_Config(v: Value) returns (v0: Value, v1: Value, v2: Value, v3: Value, v4: Value, v5: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[ValidatorConfig_Config_consensus_pubkey];
    v1 := smap(v)[ValidatorConfig_Config_validator_network_signing_pubkey];
    v2 := smap(v)[ValidatorConfig_Config_validator_network_identity_pubkey];
    v3 := smap(v)[ValidatorConfig_Config_validator_network_address];
    v4 := smap(v)[ValidatorConfig_Config_fullnodes_network_identity_pubkey];
    v5 := smap(v)[ValidatorConfig_Config_fullnodes_network_address];
}

const unique ValidatorConfig_T: TypeName;
const ValidatorConfig_T_config: FieldName;
axiom ValidatorConfig_T_config == 0;
function ValidatorConfig_T_type_value(): TypeValue {
    StructType(ValidatorConfig_T, TypeValueArray(DefaultTypeMap[0 := ValidatorConfig_Config_type_value()], 1))
}

procedure {:inline 1} Pack_ValidatorConfig_T(v0: Value) returns (v: Value)
{
    assume has_type(ValidatorConfig_Config_type_value(), v0);
    v := Struct(ValueArray(DefaultIntMap[ValidatorConfig_T_config := v0], 1));
    assume has_type(ValidatorConfig_T_type_value(), v);
}

procedure {:inline 1} Unpack_ValidatorConfig_T(v: Value) returns (v0: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[ValidatorConfig_T_config];
}



// ** structs of module LibraCoin

const unique LibraCoin_T: TypeName;
const LibraCoin_T_value: FieldName;
axiom LibraCoin_T_value == 0;
function LibraCoin_T_type_value(): TypeValue {
    StructType(LibraCoin_T, TypeValueArray(DefaultTypeMap[0 := IntegerType()], 1))
}

procedure {:inline 1} Pack_LibraCoin_T(v0: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    v := Struct(ValueArray(DefaultIntMap[LibraCoin_T_value := v0], 1));
    assume has_type(LibraCoin_T_type_value(), v);
}

procedure {:inline 1} Unpack_LibraCoin_T(v: Value) returns (v0: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[LibraCoin_T_value];
}

const unique LibraCoin_MintCapability: TypeName;
function LibraCoin_MintCapability_type_value(): TypeValue {
    StructType(LibraCoin_MintCapability, TypeValueArray(DefaultTypeMap, 0))
}

procedure {:inline 1} Pack_LibraCoin_MintCapability() returns (v: Value)
{
    v := Struct(ValueArray(DefaultIntMap, 0));
    assume has_type(LibraCoin_MintCapability_type_value(), v);
}

procedure {:inline 1} Unpack_LibraCoin_MintCapability(v: Value) returns ()
{
    assert is#Struct(v);
}

const unique LibraCoin_MarketCap: TypeName;
const LibraCoin_MarketCap_total_value: FieldName;
axiom LibraCoin_MarketCap_total_value == 0;
function LibraCoin_MarketCap_type_value(): TypeValue {
    StructType(LibraCoin_MarketCap, TypeValueArray(DefaultTypeMap[0 := IntegerType()], 1))
}

procedure {:inline 1} Pack_LibraCoin_MarketCap(v0: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    v := Struct(ValueArray(DefaultIntMap[LibraCoin_MarketCap_total_value := v0], 1));
    assume has_type(LibraCoin_MarketCap_type_value(), v);
}

procedure {:inline 1} Unpack_LibraCoin_MarketCap(v: Value) returns (v0: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[LibraCoin_MarketCap_total_value];
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
function LibraAccount_T_type_value(): TypeValue {
    StructType(LibraAccount_T, TypeValueArray(DefaultTypeMap[0 := ByteArrayType()][1 := LibraCoin_T_type_value()][2 := BooleanType()][3 := BooleanType()][4 := LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())][5 := LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())][6 := IntegerType()][7 := LibraAccount_EventHandleGenerator_type_value()], 8))
}

procedure {:inline 1} Pack_LibraAccount_T(v0: Value, v1: Value, v2: Value, v3: Value, v4: Value, v5: Value, v6: Value, v7: Value) returns (v: Value)
{
    assume has_type(ByteArrayType(), v0);
    assume has_type(LibraCoin_T_type_value(), v1);
    assume has_type(BooleanType(), v2);
    assume has_type(BooleanType(), v3);
    assume has_type(LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value()), v4);
    assume has_type(LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value()), v5);
    assume has_type(IntegerType(), v6);
    assume has_type(LibraAccount_EventHandleGenerator_type_value(), v7);
    v := Struct(ValueArray(DefaultIntMap[LibraAccount_T_authentication_key := v0][LibraAccount_T_balance := v1][LibraAccount_T_delegated_key_rotation_capability := v2][LibraAccount_T_delegated_withdrawal_capability := v3][LibraAccount_T_received_events := v4][LibraAccount_T_sent_events := v5][LibraAccount_T_sequence_number := v6][LibraAccount_T_event_generator := v7], 8));
    assume has_type(LibraAccount_T_type_value(), v);
}

procedure {:inline 1} Unpack_LibraAccount_T(v: Value) returns (v0: Value, v1: Value, v2: Value, v3: Value, v4: Value, v5: Value, v6: Value, v7: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[LibraAccount_T_authentication_key];
    v1 := smap(v)[LibraAccount_T_balance];
    v2 := smap(v)[LibraAccount_T_delegated_key_rotation_capability];
    v3 := smap(v)[LibraAccount_T_delegated_withdrawal_capability];
    v4 := smap(v)[LibraAccount_T_received_events];
    v5 := smap(v)[LibraAccount_T_sent_events];
    v6 := smap(v)[LibraAccount_T_sequence_number];
    v7 := smap(v)[LibraAccount_T_event_generator];
}

const unique LibraAccount_WithdrawalCapability: TypeName;
const LibraAccount_WithdrawalCapability_account_address: FieldName;
axiom LibraAccount_WithdrawalCapability_account_address == 0;
function LibraAccount_WithdrawalCapability_type_value(): TypeValue {
    StructType(LibraAccount_WithdrawalCapability, TypeValueArray(DefaultTypeMap[0 := AddressType()], 1))
}

procedure {:inline 1} Pack_LibraAccount_WithdrawalCapability(v0: Value) returns (v: Value)
{
    assume has_type(AddressType(), v0);
    v := Struct(ValueArray(DefaultIntMap[LibraAccount_WithdrawalCapability_account_address := v0], 1));
    assume has_type(LibraAccount_WithdrawalCapability_type_value(), v);
}

procedure {:inline 1} Unpack_LibraAccount_WithdrawalCapability(v: Value) returns (v0: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[LibraAccount_WithdrawalCapability_account_address];
}

const unique LibraAccount_KeyRotationCapability: TypeName;
const LibraAccount_KeyRotationCapability_account_address: FieldName;
axiom LibraAccount_KeyRotationCapability_account_address == 0;
function LibraAccount_KeyRotationCapability_type_value(): TypeValue {
    StructType(LibraAccount_KeyRotationCapability, TypeValueArray(DefaultTypeMap[0 := AddressType()], 1))
}

procedure {:inline 1} Pack_LibraAccount_KeyRotationCapability(v0: Value) returns (v: Value)
{
    assume has_type(AddressType(), v0);
    v := Struct(ValueArray(DefaultIntMap[LibraAccount_KeyRotationCapability_account_address := v0], 1));
    assume has_type(LibraAccount_KeyRotationCapability_type_value(), v);
}

procedure {:inline 1} Unpack_LibraAccount_KeyRotationCapability(v: Value) returns (v0: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[LibraAccount_KeyRotationCapability_account_address];
}

const unique LibraAccount_SentPaymentEvent: TypeName;
const LibraAccount_SentPaymentEvent_amount: FieldName;
axiom LibraAccount_SentPaymentEvent_amount == 0;
const LibraAccount_SentPaymentEvent_payee: FieldName;
axiom LibraAccount_SentPaymentEvent_payee == 1;
const LibraAccount_SentPaymentEvent_metadata: FieldName;
axiom LibraAccount_SentPaymentEvent_metadata == 2;
function LibraAccount_SentPaymentEvent_type_value(): TypeValue {
    StructType(LibraAccount_SentPaymentEvent, TypeValueArray(DefaultTypeMap[0 := IntegerType()][1 := AddressType()][2 := ByteArrayType()], 3))
}

procedure {:inline 1} Pack_LibraAccount_SentPaymentEvent(v0: Value, v1: Value, v2: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    assume has_type(AddressType(), v1);
    assume has_type(ByteArrayType(), v2);
    v := Struct(ValueArray(DefaultIntMap[LibraAccount_SentPaymentEvent_amount := v0][LibraAccount_SentPaymentEvent_payee := v1][LibraAccount_SentPaymentEvent_metadata := v2], 3));
    assume has_type(LibraAccount_SentPaymentEvent_type_value(), v);
}

procedure {:inline 1} Unpack_LibraAccount_SentPaymentEvent(v: Value) returns (v0: Value, v1: Value, v2: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[LibraAccount_SentPaymentEvent_amount];
    v1 := smap(v)[LibraAccount_SentPaymentEvent_payee];
    v2 := smap(v)[LibraAccount_SentPaymentEvent_metadata];
}

const unique LibraAccount_ReceivedPaymentEvent: TypeName;
const LibraAccount_ReceivedPaymentEvent_amount: FieldName;
axiom LibraAccount_ReceivedPaymentEvent_amount == 0;
const LibraAccount_ReceivedPaymentEvent_payer: FieldName;
axiom LibraAccount_ReceivedPaymentEvent_payer == 1;
const LibraAccount_ReceivedPaymentEvent_metadata: FieldName;
axiom LibraAccount_ReceivedPaymentEvent_metadata == 2;
function LibraAccount_ReceivedPaymentEvent_type_value(): TypeValue {
    StructType(LibraAccount_ReceivedPaymentEvent, TypeValueArray(DefaultTypeMap[0 := IntegerType()][1 := AddressType()][2 := ByteArrayType()], 3))
}

procedure {:inline 1} Pack_LibraAccount_ReceivedPaymentEvent(v0: Value, v1: Value, v2: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    assume has_type(AddressType(), v1);
    assume has_type(ByteArrayType(), v2);
    v := Struct(ValueArray(DefaultIntMap[LibraAccount_ReceivedPaymentEvent_amount := v0][LibraAccount_ReceivedPaymentEvent_payer := v1][LibraAccount_ReceivedPaymentEvent_metadata := v2], 3));
    assume has_type(LibraAccount_ReceivedPaymentEvent_type_value(), v);
}

procedure {:inline 1} Unpack_LibraAccount_ReceivedPaymentEvent(v: Value) returns (v0: Value, v1: Value, v2: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[LibraAccount_ReceivedPaymentEvent_amount];
    v1 := smap(v)[LibraAccount_ReceivedPaymentEvent_payer];
    v2 := smap(v)[LibraAccount_ReceivedPaymentEvent_metadata];
}

const unique LibraAccount_EventHandleGenerator: TypeName;
const LibraAccount_EventHandleGenerator_counter: FieldName;
axiom LibraAccount_EventHandleGenerator_counter == 0;
function LibraAccount_EventHandleGenerator_type_value(): TypeValue {
    StructType(LibraAccount_EventHandleGenerator, TypeValueArray(DefaultTypeMap[0 := IntegerType()], 1))
}

procedure {:inline 1} Pack_LibraAccount_EventHandleGenerator(v0: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    v := Struct(ValueArray(DefaultIntMap[LibraAccount_EventHandleGenerator_counter := v0], 1));
    assume has_type(LibraAccount_EventHandleGenerator_type_value(), v);
}

procedure {:inline 1} Unpack_LibraAccount_EventHandleGenerator(v: Value) returns (v0: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[LibraAccount_EventHandleGenerator_counter];
}

const unique LibraAccount_EventHandle: TypeName;
const LibraAccount_EventHandle_counter: FieldName;
axiom LibraAccount_EventHandle_counter == 0;
const LibraAccount_EventHandle_guid: FieldName;
axiom LibraAccount_EventHandle_guid == 1;
function LibraAccount_EventHandle_type_value(tv0: TypeValue): TypeValue {
    StructType(LibraAccount_EventHandle, TypeValueArray(DefaultTypeMap[0 := IntegerType()][1 := ByteArrayType()], 2))
}

procedure {:inline 1} Pack_LibraAccount_EventHandle(tv0: TypeValue, v0: Value, v1: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    assume has_type(ByteArrayType(), v1);
    v := Struct(ValueArray(DefaultIntMap[LibraAccount_EventHandle_counter := v0][LibraAccount_EventHandle_guid := v1], 2));
    assume has_type(LibraAccount_EventHandle_type_value(tv0), v);
}

procedure {:inline 1} Unpack_LibraAccount_EventHandle(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[LibraAccount_EventHandle_counter];
    v1 := smap(v)[LibraAccount_EventHandle_guid];
}



// ** structs of module TestLib



// ** stratified functions

procedure {:inline 1} ReadValue0(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        assert false;
    }
}

procedure {:inline 1} ReadValue1(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        e := p#Path(p)[i];
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := ReadValue0(p, i+1, v');
    }
}

procedure {:inline 1} ReadValue2(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        e := p#Path(p)[i];
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := ReadValue1(p, i+1, v');
    }
}

procedure {:inline 1} ReadValueMax(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        e := p#Path(p)[i];
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := ReadValue2(p, i+1, v');
    }
}

procedure {:inline 1} UpdateValue0(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        assert false;
    }
}

procedure {:inline 1} UpdateValue1(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := UpdateValue0(p, i+1, v', new_v);
        if (is#Struct(v)) { v' := mk_struct(smap(v)[f#Field(e) := v'], slen(v));}
        if (is#Vector(v)) { v' := mk_vector(vmap(v)[i#Index(e) := v'], vlen(v));}
    }
}

procedure {:inline 1} UpdateValue2(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := UpdateValue1(p, i+1, v', new_v);
        if (is#Struct(v)) { v' := mk_struct(smap(v)[f#Field(e) := v'], slen(v));}
        if (is#Vector(v)) { v' := mk_vector(vmap(v)[i#Index(e) := v'], vlen(v));}
    }
}

procedure {:inline 1} UpdateValueMax(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := UpdateValue2(p, i+1, v', new_v);
        if (is#Struct(v)) { v' := mk_struct(smap(v)[f#Field(e) := v'], slen(v));}
        if (is#Vector(v)) { v' := mk_vector(vmap(v)[i#Index(e) := v'], vlen(v));}
    }
}



// ** functions of module U64Util

procedure {:inline 1} U64Util_u64_to_bytes (arg0: Value) returns (ret0: Value);

// ** functions of module AddressUtil

procedure {:inline 1} AddressUtil_address_to_bytes (arg0: Value) returns (ret0: Value);

// ** functions of module BytearrayUtil

procedure {:inline 1} BytearrayUtil_bytearray_concat (arg0: Value, arg1: Value) returns (ret0: Value);

// ** functions of module Hash

procedure {:inline 1} Hash_sha2_256 (arg0: Value) returns (ret0: Value);procedure {:inline 1} Hash_sha3_256 (arg0: Value) returns (ret0: Value);

// ** functions of module Signature

procedure {:inline 1} Signature_ed25519_verify (arg0: Value, arg1: Value, arg2: Value) returns (ret0: Value);procedure {:inline 1} Signature_ed25519_threshold_verify (arg0: Value, arg1: Value, arg2: Value, arg3: Value) returns (ret0: Value);

// ** functions of module GasSchedule

procedure {:inline 1} GasSchedule_initialize (arg0: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // GasSchedule_T_type_value()
    var t1: Value; // AddressType()
    var t2: Value; // AddressType()
    var t3: Value; // BooleanType()
    var t4: Value; // BooleanType()
    var t5: Value; // IntegerType()
    var t6: Value; // GasSchedule_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(GasSchedule_T_type_value(), arg0);

    old_size := m_size;
    m_size := m_size + 7;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    tmp := Boolean(is_equal(AddressType(), contents#Memory(m)[old_size+1], contents#Memory(m)[old_size+2]));
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 4];
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    assert false;

Label_7:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call MoveToSender(GasSchedule_T_type_value(), contents#Memory(m)[old_size+6]);

    return;

}

procedure GasSchedule_initialize_verify (arg0: Value) returns ()
{
    call GasSchedule_initialize(arg0);
}

procedure {:inline 1} GasSchedule_instruction_table_size () returns (ret0: Value)
requires ExistsTxnSenderAccount();
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
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 8;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], GasSchedule_T_type_value());

    call t0 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, GasSchedule_T_instruction_schedule);

    call t6 := Vector_length(GasSchedule_Cost_type_value(), t5);
    assume has_type(IntegerType(), t6);

    m := Memory(domain#Memory(m)[old_size+6 := true], contents#Memory(m)[old_size+6 := t6]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+7];
    return;

}

procedure GasSchedule_instruction_table_size_verify () returns (ret0: Value)
{
    call ret0 := GasSchedule_instruction_table_size();
}

procedure {:inline 1} GasSchedule_native_table_size () returns (ret0: Value)
requires ExistsTxnSenderAccount();
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
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 8;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], GasSchedule_T_type_value());

    call t0 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, GasSchedule_T_native_schedule);

    call t6 := Vector_length(GasSchedule_Cost_type_value(), t5);
    assume has_type(IntegerType(), t6);

    m := Memory(domain#Memory(m)[old_size+6 := true], contents#Memory(m)[old_size+6 := t6]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+7];
    return;

}

procedure GasSchedule_native_table_size_verify () returns (ret0: Value)
{
    call ret0 := GasSchedule_native_table_size();
}



// ** functions of module ValidatorConfig

procedure {:inline 1} ValidatorConfig_has (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 3;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := Exists(contents#Memory(m)[old_size+1], ValidatorConfig_T_type_value());
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+2];
    return;

}

procedure ValidatorConfig_has_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := ValidatorConfig_has(arg0);
}

procedure {:inline 1} ValidatorConfig_config (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t4: Reference; // ReferenceType(ValidatorConfig_T_type_value())
    var t5: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t6: Value; // ValidatorConfig_Config_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 7;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], ValidatorConfig_T_type_value());

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, ValidatorConfig_T_config);

    call tmp := ReadRef(t5);
    assume has_type(ValidatorConfig_Config_type_value(), tmp);

    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+6];
    return;

}

procedure ValidatorConfig_config_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := ValidatorConfig_config(arg0);
}

procedure {:inline 1} ValidatorConfig_consensus_pubkey (arg0: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, ValidatorConfig_Config_consensus_pubkey);

    call tmp := ReadRef(t2);
    assume has_type(ByteArrayType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure ValidatorConfig_consensus_pubkey_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := ValidatorConfig_consensus_pubkey(arg0);
}

procedure {:inline 1} ValidatorConfig_validator_network_signing_pubkey (arg0: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, ValidatorConfig_Config_validator_network_signing_pubkey);

    call tmp := ReadRef(t2);
    assume has_type(ByteArrayType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure ValidatorConfig_validator_network_signing_pubkey_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := ValidatorConfig_validator_network_signing_pubkey(arg0);
}

procedure {:inline 1} ValidatorConfig_validator_network_identity_pubkey (arg0: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, ValidatorConfig_Config_validator_network_identity_pubkey);

    call tmp := ReadRef(t2);
    assume has_type(ByteArrayType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure ValidatorConfig_validator_network_identity_pubkey_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := ValidatorConfig_validator_network_identity_pubkey(arg0);
}

procedure {:inline 1} ValidatorConfig_validator_network_address (arg0: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, ValidatorConfig_Config_validator_network_address);

    call tmp := ReadRef(t2);
    assume has_type(ByteArrayType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure ValidatorConfig_validator_network_address_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := ValidatorConfig_validator_network_address(arg0);
}

procedure {:inline 1} ValidatorConfig_fullnodes_network_identity_pubkey (arg0: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, ValidatorConfig_Config_fullnodes_network_identity_pubkey);

    call tmp := ReadRef(t2);
    assume has_type(ByteArrayType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure ValidatorConfig_fullnodes_network_identity_pubkey_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := ValidatorConfig_fullnodes_network_identity_pubkey(arg0);
}

procedure {:inline 1} ValidatorConfig_fullnodes_network_address (arg0: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t1: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, ValidatorConfig_Config_fullnodes_network_address);

    call tmp := ReadRef(t2);
    assume has_type(ByteArrayType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure ValidatorConfig_fullnodes_network_address_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := ValidatorConfig_fullnodes_network_address(arg0);
}

procedure {:inline 1} ValidatorConfig_register_candidate_validator (arg0: Value, arg1: Value, arg2: Value, arg3: Value, arg4: Value, arg5: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // ByteArrayType()
    var t1: Value; // ByteArrayType()
    var t2: Value; // ByteArrayType()
    var t3: Value; // ByteArrayType()
    var t4: Value; // ByteArrayType()
    var t5: Value; // ByteArrayType()
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
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(ByteArrayType(), arg0);
    assume has_type(ByteArrayType(), arg1);
    assume has_type(ByteArrayType(), arg2);
    assume has_type(ByteArrayType(), arg3);
    assume has_type(ByteArrayType(), arg4);
    assume has_type(ByteArrayType(), arg5);

    old_size := m_size;
    m_size := m_size + 14;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size :=  arg2]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size :=  arg3]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size :=  arg4]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size :=  arg5]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+6]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+7]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+8]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+9]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+10]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+11]);

    call tmp := Pack_ValidatorConfig_Config(contents#Memory(m)[old_size+6], contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+8], contents#Memory(m)[old_size+9], contents#Memory(m)[old_size+10], contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    assume has_type(ValidatorConfig_Config_type_value(), contents#Memory(m)[old_size+12]);

    call tmp := Pack_ValidatorConfig_T(contents#Memory(m)[old_size+12]);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    call MoveToSender(ValidatorConfig_T_type_value(), contents#Memory(m)[old_size+13]);

    return;

}

procedure ValidatorConfig_register_candidate_validator_verify (arg0: Value, arg1: Value, arg2: Value, arg3: Value, arg4: Value, arg5: Value) returns ()
{
    call ValidatorConfig_register_candidate_validator(arg0, arg1, arg2, arg3, arg4, arg5);
}

procedure {:inline 1} ValidatorConfig_rotate_consensus_pubkey (arg0: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // ByteArrayType()
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
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(ByteArrayType(), arg0);

    old_size := m_size;
    m_size := m_size + 12;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t5 := BorrowGlobal(contents#Memory(m)[old_size+4], ValidatorConfig_T_type_value());

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, ValidatorConfig_T_config);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, ValidatorConfig_Config_consensus_pubkey);

    call t3 := CopyOrMoveRef(t9);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call t11 := CopyOrMoveRef(t3);

    call WriteRef(t11, contents#Memory(m)[old_size+10]);

    return;

}

procedure ValidatorConfig_rotate_consensus_pubkey_verify (arg0: Value) returns ()
{
    call ValidatorConfig_rotate_consensus_pubkey(arg0);
}

procedure {:inline 1} ValidatorConfig_rotate_validator_network_identity_pubkey (arg0: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // ByteArrayType()
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
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(ByteArrayType(), arg0);

    old_size := m_size;
    m_size := m_size + 12;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t5 := BorrowGlobal(contents#Memory(m)[old_size+4], ValidatorConfig_T_type_value());

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, ValidatorConfig_T_config);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, ValidatorConfig_Config_validator_network_identity_pubkey);

    call t3 := CopyOrMoveRef(t9);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call t11 := CopyOrMoveRef(t3);

    call WriteRef(t11, contents#Memory(m)[old_size+10]);

    return;

}

procedure ValidatorConfig_rotate_validator_network_identity_pubkey_verify (arg0: Value) returns ()
{
    call ValidatorConfig_rotate_validator_network_identity_pubkey(arg0);
}

procedure {:inline 1} ValidatorConfig_rotate_validator_network_address (arg0: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // ByteArrayType()
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
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(ByteArrayType(), arg0);

    old_size := m_size;
    m_size := m_size + 12;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t5 := BorrowGlobal(contents#Memory(m)[old_size+4], ValidatorConfig_T_type_value());

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, ValidatorConfig_T_config);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, ValidatorConfig_Config_validator_network_address);

    call t3 := CopyOrMoveRef(t9);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call t11 := CopyOrMoveRef(t3);

    call WriteRef(t11, contents#Memory(m)[old_size+10]);

    return;

}

procedure ValidatorConfig_rotate_validator_network_address_verify (arg0: Value) returns ()
{
    call ValidatorConfig_rotate_validator_network_address(arg0);
}



// ** functions of module LibraCoin

procedure {:inline 1} LibraCoin_mint_with_default_capability (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(LibraCoin_MintCapability_type_value())
    var t4: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);

    old_size := m_size;
    m_size := m_size + 5;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], LibraCoin_MintCapability_type_value());

    call t4 := LibraCoin_mint(contents#Memory(m)[old_size+1], t3);
    assume has_type(LibraCoin_T_type_value(), t4);

    m := Memory(domain#Memory(m)[old_size+4 := true], contents#Memory(m)[old_size+4 := t4]);

    ret0 := contents#Memory(m)[old_size+4];
    return;

}

procedure LibraCoin_mint_with_default_capability_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraCoin_mint_with_default_capability(arg0);
}

procedure {:inline 1} LibraCoin_mint (arg0: Value, arg1: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Reference; // ReferenceType(LibraCoin_MintCapability_type_value())
    var t2: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var t3: Value; // IntegerType()
    var t4: Reference; // ReferenceType(LibraCoin_MintCapability_type_value())
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // BooleanType()
    var t10: Value; // BooleanType()
    var t11: Value; // IntegerType()
    var t12: Value; // AddressType()
    var t13: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var t14: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var t15: Reference; // ReferenceType(IntegerType())
    var t16: Value; // IntegerType()
    var t17: Value; // IntegerType()
    var t18: Value; // IntegerType()
    var t19: Value; // IntegerType()
    var t20: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var t21: Reference; // ReferenceType(IntegerType())
    var t22: Value; // IntegerType()
    var t23: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);

    old_size := m_size;
    m_size := m_size + 24;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    t1 := arg1;

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(t1);

    // unimplemented instruction

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := LdConst(1000000000);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := LdConst(1000000);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := Mul(contents#Memory(m)[old_size+6], contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := Le(contents#Memory(m)[old_size+5], contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 10];
    if (!b#Boolean(tmp)) { goto Label_11; }

    call tmp := LdConst(11);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    assert false;

Label_11:
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call t13 := BorrowGlobal(contents#Memory(m)[old_size+12], LibraCoin_MarketCap_type_value());

    call t2 := CopyOrMoveRef(t13);

    call t14 := CopyOrMoveRef(t2);

    call t15 := BorrowField(t14, LibraCoin_MarketCap_total_value);

    call tmp := ReadRef(t15);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+16]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+17], contents#Memory(m)[old_size+18]);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    call t20 := CopyOrMoveRef(t2);

    call t21 := BorrowField(t20, LibraCoin_MarketCap_total_value);

    call WriteRef(t21, contents#Memory(m)[old_size+19]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+22]);

    call tmp := Pack_LibraCoin_T(contents#Memory(m)[old_size+22]);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+23];
    return;

}

procedure LibraCoin_mint_verify (arg0: Value, arg1: Reference) returns (ret0: Value)
{
    call ret0 := LibraCoin_mint(arg0, arg1);
}

procedure {:inline 1} LibraCoin_initialize () returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Value; // BooleanType()
    var t3: Value; // BooleanType()
    var t4: Value; // IntegerType()
    var t5: Value; // LibraCoin_MintCapability_type_value()
    var t6: Value; // IntegerType()
    var t7: Value; // LibraCoin_MarketCap_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 8;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    tmp := Boolean(is_equal(AddressType(), contents#Memory(m)[old_size+0], contents#Memory(m)[old_size+1]));
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 3];
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    assert false;

Label_7:
    call tmp := Pack_LibraCoin_MintCapability();
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call MoveToSender(LibraCoin_MintCapability_type_value(), contents#Memory(m)[old_size+5]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+6]);

    call tmp := Pack_LibraCoin_MarketCap(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call MoveToSender(LibraCoin_MarketCap_type_value(), contents#Memory(m)[old_size+7]);

    return;

}

procedure LibraCoin_initialize_verify () returns ()
{
    call LibraCoin_initialize();
}

procedure {:inline 1} LibraCoin_market_cap () returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call t1 := BorrowGlobal(contents#Memory(m)[old_size+0], LibraCoin_MarketCap_type_value());

    call t2 := BorrowField(t1, LibraCoin_MarketCap_total_value);

    call tmp := ReadRef(t2);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure LibraCoin_market_cap_verify () returns (ret0: Value)
{
    call ret0 := LibraCoin_market_cap();
}

procedure {:inline 1} LibraCoin_zero () returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 2;

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+0]);

    call tmp := Pack_LibraCoin_T(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+1];
    return;

}

procedure LibraCoin_zero_verify () returns (ret0: Value)
{
    call ret0 := LibraCoin_zero();
}

procedure {:inline 1} LibraCoin_value (arg0: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t1: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, LibraCoin_T_value);

    call tmp := ReadRef(t2);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure LibraCoin_value_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := LibraCoin_value(arg0);
}

procedure {:inline 1} LibraCoin_split (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // LibraCoin_T_type_value()
    var t1: Value; // IntegerType()
    var t2: Value; // LibraCoin_T_type_value()
    var t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t4: Value; // IntegerType()
    var t5: Value; // LibraCoin_T_type_value()
    var t6: Value; // LibraCoin_T_type_value()
    var t7: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(LibraCoin_T_type_value(), arg0);
    assume has_type(IntegerType(), arg1);

    old_size := m_size;
    m_size := m_size + 8;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t3 := BorrowLoc(old_size+0);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t5 := LibraCoin_withdraw(t3, contents#Memory(m)[old_size+4]);
    assume has_type(LibraCoin_T_type_value(), t5);

    m := Memory(domain#Memory(m)[old_size+5 := true], contents#Memory(m)[old_size+5 := t5]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+6];
    ret1 := contents#Memory(m)[old_size+7];
    return;

}

procedure LibraCoin_split_verify (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
{
    call ret0, ret1 := LibraCoin_split(arg0, arg1);
}

procedure {:inline 1} LibraCoin_withdraw (arg0: Reference, arg1: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t1: Value; // IntegerType()
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
    assume !abort_flag;

    // assume arguments are of correct types
        assume has_type(IntegerType(), arg1);

    old_size := m_size;
    m_size := m_size + 18;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(t0);

    call t4 := BorrowField(t3, LibraCoin_T_value);

    call tmp := ReadRef(t4);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := Ge(contents#Memory(m)[old_size+6], contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 9];
    if (!b#Boolean(tmp)) { goto Label_11; }

    call tmp := LdConst(10);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    assert false;

Label_11:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call tmp := Sub(contents#Memory(m)[old_size+11], contents#Memory(m)[old_size+12]);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    call t14 := CopyOrMoveRef(t0);

    call t15 := BorrowField(t14, LibraCoin_T_value);

    call WriteRef(t15, contents#Memory(m)[old_size+13]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+16]);

    call tmp := Pack_LibraCoin_T(contents#Memory(m)[old_size+16]);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+17];
    return;

}

procedure LibraCoin_withdraw_verify (arg0: Reference, arg1: Value) returns (ret0: Value)
{
    call ret0 := LibraCoin_withdraw(arg0, arg1);
}

procedure {:inline 1} LibraCoin_join (arg0: Value, arg1: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // LibraCoin_T_type_value()
    var t1: Value; // LibraCoin_T_type_value()
    var t2: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t3: Value; // LibraCoin_T_type_value()
    var t4: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(LibraCoin_T_type_value(), arg0);
    assume has_type(LibraCoin_T_type_value(), arg1);

    old_size := m_size;
    m_size := m_size + 5;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t2 := BorrowLoc(old_size+0);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call LibraCoin_deposit(t2, contents#Memory(m)[old_size+3]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+4];
    return;

}

procedure LibraCoin_join_verify (arg0: Value, arg1: Value) returns (ret0: Value)
{
    call ret0 := LibraCoin_join(arg0, arg1);
}

procedure {:inline 1} LibraCoin_deposit (arg0: Reference, arg1: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t1: Value; // LibraCoin_T_type_value()
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
    assume !abort_flag;

    // assume arguments are of correct types
        assume has_type(LibraCoin_T_type_value(), arg1);

    old_size := m_size;
    m_size := m_size + 14;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, LibraCoin_T_value);

    call tmp := ReadRef(t5);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t8 := Unpack_LibraCoin_T(contents#Memory(m)[old_size+7]);
    assume has_type(IntegerType(), t8);

    m := Memory(domain#Memory(m)[old_size+8 := true], contents#Memory(m)[old_size+8 := t8]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+9], contents#Memory(m)[old_size+10]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call t12 := CopyOrMoveRef(t0);

    call t13 := BorrowField(t12, LibraCoin_T_value);

    call WriteRef(t13, contents#Memory(m)[old_size+11]);

    return;

}

procedure LibraCoin_deposit_verify (arg0: Reference, arg1: Value) returns ()
{
    call LibraCoin_deposit(arg0, arg1);
}

procedure {:inline 1} LibraCoin_destroy_zero (arg0: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // LibraCoin_T_type_value()
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
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(LibraCoin_T_type_value(), arg0);

    old_size := m_size;
    m_size := m_size + 9;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := Unpack_LibraCoin_T(contents#Memory(m)[old_size+2]);
    assume has_type(IntegerType(), t3);

    m := Memory(domain#Memory(m)[old_size+3 := true], contents#Memory(m)[old_size+3 := t3]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    tmp := Boolean(is_equal(IntegerType(), contents#Memory(m)[old_size+4], contents#Memory(m)[old_size+5]));
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 7];
    if (!b#Boolean(tmp)) { goto Label_10; }

    call tmp := LdConst(11);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    assert false;

Label_10:
    return;

}

procedure LibraCoin_destroy_zero_verify (arg0: Value) returns ()
{
    call LibraCoin_destroy_zero(arg0);
}



// ** functions of module LibraAccount

procedure {:inline 1} LibraAccount_make (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // LibraCoin_T_type_value()
    var t2: Value; // LibraAccount_EventHandleGenerator_type_value()
    var t3: Value; // LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())
    var t4: Value; // LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())
    var t5: Value; // ByteArrayType()
    var t6: Value; // AddressType()
    var t7: Value; // ByteArrayType()
    var t8: Value; // IntegerType()
    var t9: Value; // LibraAccount_EventHandleGenerator_type_value()
    var t10: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t11: Value; // AddressType()
    var t12: Value; // LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())
    var t13: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t14: Value; // AddressType()
    var t15: Value; // LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())
    var t16: Value; // LibraCoin_T_type_value()
    var t17: Value; // ByteArrayType()
    var t18: Value; // LibraCoin_T_type_value()
    var t19: Value; // BooleanType()
    var t20: Value; // BooleanType()
    var t21: Value; // LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value())
    var t22: Value; // LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value())
    var t23: Value; // IntegerType()
    var t24: Value; // LibraAccount_EventHandleGenerator_type_value()
    var t25: Value; // LibraAccount_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 26;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call t7 := AddressUtil_address_to_bytes(contents#Memory(m)[old_size+6]);
    assume has_type(ByteArrayType(), t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+8]);

    call tmp := Pack_LibraAccount_EventHandleGenerator(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t10 := BorrowLoc(old_size+2);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call t12 := LibraAccount_new_event_handle_impl(LibraAccount_SentPaymentEvent_type_value(), t10, contents#Memory(m)[old_size+11]);
    assume has_type(LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value()), t12);

    m := Memory(domain#Memory(m)[old_size+12 := true], contents#Memory(m)[old_size+12 := t12]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+12]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t13 := BorrowLoc(old_size+2);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call t15 := LibraAccount_new_event_handle_impl(LibraAccount_ReceivedPaymentEvent_type_value(), t13, contents#Memory(m)[old_size+14]);
    assume has_type(LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value()), t15);

    m := Memory(domain#Memory(m)[old_size+15 := true], contents#Memory(m)[old_size+15 := t15]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t16 := LibraCoin_zero();
    assume has_type(LibraCoin_T_type_value(), t16);

    m := Memory(domain#Memory(m)[old_size+16 := true], contents#Memory(m)[old_size+16 := t16]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+16]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[24+old_size := true], contents#Memory(m)[24+old_size := tmp]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+17]);

    assume has_type(LibraCoin_T_type_value(), contents#Memory(m)[old_size+18]);

    assume has_type(BooleanType(), contents#Memory(m)[old_size+19]);

    assume has_type(BooleanType(), contents#Memory(m)[old_size+20]);

    assume has_type(LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value()), contents#Memory(m)[old_size+21]);

    assume has_type(LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value()), contents#Memory(m)[old_size+22]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+23]);

    assume has_type(LibraAccount_EventHandleGenerator_type_value(), contents#Memory(m)[old_size+24]);

    call tmp := Pack_LibraAccount_T(contents#Memory(m)[old_size+17], contents#Memory(m)[old_size+18], contents#Memory(m)[old_size+19], contents#Memory(m)[old_size+20], contents#Memory(m)[old_size+21], contents#Memory(m)[old_size+22], contents#Memory(m)[old_size+23], contents#Memory(m)[old_size+24]);
    m := Memory(domain#Memory(m)[25+old_size := true], contents#Memory(m)[25+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+25];
    return;

}

procedure LibraAccount_make_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_make(arg0);
}

procedure {:inline 1} LibraAccount_deposit (arg0: Value, arg1: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // LibraCoin_T_type_value()
    var t2: Value; // AddressType()
    var t3: Value; // LibraCoin_T_type_value()
    var t4: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);
    assume has_type(LibraCoin_T_type_value(), arg1);

    old_size := m_size;
    m_size := m_size + 5;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    // unimplemented instruction

    call LibraAccount_deposit_with_metadata(contents#Memory(m)[old_size+2], contents#Memory(m)[old_size+3], contents#Memory(m)[old_size+4]);

    return;

}

procedure LibraAccount_deposit_verify (arg0: Value, arg1: Value) returns ()
{
    call LibraAccount_deposit(arg0, arg1);
}

procedure {:inline 1} LibraAccount_deposit_with_metadata (arg0: Value, arg1: Value, arg2: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // LibraCoin_T_type_value()
    var t2: Value; // ByteArrayType()
    var t3: Value; // IntegerType()
    var t4: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t5: Value; // AddressType()
    var t6: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t7: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // IntegerType()
    var t11: Value; // BooleanType()
    var t12: Value; // BooleanType()
    var t13: Value; // IntegerType()
    var t14: Value; // AddressType()
    var t15: Value; // AddressType()
    var t16: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t17: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t18: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value()))
    var t19: Value; // IntegerType()
    var t20: Value; // AddressType()
    var t21: Value; // ByteArrayType()
    var t22: Value; // LibraAccount_SentPaymentEvent_type_value()
    var t23: Value; // AddressType()
    var t24: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t25: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t26: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t27: Value; // LibraCoin_T_type_value()
    var t28: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t29: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value()))
    var t30: Value; // IntegerType()
    var t31: Value; // AddressType()
    var t32: Value; // ByteArrayType()
    var t33: Value; // LibraAccount_ReceivedPaymentEvent_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);
    assume has_type(LibraCoin_T_type_value(), arg1);
    assume has_type(ByteArrayType(), arg2);

    old_size := m_size;
    m_size := m_size + 34;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size :=  arg2]);

    // bytecode translation starts here
    call t7 := BorrowLoc(old_size+1);

    call t8 := LibraCoin_value(t7);
    assume has_type(IntegerType(), t8);

    m := Memory(domain#Memory(m)[old_size+8 := true], contents#Memory(m)[old_size+8 := t8]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := Gt(contents#Memory(m)[old_size+9], contents#Memory(m)[old_size+10]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 12];
    if (!b#Boolean(tmp)) { goto Label_10; }

    call tmp := LdConst(7);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    assert false;

Label_10:
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+14]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call t16 := BorrowGlobal(contents#Memory(m)[old_size+15], LibraAccount_T_type_value());

    call t6 := CopyOrMoveRef(t16);

    call t17 := CopyOrMoveRef(t6);

    call t18 := BorrowField(t17, LibraAccount_T_sent_events);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+19]);

    assume has_type(AddressType(), contents#Memory(m)[old_size+20]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+21]);

    call tmp := Pack_LibraAccount_SentPaymentEvent(contents#Memory(m)[old_size+19], contents#Memory(m)[old_size+20], contents#Memory(m)[old_size+21]);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    call LibraAccount_emit_event(LibraAccount_SentPaymentEvent_type_value(), t18, contents#Memory(m)[old_size+22]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    call t24 := BorrowGlobal(contents#Memory(m)[old_size+23], LibraAccount_T_type_value());

    call t4 := CopyOrMoveRef(t24);

    call t25 := CopyOrMoveRef(t4);

    call t26 := BorrowField(t25, LibraAccount_T_balance);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[27+old_size := true], contents#Memory(m)[27+old_size := tmp]);

    call LibraCoin_deposit(t26, contents#Memory(m)[old_size+27]);

    call t28 := CopyOrMoveRef(t4);

    call t29 := BorrowField(t28, LibraAccount_T_received_events);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[30+old_size := true], contents#Memory(m)[30+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[31+old_size := true], contents#Memory(m)[31+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[32+old_size := true], contents#Memory(m)[32+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+30]);

    assume has_type(AddressType(), contents#Memory(m)[old_size+31]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+32]);

    call tmp := Pack_LibraAccount_ReceivedPaymentEvent(contents#Memory(m)[old_size+30], contents#Memory(m)[old_size+31], contents#Memory(m)[old_size+32]);
    m := Memory(domain#Memory(m)[33+old_size := true], contents#Memory(m)[33+old_size := tmp]);

    call LibraAccount_emit_event(LibraAccount_ReceivedPaymentEvent_type_value(), t29, contents#Memory(m)[old_size+33]);

    return;

}

procedure LibraAccount_deposit_with_metadata_verify (arg0: Value, arg1: Value, arg2: Value) returns ()
{
    call LibraAccount_deposit_with_metadata(arg0, arg1, arg2);
}

procedure {:inline 1} LibraAccount_mint_to_address (arg0: Value, arg1: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // IntegerType()
    var t2: Value; // AddressType()
    var t3: Value; // BooleanType()
    var t4: Value; // BooleanType()
    var t5: Value; // AddressType()
    var t6: Value; // AddressType()
    var t7: Value; // IntegerType()
    var t8: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);
    assume has_type(IntegerType(), arg1);

    old_size := m_size;
    m_size := m_size + 9;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := Exists(contents#Memory(m)[old_size+2], LibraAccount_T_type_value());
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 4];
    if (!b#Boolean(tmp)) { goto Label_6; }

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call LibraAccount_create_account(contents#Memory(m)[old_size+5]);

Label_6:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t8 := LibraCoin_mint_with_default_capability(contents#Memory(m)[old_size+7]);
    assume has_type(LibraCoin_T_type_value(), t8);

    m := Memory(domain#Memory(m)[old_size+8 := true], contents#Memory(m)[old_size+8 := t8]);

    call LibraAccount_deposit(contents#Memory(m)[old_size+6], contents#Memory(m)[old_size+8]);

    return;

}

procedure LibraAccount_mint_to_address_verify (arg0: Value, arg1: Value) returns ()
{
    call LibraAccount_mint_to_address(arg0, arg1);
}

procedure {:inline 1} LibraAccount_withdraw_from_account (arg0: Reference, arg1: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t1: Value; // IntegerType()
    var t2: Value; // LibraCoin_T_type_value()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t5: Value; // IntegerType()
    var t6: Value; // LibraCoin_T_type_value()
    var t7: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
        assume has_type(IntegerType(), arg1);

    old_size := m_size;
    m_size := m_size + 8;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(t0);

    call t4 := BorrowField(t3, LibraAccount_T_balance);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := LibraCoin_withdraw(t4, contents#Memory(m)[old_size+5]);
    assume has_type(LibraCoin_T_type_value(), t6);

    m := Memory(domain#Memory(m)[old_size+6 := true], contents#Memory(m)[old_size+6 := t6]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+7];
    return;

}

procedure LibraAccount_withdraw_from_account_verify (arg0: Reference, arg1: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_withdraw_from_account(arg0, arg1);
}

procedure {:inline 1} LibraAccount_withdraw_from_sender (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // IntegerType()
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
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);

    old_size := m_size;
    m_size := m_size + 11;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], LibraAccount_T_type_value());

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, LibraAccount_T_delegated_withdrawal_capability);

    call tmp := ReadRef(t5);
    assume has_type(BooleanType(), tmp);

    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 6];
    if (!b#Boolean(tmp)) { goto Label_9; }

    call tmp := LdConst(11);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    assert false;

Label_9:
    call t8 := CopyOrMoveRef(t1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call t10 := LibraAccount_withdraw_from_account(t8, contents#Memory(m)[old_size+9]);
    assume has_type(LibraCoin_T_type_value(), t10);

    m := Memory(domain#Memory(m)[old_size+10 := true], contents#Memory(m)[old_size+10 := t10]);

    ret0 := contents#Memory(m)[old_size+10];
    return;

}

procedure LibraAccount_withdraw_from_sender_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_withdraw_from_sender(arg0);
}

procedure {:inline 1} LibraAccount_withdraw_with_capability (arg0: Reference, arg1: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t1: Value; // IntegerType()
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
    assume !abort_flag;

    // assume arguments are of correct types
        assume has_type(IntegerType(), arg1);

    old_size := m_size;
    m_size := m_size + 10;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(t0);

    call t4 := BorrowField(t3, LibraAccount_WithdrawalCapability_account_address);

    call tmp := ReadRef(t4);
    assume has_type(AddressType(), tmp);

    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := BorrowGlobal(contents#Memory(m)[old_size+5], LibraAccount_T_type_value());

    call t2 := CopyOrMoveRef(t6);

    call t7 := CopyOrMoveRef(t2);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call t9 := LibraAccount_withdraw_from_account(t7, contents#Memory(m)[old_size+8]);
    assume has_type(LibraCoin_T_type_value(), t9);

    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    ret0 := contents#Memory(m)[old_size+9];
    return;

}

procedure LibraAccount_withdraw_with_capability_verify (arg0: Reference, arg1: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_withdraw_with_capability(arg0, arg1);
}

procedure {:inline 1} LibraAccount_extract_sender_withdrawal_capability () returns (ret0: Value)
requires ExistsTxnSenderAccount();
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
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 15;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t5 := BorrowGlobal(contents#Memory(m)[old_size+4], LibraAccount_T_type_value());

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, LibraAccount_T_delegated_withdrawal_capability);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t8);
    assume has_type(BooleanType(), tmp);

    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 9];
    if (!b#Boolean(tmp)) { goto Label_13; }

    call tmp := LdConst(11);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    assert false;

Label_13:
    call tmp := LdTrue();
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call t12 := CopyOrMoveRef(t2);

    call WriteRef(t12, contents#Memory(m)[old_size+11]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    assume has_type(AddressType(), contents#Memory(m)[old_size+13]);

    call tmp := Pack_LibraAccount_WithdrawalCapability(contents#Memory(m)[old_size+13]);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+14];
    return;

}

procedure LibraAccount_extract_sender_withdrawal_capability_verify () returns (ret0: Value)
{
    call ret0 := LibraAccount_extract_sender_withdrawal_capability();
}

procedure {:inline 1} LibraAccount_restore_withdrawal_capability (arg0: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // LibraAccount_WithdrawalCapability_type_value()
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
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(LibraAccount_WithdrawalCapability_type_value(), arg0);

    old_size := m_size;
    m_size := m_size + 10;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t4 := Unpack_LibraAccount_WithdrawalCapability(contents#Memory(m)[old_size+3]);
    assume has_type(AddressType(), t4);

    m := Memory(domain#Memory(m)[old_size+4 := true], contents#Memory(m)[old_size+4 := t4]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := BorrowGlobal(contents#Memory(m)[old_size+5], LibraAccount_T_type_value());

    call t2 := CopyOrMoveRef(t6);

    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, LibraAccount_T_delegated_withdrawal_capability);

    call WriteRef(t9, contents#Memory(m)[old_size+7]);

    return;

}

procedure LibraAccount_restore_withdrawal_capability_verify (arg0: Value) returns ()
{
    call LibraAccount_restore_withdrawal_capability(arg0);
}

procedure {:inline 1} LibraAccount_pay_from_sender_with_metadata (arg0: Value, arg1: Value, arg2: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // IntegerType()
    var t2: Value; // ByteArrayType()
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
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);
    assume has_type(IntegerType(), arg1);
    assume has_type(ByteArrayType(), arg2);

    old_size := m_size;
    m_size := m_size + 11;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size :=  arg2]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := Exists(contents#Memory(m)[old_size+3], LibraAccount_T_type_value());
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 5];
    if (!b#Boolean(tmp)) { goto Label_6; }

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call LibraAccount_create_account(contents#Memory(m)[old_size+6]);

Label_6:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call t9 := LibraAccount_withdraw_from_sender(contents#Memory(m)[old_size+8]);
    assume has_type(LibraCoin_T_type_value(), t9);

    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call LibraAccount_deposit_with_metadata(contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+9], contents#Memory(m)[old_size+10]);

    return;

}

procedure LibraAccount_pay_from_sender_with_metadata_verify (arg0: Value, arg1: Value, arg2: Value) returns ()
{
    call LibraAccount_pay_from_sender_with_metadata(arg0, arg1, arg2);
}

procedure {:inline 1} LibraAccount_pay_from_sender (arg0: Value, arg1: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // IntegerType()
    var t2: Value; // AddressType()
    var t3: Value; // IntegerType()
    var t4: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);
    assume has_type(IntegerType(), arg1);

    old_size := m_size;
    m_size := m_size + 5;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    // unimplemented instruction

    call LibraAccount_pay_from_sender_with_metadata(contents#Memory(m)[old_size+2], contents#Memory(m)[old_size+3], contents#Memory(m)[old_size+4]);

    return;

}

procedure LibraAccount_pay_from_sender_verify (arg0: Value, arg1: Value) returns ()
{
    call LibraAccount_pay_from_sender(arg0, arg1);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key_for_account (arg0: Reference, arg1: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t1: Value; // ByteArrayType()
    var t2: Value; // ByteArrayType()
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Reference; // ReferenceType(ByteArrayType())

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
        assume has_type(ByteArrayType(), arg1);

    old_size := m_size;
    m_size := m_size + 5;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := CopyOrMoveRef(t0);

    call t4 := BorrowField(t3, LibraAccount_T_authentication_key);

    call WriteRef(t4, contents#Memory(m)[old_size+2]);

    return;

}

procedure LibraAccount_rotate_authentication_key_for_account_verify (arg0: Reference, arg1: Value) returns ()
{
    call LibraAccount_rotate_authentication_key_for_account(arg0, arg1);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key (arg0: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // ByteArrayType()
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
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(ByteArrayType(), arg0);

    old_size := m_size;
    m_size := m_size + 10;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], LibraAccount_T_type_value());

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, LibraAccount_T_delegated_key_rotation_capability);

    call tmp := ReadRef(t5);
    assume has_type(BooleanType(), tmp);

    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 6];
    if (!b#Boolean(tmp)) { goto Label_9; }

    call tmp := LdConst(11);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    assert false;

Label_9:
    call t8 := CopyOrMoveRef(t1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call LibraAccount_rotate_authentication_key_for_account(t8, contents#Memory(m)[old_size+9]);

    return;

}

procedure LibraAccount_rotate_authentication_key_verify (arg0: Value) returns ()
{
    call LibraAccount_rotate_authentication_key(arg0);
}

procedure {:inline 1} LibraAccount_rotate_authentication_key_with_capability (arg0: Reference, arg1: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var t1: Value; // ByteArrayType()
    var t2: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var t3: Reference; // ReferenceType(AddressType())
    var t4: Value; // AddressType()
    var t5: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t6: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
        assume has_type(ByteArrayType(), arg1);

    old_size := m_size;
    m_size := m_size + 7;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t2 := CopyOrMoveRef(t0);

    call t3 := BorrowField(t2, LibraAccount_KeyRotationCapability_account_address);

    call tmp := ReadRef(t3);
    assume has_type(AddressType(), tmp);

    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t5 := BorrowGlobal(contents#Memory(m)[old_size+4], LibraAccount_T_type_value());

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call LibraAccount_rotate_authentication_key_for_account(t5, contents#Memory(m)[old_size+6]);

    return;

}

procedure LibraAccount_rotate_authentication_key_with_capability_verify (arg0: Reference, arg1: Value) returns ()
{
    call LibraAccount_rotate_authentication_key_with_capability(arg0, arg1);
}

procedure {:inline 1} LibraAccount_extract_sender_key_rotation_capability () returns (ret0: Value)
requires ExistsTxnSenderAccount();
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
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 13;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t4 := BorrowGlobal(contents#Memory(m)[old_size+3], LibraAccount_T_type_value());

    call t5 := BorrowField(t4, LibraAccount_T_delegated_key_rotation_capability);

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call tmp := ReadRef(t6);
    assume has_type(BooleanType(), tmp);

    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 7];
    if (!b#Boolean(tmp)) { goto Label_11; }

    call tmp := LdConst(11);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    assert false;

Label_11:
    call tmp := LdTrue();
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call t10 := CopyOrMoveRef(t1);

    call WriteRef(t10, contents#Memory(m)[old_size+9]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    assume has_type(AddressType(), contents#Memory(m)[old_size+11]);

    call tmp := Pack_LibraAccount_KeyRotationCapability(contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+12];
    return;

}

procedure LibraAccount_extract_sender_key_rotation_capability_verify () returns (ret0: Value)
{
    call ret0 := LibraAccount_extract_sender_key_rotation_capability();
}

procedure {:inline 1} LibraAccount_restore_key_rotation_capability (arg0: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // LibraAccount_KeyRotationCapability_type_value()
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
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(LibraAccount_KeyRotationCapability_type_value(), arg0);

    old_size := m_size;
    m_size := m_size + 10;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t4 := Unpack_LibraAccount_KeyRotationCapability(contents#Memory(m)[old_size+3]);
    assume has_type(AddressType(), t4);

    m := Memory(domain#Memory(m)[old_size+4 := true], contents#Memory(m)[old_size+4 := t4]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := BorrowGlobal(contents#Memory(m)[old_size+5], LibraAccount_T_type_value());

    call t2 := CopyOrMoveRef(t6);

    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t8 := CopyOrMoveRef(t2);

    call t9 := BorrowField(t8, LibraAccount_T_delegated_key_rotation_capability);

    call WriteRef(t9, contents#Memory(m)[old_size+7]);

    return;

}

procedure LibraAccount_restore_key_rotation_capability_verify (arg0: Value) returns ()
{
    call LibraAccount_restore_key_rotation_capability(arg0);
}

procedure {:inline 1} LibraAccount_create_account (arg0: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
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
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 19;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+2]);

    call tmp := Pack_LibraAccount_EventHandleGenerator(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := AddressUtil_address_to_bytes(contents#Memory(m)[old_size+5]);
    assume has_type(ByteArrayType(), t6);

    m := Memory(domain#Memory(m)[old_size+6 := true], contents#Memory(m)[old_size+6 := t6]);

    call t7 := LibraCoin_zero();
    assume has_type(LibraCoin_T_type_value(), t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call t10 := BorrowLoc(old_size+1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call t12 := LibraAccount_new_event_handle_impl(LibraAccount_ReceivedPaymentEvent_type_value(), t10, contents#Memory(m)[old_size+11]);
    assume has_type(LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value()), t12);

    m := Memory(domain#Memory(m)[old_size+12 := true], contents#Memory(m)[old_size+12 := t12]);

    call t13 := BorrowLoc(old_size+1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call t15 := LibraAccount_new_event_handle_impl(LibraAccount_SentPaymentEvent_type_value(), t13, contents#Memory(m)[old_size+14]);
    assume has_type(LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value()), t15);

    m := Memory(domain#Memory(m)[old_size+15 := true], contents#Memory(m)[old_size+15 := t15]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+6]);

    assume has_type(LibraCoin_T_type_value(), contents#Memory(m)[old_size+7]);

    assume has_type(BooleanType(), contents#Memory(m)[old_size+8]);

    assume has_type(BooleanType(), contents#Memory(m)[old_size+9]);

    assume has_type(LibraAccount_EventHandle_type_value(LibraAccount_ReceivedPaymentEvent_type_value()), contents#Memory(m)[old_size+12]);

    assume has_type(LibraAccount_EventHandle_type_value(LibraAccount_SentPaymentEvent_type_value()), contents#Memory(m)[old_size+15]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+16]);

    assume has_type(LibraAccount_EventHandleGenerator_type_value(), contents#Memory(m)[old_size+17]);

    call tmp := Pack_LibraAccount_T(contents#Memory(m)[old_size+6], contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+8], contents#Memory(m)[old_size+9], contents#Memory(m)[old_size+12], contents#Memory(m)[old_size+15], contents#Memory(m)[old_size+16], contents#Memory(m)[old_size+17]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call LibraAccount_save_account(contents#Memory(m)[old_size+4], contents#Memory(m)[old_size+18]);

    return;

}

procedure LibraAccount_create_account_verify (arg0: Value) returns ()
{
    call LibraAccount_create_account(arg0);
}

procedure {:inline 1} LibraAccount_create_new_account (arg0: Value, arg1: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // IntegerType()
    var t2: Value; // AddressType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // BooleanType()
    var t6: Value; // AddressType()
    var t7: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);
    assume has_type(IntegerType(), arg1);

    old_size := m_size;
    m_size := m_size + 8;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call LibraAccount_create_account(contents#Memory(m)[old_size+2]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := Gt(contents#Memory(m)[old_size+3], contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 5];
    if (!b#Boolean(tmp)) { goto Label_9; }

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call LibraAccount_pay_from_sender(contents#Memory(m)[old_size+6], contents#Memory(m)[old_size+7]);

Label_9:
    return;

}

procedure LibraAccount_create_new_account_verify (arg0: Value, arg1: Value) returns ()
{
    call LibraAccount_create_new_account(arg0, arg1);
}

procedure {:inline 1} LibraAccount_save_account (arg0: Value, arg1: Value) returns ();procedure {:inline 1} LibraAccount_balance_for_account (arg0: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t1: Value; // IntegerType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 6;
    t0 := arg0;

    // bytecode translation starts here
    call t2 := CopyOrMoveRef(t0);

    call t3 := BorrowField(t2, LibraAccount_T_balance);

    call t4 := LibraCoin_value(t3);
    assume has_type(IntegerType(), t4);

    m := Memory(domain#Memory(m)[old_size+4 := true], contents#Memory(m)[old_size+4 := t4]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+5];
    return;

}

procedure LibraAccount_balance_for_account_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := LibraAccount_balance_for_account(arg0);
}

procedure {:inline 1} LibraAccount_balance (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 4;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t2 := BorrowGlobal(contents#Memory(m)[old_size+1], LibraAccount_T_type_value());

    call t3 := LibraAccount_balance_for_account(t2);
    assume has_type(IntegerType(), t3);

    m := Memory(domain#Memory(m)[old_size+3 := true], contents#Memory(m)[old_size+3 := t3]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure LibraAccount_balance_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_balance(arg0);
}

procedure {:inline 1} LibraAccount_sequence_number_for_account (arg0: Reference) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t1: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t2: Reference; // ReferenceType(IntegerType())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, LibraAccount_T_sequence_number);

    call tmp := ReadRef(t2);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure LibraAccount_sequence_number_for_account_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := LibraAccount_sequence_number_for_account(arg0);
}

procedure {:inline 1} LibraAccount_sequence_number (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 4;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t2 := BorrowGlobal(contents#Memory(m)[old_size+1], LibraAccount_T_type_value());

    call t3 := LibraAccount_sequence_number_for_account(t2);
    assume has_type(IntegerType(), t3);

    m := Memory(domain#Memory(m)[old_size+3 := true], contents#Memory(m)[old_size+3 := t3]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure LibraAccount_sequence_number_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_sequence_number(arg0);
}

procedure {:inline 1} LibraAccount_delegated_key_rotation_capability (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(BooleanType())
    var t4: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 5;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t2 := BorrowGlobal(contents#Memory(m)[old_size+1], LibraAccount_T_type_value());

    call t3 := BorrowField(t2, LibraAccount_T_delegated_key_rotation_capability);

    call tmp := ReadRef(t3);
    assume has_type(BooleanType(), tmp);

    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+4];
    return;

}

procedure LibraAccount_delegated_key_rotation_capability_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_delegated_key_rotation_capability(arg0);
}

procedure {:inline 1} LibraAccount_delegated_withdrawal_capability (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(BooleanType())
    var t4: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 5;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t2 := BorrowGlobal(contents#Memory(m)[old_size+1], LibraAccount_T_type_value());

    call t3 := BorrowField(t2, LibraAccount_T_delegated_withdrawal_capability);

    call tmp := ReadRef(t3);
    assume has_type(BooleanType(), tmp);

    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+4];
    return;

}

procedure LibraAccount_delegated_withdrawal_capability_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_delegated_withdrawal_capability(arg0);
}

procedure {:inline 1} LibraAccount_withdrawal_capability_address (arg0: Reference) returns (ret0: Reference)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t1: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t2: Reference; // ReferenceType(AddressType())

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 3;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, LibraAccount_WithdrawalCapability_account_address);

    ret0 := t2;
    return;

}

procedure LibraAccount_withdrawal_capability_address_verify (arg0: Reference) returns (ret0: Reference)
{
    call ret0 := LibraAccount_withdrawal_capability_address(arg0);
}

procedure {:inline 1} LibraAccount_key_rotation_capability_address (arg0: Reference) returns (ret0: Reference)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var t1: Reference; // ReferenceType(LibraAccount_KeyRotationCapability_type_value())
    var t2: Reference; // ReferenceType(AddressType())

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 3;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, LibraAccount_KeyRotationCapability_account_address);

    ret0 := t2;
    return;

}

procedure LibraAccount_key_rotation_capability_address_verify (arg0: Reference) returns (ret0: Reference)
{
    call ret0 := LibraAccount_key_rotation_capability_address(arg0);
}

procedure {:inline 1} LibraAccount_exists (arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 3;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := Exists(contents#Memory(m)[old_size+1], LibraAccount_T_type_value());
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+2];
    return;

}

procedure LibraAccount_exists_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_exists(arg0);
}

procedure {:inline 1} LibraAccount_prologue () returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // BooleanType()
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Value; // ByteArrayType()
    var t5: Value; // ByteArrayType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // IntegerType()
    var t11: Value; // IntegerType()
    var t12: Value; // AddressType()
    var t13: Value; // AddressType()
    var t14: Value; // BooleanType()
    var t15: Value; // BooleanType()
    var t16: Value; // BooleanType()
    var t17: Value; // IntegerType()
    var t18: Value; // AddressType()
    var t19: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t20: Value; // ByteArrayType()
    var t21: Value; // ByteArrayType()
    var t22: Value; // ByteArrayType()
    var t23: Value; // ByteArrayType()
    var t24: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t25: Reference; // ReferenceType(ByteArrayType())
    var t26: Value; // ByteArrayType()
    var t27: Value; // BooleanType()
    var t28: Value; // BooleanType()
    var t29: Value; // IntegerType()
    var t30: Value; // IntegerType()
    var t31: Value; // IntegerType()
    var t32: Value; // IntegerType()
    var t33: Value; // IntegerType()
    var t34: Value; // IntegerType()
    var t35: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t36: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t37: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t38: Value; // IntegerType()
    var t39: Value; // IntegerType()
    var t40: Value; // IntegerType()
    var t41: Value; // BooleanType()
    var t42: Value; // BooleanType()
    var t43: Value; // IntegerType()
    var t44: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t45: Reference; // ReferenceType(IntegerType())
    var t46: Value; // IntegerType()
    var t47: Value; // IntegerType()
    var t48: Value; // IntegerType()
    var t49: Value; // IntegerType()
    var t50: Value; // BooleanType()
    var t51: Value; // BooleanType()
    var t52: Value; // IntegerType()
    var t53: Value; // IntegerType()
    var t54: Value; // IntegerType()
    var t55: Value; // BooleanType()
    var t56: Value; // BooleanType()
    var t57: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 58;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+12]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    call tmp := Exists(contents#Memory(m)[old_size+13], LibraAccount_T_type_value());
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+14]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 16];
    if (!b#Boolean(tmp)) { goto Label_10; }

    call tmp := LdConst(5);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    assert false;

Label_10:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call t19 := BorrowGlobal(contents#Memory(m)[old_size+18], LibraAccount_T_type_value());

    call t2 := CopyOrMoveRef(t19);

    call tmp := GetTxnPublicKey();
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+20]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    call t22 := Hash_sha3_256(contents#Memory(m)[old_size+21]);
    assume has_type(ByteArrayType(), t22);

    m := Memory(domain#Memory(m)[old_size+22 := true], contents#Memory(m)[old_size+22 := t22]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+22]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    call t24 := CopyOrMoveRef(t2);

    call t25 := BorrowField(t24, LibraAccount_T_authentication_key);

    call tmp := ReadRef(t25);
    assume has_type(ByteArrayType(), tmp);

    m := Memory(domain#Memory(m)[26+old_size := true], contents#Memory(m)[26+old_size := tmp]);

    tmp := Boolean(is_equal(ByteArrayType(), contents#Memory(m)[old_size+23], contents#Memory(m)[old_size+26]));
    m := Memory(domain#Memory(m)[27+old_size := true], contents#Memory(m)[27+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+27]);
    m := Memory(domain#Memory(m)[28+old_size := true], contents#Memory(m)[28+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 28];
    if (!b#Boolean(tmp)) { goto Label_27; }

    call tmp := LdConst(2);
    m := Memory(domain#Memory(m)[29+old_size := true], contents#Memory(m)[29+old_size := tmp]);

    assert false;

Label_27:
    call tmp := GetTxnGasUnitPrice();
    m := Memory(domain#Memory(m)[30+old_size := true], contents#Memory(m)[30+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+30]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := GetTxnMaxGasUnits();
    m := Memory(domain#Memory(m)[31+old_size := true], contents#Memory(m)[31+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+31]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[32+old_size := true], contents#Memory(m)[32+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[33+old_size := true], contents#Memory(m)[33+old_size := tmp]);

    call tmp := Mul(contents#Memory(m)[old_size+32], contents#Memory(m)[old_size+33]);
    m := Memory(domain#Memory(m)[34+old_size := true], contents#Memory(m)[34+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+34]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call t35 := CopyOrMoveRef(t2);

    call t36 := FreezeRef(t35);

    call t3 := CopyOrMoveRef(t36);

    call t37 := CopyOrMoveRef(t3);

    call t38 := LibraAccount_balance_for_account(t37);
    assume has_type(IntegerType(), t38);

    m := Memory(domain#Memory(m)[old_size+38 := true], contents#Memory(m)[old_size+38 := t38]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+38]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[39+old_size := true], contents#Memory(m)[39+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[40+old_size := true], contents#Memory(m)[40+old_size := tmp]);

    call tmp := Ge(contents#Memory(m)[old_size+39], contents#Memory(m)[old_size+40]);
    m := Memory(domain#Memory(m)[41+old_size := true], contents#Memory(m)[41+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+41]);
    m := Memory(domain#Memory(m)[42+old_size := true], contents#Memory(m)[42+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 42];
    if (!b#Boolean(tmp)) { goto Label_48; }

    call tmp := LdConst(6);
    m := Memory(domain#Memory(m)[43+old_size := true], contents#Memory(m)[43+old_size := tmp]);

    assert false;

Label_48:
    call t44 := CopyOrMoveRef(t2);

    call t45 := BorrowField(t44, LibraAccount_T_sequence_number);

    call tmp := ReadRef(t45);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[46+old_size := true], contents#Memory(m)[46+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+46]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := GetTxnSequenceNumber();
    m := Memory(domain#Memory(m)[47+old_size := true], contents#Memory(m)[47+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+47]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[48+old_size := true], contents#Memory(m)[48+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+10]);
    m := Memory(domain#Memory(m)[49+old_size := true], contents#Memory(m)[49+old_size := tmp]);

    call tmp := Ge(contents#Memory(m)[old_size+48], contents#Memory(m)[old_size+49]);
    m := Memory(domain#Memory(m)[50+old_size := true], contents#Memory(m)[50+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+50]);
    m := Memory(domain#Memory(m)[51+old_size := true], contents#Memory(m)[51+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 51];
    if (!b#Boolean(tmp)) { goto Label_61; }

    call tmp := LdConst(3);
    m := Memory(domain#Memory(m)[52+old_size := true], contents#Memory(m)[52+old_size := tmp]);

    assert false;

Label_61:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[53+old_size := true], contents#Memory(m)[53+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+10]);
    m := Memory(domain#Memory(m)[54+old_size := true], contents#Memory(m)[54+old_size := tmp]);

    tmp := Boolean(is_equal(IntegerType(), contents#Memory(m)[old_size+53], contents#Memory(m)[old_size+54]));
    m := Memory(domain#Memory(m)[55+old_size := true], contents#Memory(m)[55+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+55]);
    m := Memory(domain#Memory(m)[56+old_size := true], contents#Memory(m)[56+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 56];
    if (!b#Boolean(tmp)) { goto Label_68; }

    call tmp := LdConst(4);
    m := Memory(domain#Memory(m)[57+old_size := true], contents#Memory(m)[57+old_size := tmp]);

    assert false;

Label_68:
    return;

}

procedure LibraAccount_prologue_verify () returns ()
{
    call LibraAccount_prologue();
}

procedure {:inline 1} LibraAccount_epilogue () returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t2: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t3: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // LibraCoin_T_type_value()
    var t10: Value; // IntegerType()
    var t11: Value; // AddressType()
    var t12: Value; // AddressType()
    var t13: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Value; // IntegerType()
    var t17: Value; // IntegerType()
    var t18: Value; // IntegerType()
    var t19: Value; // IntegerType()
    var t20: Value; // IntegerType()
    var t21: Value; // IntegerType()
    var t22: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t23: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t24: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t25: Value; // IntegerType()
    var t26: Value; // IntegerType()
    var t27: Value; // IntegerType()
    var t28: Value; // BooleanType()
    var t29: Value; // BooleanType()
    var t30: Value; // IntegerType()
    var t31: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t32: Value; // IntegerType()
    var t33: Value; // LibraCoin_T_type_value()
    var t34: Value; // IntegerType()
    var t35: Value; // IntegerType()
    var t36: Value; // IntegerType()
    var t37: Value; // IntegerType()
    var t38: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t39: Reference; // ReferenceType(IntegerType())
    var t40: Value; // AddressType()
    var t41: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t42: Reference; // ReferenceType(LibraAccount_T_type_value())
    var t43: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t44: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 45;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call t13 := BorrowGlobal(contents#Memory(m)[old_size+12], LibraAccount_T_type_value());

    call t1 := CopyOrMoveRef(t13);

    call tmp := GetTxnGasUnitPrice();
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+14]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := GetTxnMaxGasUnits();
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := GetGasRemaining();
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+16]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    call tmp := Sub(contents#Memory(m)[old_size+18], contents#Memory(m)[old_size+19]);
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call tmp := Mul(contents#Memory(m)[old_size+17], contents#Memory(m)[old_size+20]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+21]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t22 := CopyOrMoveRef(t1);

    call t23 := FreezeRef(t22);

    call t3 := CopyOrMoveRef(t23);

    call t24 := CopyOrMoveRef(t3);

    call t25 := LibraAccount_balance_for_account(t24);
    assume has_type(IntegerType(), t25);

    m := Memory(domain#Memory(m)[old_size+25 := true], contents#Memory(m)[old_size+25 := t25]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+25]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[26+old_size := true], contents#Memory(m)[26+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[27+old_size := true], contents#Memory(m)[27+old_size := tmp]);

    call tmp := Ge(contents#Memory(m)[old_size+26], contents#Memory(m)[old_size+27]);
    m := Memory(domain#Memory(m)[28+old_size := true], contents#Memory(m)[28+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+28]);
    m := Memory(domain#Memory(m)[29+old_size := true], contents#Memory(m)[29+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 29];
    if (!b#Boolean(tmp)) { goto Label_30; }

    call tmp := LdConst(6);
    m := Memory(domain#Memory(m)[30+old_size := true], contents#Memory(m)[30+old_size := tmp]);

    assert false;

Label_30:
    call t31 := CopyOrMoveRef(t1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[32+old_size := true], contents#Memory(m)[32+old_size := tmp]);

    call t33 := LibraAccount_withdraw_from_account(t31, contents#Memory(m)[old_size+32]);
    assume has_type(LibraCoin_T_type_value(), t33);

    m := Memory(domain#Memory(m)[old_size+33 := true], contents#Memory(m)[old_size+33 := t33]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+33]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := GetTxnSequenceNumber();
    m := Memory(domain#Memory(m)[34+old_size := true], contents#Memory(m)[34+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+34]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+10]);
    m := Memory(domain#Memory(m)[35+old_size := true], contents#Memory(m)[35+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[36+old_size := true], contents#Memory(m)[36+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+35], contents#Memory(m)[old_size+36]);
    m := Memory(domain#Memory(m)[37+old_size := true], contents#Memory(m)[37+old_size := tmp]);

    call t38 := CopyOrMoveRef(t1);

    call t39 := BorrowField(t38, LibraAccount_T_sequence_number);

    call WriteRef(t39, contents#Memory(m)[old_size+37]);

    call tmp := LdAddr(4078);
    m := Memory(domain#Memory(m)[40+old_size := true], contents#Memory(m)[40+old_size := tmp]);

    call t41 := BorrowGlobal(contents#Memory(m)[old_size+40], LibraAccount_T_type_value());

    call t2 := CopyOrMoveRef(t41);

    call t42 := CopyOrMoveRef(t2);

    call t43 := BorrowField(t42, LibraAccount_T_balance);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[44+old_size := true], contents#Memory(m)[44+old_size := tmp]);

    call LibraCoin_deposit(t43, contents#Memory(m)[old_size+44]);

    return;

}

procedure LibraAccount_epilogue_verify () returns ()
{
    call LibraAccount_epilogue();
}

procedure {:inline 1} LibraAccount_fresh_guid (arg0: Reference, arg1: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t1: Value; // AddressType()
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
    assume !abort_flag;

    // assume arguments are of correct types
        assume has_type(AddressType(), arg1);

    old_size := m_size;
    m_size := m_size + 22;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t6 := CopyOrMoveRef(t0);

    call t7 := BorrowField(t6, LibraAccount_EventHandleGenerator_counter);

    call t2 := CopyOrMoveRef(t7);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call t9 := AddressUtil_address_to_bytes(contents#Memory(m)[old_size+8]);
    assume has_type(ByteArrayType(), t9);

    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t10 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t10);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call t12 := U64Util_u64_to_bytes(contents#Memory(m)[old_size+11]);
    assume has_type(ByteArrayType(), t12);

    m := Memory(domain#Memory(m)[old_size+12 := true], contents#Memory(m)[old_size+12 := t12]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+12]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t13 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t13);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+14], contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call t17 := CopyOrMoveRef(t2);

    call WriteRef(t17, contents#Memory(m)[old_size+16]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    call t20 := BytearrayUtil_bytearray_concat(contents#Memory(m)[old_size+18], contents#Memory(m)[old_size+19]);
    assume has_type(ByteArrayType(), t20);

    m := Memory(domain#Memory(m)[old_size+20 := true], contents#Memory(m)[old_size+20 := t20]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+20]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+21];
    return;

}

procedure LibraAccount_fresh_guid_verify (arg0: Reference, arg1: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_fresh_guid(arg0, arg1);
}

procedure {:inline 1} LibraAccount_new_event_handle_impl (tv0: TypeValue, arg0: Reference, arg1: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t1: Value; // AddressType()
    var t2: Value; // IntegerType()
    var t3: Reference; // ReferenceType(LibraAccount_EventHandleGenerator_type_value())
    var t4: Value; // AddressType()
    var t5: Value; // ByteArrayType()
    var t6: Value; // LibraAccount_EventHandle_type_value(tv0)

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
        assume has_type(AddressType(), arg1);

    old_size := m_size;
    m_size := m_size + 7;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := CopyOrMoveRef(t0);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t5 := LibraAccount_fresh_guid(t3, contents#Memory(m)[old_size+4]);
    assume has_type(ByteArrayType(), t5);

    m := Memory(domain#Memory(m)[old_size+5 := true], contents#Memory(m)[old_size+5 := t5]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+2]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+5]);

    call tmp := Pack_LibraAccount_EventHandle(tv0, contents#Memory(m)[old_size+2], contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+6];
    return;

}

procedure LibraAccount_new_event_handle_impl_verify (tv0: TypeValue, arg0: Reference, arg1: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_new_event_handle_impl(tv0: TypeValue, arg0, arg1);
}

procedure {:inline 1} LibraAccount_new_event_handle (tv0: TypeValue) returns (ret0: Value)
requires ExistsTxnSenderAccount();
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
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 8;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], LibraAccount_T_type_value());

    call t0 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, LibraAccount_T_event_generator);

    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call t7 := LibraAccount_new_event_handle_impl(tv0, t5, contents#Memory(m)[old_size+6]);
    assume has_type(LibraAccount_EventHandle_type_value(tv0), t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    ret0 := contents#Memory(m)[old_size+7];
    return;

}

procedure LibraAccount_new_event_handle_verify (tv0: TypeValue) returns (ret0: Value)
{
    call ret0 := LibraAccount_new_event_handle(tv0: TypeValue);
}

procedure {:inline 1} LibraAccount_emit_event (tv0: TypeValue, arg0: Reference, arg1: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(tv0))
    var t1: Value; // tv0
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
    assume !abort_flag;

    // assume arguments are of correct types
        assume has_type(tv0, arg1);

    old_size := m_size;
    m_size := m_size + 18;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, LibraAccount_EventHandle_guid);

    call tmp := ReadRef(t5);
    assume has_type(ByteArrayType(), tmp);

    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t7 := CopyOrMoveRef(t0);

    call t8 := BorrowField(t7, LibraAccount_EventHandle_counter);

    call t2 := CopyOrMoveRef(t8);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call t10 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t10);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call LibraAccount_write_to_event_store(tv0, contents#Memory(m)[old_size+9], contents#Memory(m)[old_size+11], contents#Memory(m)[old_size+12]);

    call t13 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t13);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+14], contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call t17 := CopyOrMoveRef(t2);

    call WriteRef(t17, contents#Memory(m)[old_size+16]);

    return;

}

procedure LibraAccount_emit_event_verify (tv0: TypeValue, arg0: Reference, arg1: Value) returns ()
{
    call LibraAccount_emit_event(tv0: TypeValue, arg0, arg1);
}

procedure {:inline 1} LibraAccount_write_to_event_store (tv0: TypeValue, arg0: Value, arg1: Value, arg2: Value) returns ();procedure {:inline 1} LibraAccount_destroy_handle (tv0: TypeValue, arg0: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // LibraAccount_EventHandle_type_value(tv0)
    var t1: Value; // ByteArrayType()
    var t2: Value; // IntegerType()
    var t3: Value; // LibraAccount_EventHandle_type_value(tv0)
    var t4: Value; // IntegerType()
    var t5: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(LibraAccount_EventHandle_type_value(tv0), arg0);

    old_size := m_size;
    m_size := m_size + 6;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t4, t5 := Unpack_LibraAccount_EventHandle(contents#Memory(m)[old_size+3]);
    assume has_type(IntegerType(), t4);

    assume has_type(ByteArrayType(), t5);

    m := Memory(domain#Memory(m)[old_size+4 := true], contents#Memory(m)[old_size+4 := t4]);
    m := Memory(domain#Memory(m)[old_size+5 := true], contents#Memory(m)[old_size+5 := t5]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    return;

}

procedure LibraAccount_destroy_handle_verify (tv0: TypeValue, arg0: Value) returns ()
{
    call LibraAccount_destroy_handle(tv0: TypeValue, arg0);
}



// ** functions of module TestLib
