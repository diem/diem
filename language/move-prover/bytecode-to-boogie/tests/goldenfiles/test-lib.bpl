

// everything below is auto generated

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

const unique ValidatorConfig_Config: TypeName;
const ValidatorConfig_Config_consensus_pubkey: FieldName;
axiom ValidatorConfig_Config_consensus_pubkey == 0;
const ValidatorConfig_Config_network_identity_pubkey: FieldName;
axiom ValidatorConfig_Config_network_identity_pubkey == 1;
const ValidatorConfig_Config_network_signing_pubkey: FieldName;
axiom ValidatorConfig_Config_network_signing_pubkey == 2;
function ValidatorConfig_Config_type_value(): TypeValue {
    StructType(ValidatorConfig_Config, TypeValueArray(DefaultTypeMap[0 := ByteArrayType()][1 := ByteArrayType()][2 := ByteArrayType()], 3))
}

procedure {:inline 1} Pack_ValidatorConfig_Config(v0: Value, v1: Value, v2: Value) returns (v: Value)
{
    assume has_type(ByteArrayType(), v0);
    assume has_type(ByteArrayType(), v1);
    assume has_type(ByteArrayType(), v2);
    v := Struct(ValueArray(DefaultIntMap[ValidatorConfig_Config_consensus_pubkey := v0][ValidatorConfig_Config_network_identity_pubkey := v1][ValidatorConfig_Config_network_signing_pubkey := v2], 3));
    assume has_type(ValidatorConfig_Config_type_value(), v);
}

procedure {:inline 1} Unpack_ValidatorConfig_Config(v: Value) returns (v0: Value, v1: Value, v2: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[ValidatorConfig_Config_consensus_pubkey];
    v1 := smap(v)[ValidatorConfig_Config_network_identity_pubkey];
    v2 := smap(v)[ValidatorConfig_Config_network_signing_pubkey];
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

const unique LibraSystem_ValidatorInfo: TypeName;
const LibraSystem_ValidatorInfo_addr: FieldName;
axiom LibraSystem_ValidatorInfo_addr == 0;
const LibraSystem_ValidatorInfo_consensus_pubkey: FieldName;
axiom LibraSystem_ValidatorInfo_consensus_pubkey == 1;
const LibraSystem_ValidatorInfo_consensus_voting_power: FieldName;
axiom LibraSystem_ValidatorInfo_consensus_voting_power == 2;
const LibraSystem_ValidatorInfo_network_signing_pubkey: FieldName;
axiom LibraSystem_ValidatorInfo_network_signing_pubkey == 3;
const LibraSystem_ValidatorInfo_network_identity_pubkey: FieldName;
axiom LibraSystem_ValidatorInfo_network_identity_pubkey == 4;
function LibraSystem_ValidatorInfo_type_value(): TypeValue {
    StructType(LibraSystem_ValidatorInfo, TypeValueArray(DefaultTypeMap[0 := AddressType()][1 := ByteArrayType()][2 := IntegerType()][3 := ByteArrayType()][4 := ByteArrayType()], 5))
}

procedure {:inline 1} Pack_LibraSystem_ValidatorInfo(v0: Value, v1: Value, v2: Value, v3: Value, v4: Value) returns (v: Value)
{
    assume has_type(AddressType(), v0);
    assume has_type(ByteArrayType(), v1);
    assume has_type(IntegerType(), v2);
    assume has_type(ByteArrayType(), v3);
    assume has_type(ByteArrayType(), v4);
    v := Struct(ValueArray(DefaultIntMap[LibraSystem_ValidatorInfo_addr := v0][LibraSystem_ValidatorInfo_consensus_pubkey := v1][LibraSystem_ValidatorInfo_consensus_voting_power := v2][LibraSystem_ValidatorInfo_network_signing_pubkey := v3][LibraSystem_ValidatorInfo_network_identity_pubkey := v4], 5));
    assume has_type(LibraSystem_ValidatorInfo_type_value(), v);
}

procedure {:inline 1} Unpack_LibraSystem_ValidatorInfo(v: Value) returns (v0: Value, v1: Value, v2: Value, v3: Value, v4: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[LibraSystem_ValidatorInfo_addr];
    v1 := smap(v)[LibraSystem_ValidatorInfo_consensus_pubkey];
    v2 := smap(v)[LibraSystem_ValidatorInfo_consensus_voting_power];
    v3 := smap(v)[LibraSystem_ValidatorInfo_network_signing_pubkey];
    v4 := smap(v)[LibraSystem_ValidatorInfo_network_identity_pubkey];
}

const unique LibraSystem_ValidatorSetChangeEvent: TypeName;
const LibraSystem_ValidatorSetChangeEvent_new_validator_set: FieldName;
axiom LibraSystem_ValidatorSetChangeEvent_new_validator_set == 0;
function LibraSystem_ValidatorSetChangeEvent_type_value(): TypeValue {
    StructType(LibraSystem_ValidatorSetChangeEvent, TypeValueArray(DefaultTypeMap[0 := Vector_T_type_value(LibraSystem_ValidatorInfo_type_value())], 1))
}

procedure {:inline 1} Pack_LibraSystem_ValidatorSetChangeEvent(v0: Value) returns (v: Value)
{
    assume has_type(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()), v0);
    v := Struct(ValueArray(DefaultIntMap[LibraSystem_ValidatorSetChangeEvent_new_validator_set := v0], 1));
    assume has_type(LibraSystem_ValidatorSetChangeEvent_type_value(), v);
}

procedure {:inline 1} Unpack_LibraSystem_ValidatorSetChangeEvent(v: Value) returns (v0: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[LibraSystem_ValidatorSetChangeEvent_new_validator_set];
}

const unique LibraSystem_ValidatorSet: TypeName;
const LibraSystem_ValidatorSet_validators: FieldName;
axiom LibraSystem_ValidatorSet_validators == 0;
const LibraSystem_ValidatorSet_additions: FieldName;
axiom LibraSystem_ValidatorSet_additions == 1;
const LibraSystem_ValidatorSet_change_events: FieldName;
axiom LibraSystem_ValidatorSet_change_events == 2;
function LibraSystem_ValidatorSet_type_value(): TypeValue {
    StructType(LibraSystem_ValidatorSet, TypeValueArray(DefaultTypeMap[0 := Vector_T_type_value(LibraSystem_ValidatorInfo_type_value())][1 := Vector_T_type_value(AddressType())][2 := LibraAccount_EventHandle_type_value(LibraSystem_ValidatorSetChangeEvent_type_value())], 3))
}

procedure {:inline 1} Pack_LibraSystem_ValidatorSet(v0: Value, v1: Value, v2: Value) returns (v: Value)
{
    assume has_type(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()), v0);
    assume has_type(Vector_T_type_value(AddressType()), v1);
    assume has_type(LibraAccount_EventHandle_type_value(LibraSystem_ValidatorSetChangeEvent_type_value()), v2);
    v := Struct(ValueArray(DefaultIntMap[LibraSystem_ValidatorSet_validators := v0][LibraSystem_ValidatorSet_additions := v1][LibraSystem_ValidatorSet_change_events := v2], 3));
    assume has_type(LibraSystem_ValidatorSet_type_value(), v);
}

procedure {:inline 1} Unpack_LibraSystem_ValidatorSet(v: Value) returns (v0: Value, v1: Value, v2: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[LibraSystem_ValidatorSet_validators];
    v1 := smap(v)[LibraSystem_ValidatorSet_additions];
    v2 := smap(v)[LibraSystem_ValidatorSet_change_events];
}

const unique LibraSystem_BlockMetadata: TypeName;
const LibraSystem_BlockMetadata_height: FieldName;
axiom LibraSystem_BlockMetadata_height == 0;
const LibraSystem_BlockMetadata_timestamp: FieldName;
axiom LibraSystem_BlockMetadata_timestamp == 1;
const LibraSystem_BlockMetadata_id: FieldName;
axiom LibraSystem_BlockMetadata_id == 2;
const LibraSystem_BlockMetadata_proposer: FieldName;
axiom LibraSystem_BlockMetadata_proposer == 3;
function LibraSystem_BlockMetadata_type_value(): TypeValue {
    StructType(LibraSystem_BlockMetadata, TypeValueArray(DefaultTypeMap[0 := IntegerType()][1 := IntegerType()][2 := ByteArrayType()][3 := AddressType()], 4))
}

procedure {:inline 1} Pack_LibraSystem_BlockMetadata(v0: Value, v1: Value, v2: Value, v3: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    assume has_type(IntegerType(), v1);
    assume has_type(ByteArrayType(), v2);
    assume has_type(AddressType(), v3);
    v := Struct(ValueArray(DefaultIntMap[LibraSystem_BlockMetadata_height := v0][LibraSystem_BlockMetadata_timestamp := v1][LibraSystem_BlockMetadata_id := v2][LibraSystem_BlockMetadata_proposer := v3], 4));
    assume has_type(LibraSystem_BlockMetadata_type_value(), v);
}

procedure {:inline 1} Unpack_LibraSystem_BlockMetadata(v: Value) returns (v0: Value, v1: Value, v2: Value, v3: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[LibraSystem_BlockMetadata_height];
    v1 := smap(v)[LibraSystem_BlockMetadata_timestamp];
    v2 := smap(v)[LibraSystem_BlockMetadata_id];
    v3 := smap(v)[LibraSystem_BlockMetadata_proposer];
}

const unique TransactionFeeDistribution_T: TypeName;
const TransactionFeeDistribution_T_last_epoch_paid: FieldName;
axiom TransactionFeeDistribution_T_last_epoch_paid == 0;
const TransactionFeeDistribution_T_fee_withdrawal_capability: FieldName;
axiom TransactionFeeDistribution_T_fee_withdrawal_capability == 1;
function TransactionFeeDistribution_T_type_value(): TypeValue {
    StructType(TransactionFeeDistribution_T, TypeValueArray(DefaultTypeMap[0 := IntegerType()][1 := LibraAccount_WithdrawalCapability_type_value()], 2))
}

procedure {:inline 1} Pack_TransactionFeeDistribution_T(v0: Value, v1: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    assume has_type(LibraAccount_WithdrawalCapability_type_value(), v1);
    v := Struct(ValueArray(DefaultIntMap[TransactionFeeDistribution_T_last_epoch_paid := v0][TransactionFeeDistribution_T_fee_withdrawal_capability := v1], 2));
    assume has_type(TransactionFeeDistribution_T_type_value(), v);
}

procedure {:inline 1} Unpack_TransactionFeeDistribution_T(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[TransactionFeeDistribution_T_last_epoch_paid];
    v1 := smap(v)[TransactionFeeDistribution_T_fee_withdrawal_capability];
}

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

procedure {:inline 1} AddressUtil_address_to_bytes (arg0: Value) returns (ret0: Value);
procedure {:inline 1} BytearrayUtil_bytearray_concat (arg0: Value, arg1: Value) returns (ret0: Value);
procedure {:inline 1} LibraCoin_mint_with_default_capability (arg0: Value) returns (ret0: Value)
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

procedure {:inline 1} Hash_sha2_256 (arg0: Value) returns (ret0: Value);
procedure {:inline 1} Hash_sha3_256 (arg0: Value) returns (ret0: Value);
procedure {:inline 1} Signature_ed25519_verify (arg0: Value, arg1: Value, arg2: Value) returns (ret0: Value);
procedure {:inline 1} Signature_ed25519_threshold_verify (arg0: Value, arg1: Value, arg2: Value, arg3: Value) returns (ret0: Value);
procedure {:inline 1} U64Util_u64_to_bytes (arg0: Value) returns (ret0: Value);
procedure {:inline 1} ValidatorConfig_has (arg0: Value) returns (ret0: Value)
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

procedure {:inline 1} ValidatorConfig_network_identity_pubkey (arg0: Reference) returns (ret0: Value)
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

    call t2 := BorrowField(t1, ValidatorConfig_Config_network_identity_pubkey);

    call tmp := ReadRef(t2);
    assume has_type(ByteArrayType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure ValidatorConfig_network_identity_pubkey_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := ValidatorConfig_network_identity_pubkey(arg0);
}

procedure {:inline 1} ValidatorConfig_network_signing_pubkey (arg0: Reference) returns (ret0: Value)
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

    call t2 := BorrowField(t1, ValidatorConfig_Config_network_signing_pubkey);

    call tmp := ReadRef(t2);
    assume has_type(ByteArrayType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure ValidatorConfig_network_signing_pubkey_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := ValidatorConfig_network_signing_pubkey(arg0);
}

procedure {:inline 1} ValidatorConfig_register_candidate_validator (arg0: Value, arg1: Value, arg2: Value) returns ()
{
    // declare local variables
    var t0: Value; // ByteArrayType()
    var t1: Value; // ByteArrayType()
    var t2: Value; // ByteArrayType()
    var t3: Value; // ByteArrayType()
    var t4: Value; // ByteArrayType()
    var t5: Value; // ByteArrayType()
    var t6: Value; // ValidatorConfig_Config_type_value()
    var t7: Value; // ValidatorConfig_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(ByteArrayType(), arg0);
    assume has_type(ByteArrayType(), arg1);
    assume has_type(ByteArrayType(), arg2);

    old_size := m_size;
    m_size := m_size + 8;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size :=  arg2]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+3]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+4]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+5]);

    call tmp := Pack_ValidatorConfig_Config(contents#Memory(m)[old_size+3], contents#Memory(m)[old_size+4], contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    assume has_type(ValidatorConfig_Config_type_value(), contents#Memory(m)[old_size+6]);

    call tmp := Pack_ValidatorConfig_T(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call MoveToSender(ValidatorConfig_T_type_value(), contents#Memory(m)[old_size+7]);

    return;

}

procedure ValidatorConfig_register_candidate_validator_verify (arg0: Value, arg1: Value, arg2: Value) returns ()
{
    call ValidatorConfig_register_candidate_validator(arg0, arg1, arg2);
}

procedure {:inline 1} ValidatorConfig_rotate_consensus_pubkey (arg0: Value) returns ()
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

procedure {:inline 1} GasSchedule_initialize () returns ()
{
    // declare local variables
    var t0: Value; // Vector_T_type_value(GasSchedule_Cost_type_value())
    var t1: Value; // Vector_T_type_value(GasSchedule_Cost_type_value())
    var t2: Value; // AddressType()
    var t3: Value; // AddressType()
    var t4: Value; // BooleanType()
    var t5: Value; // BooleanType()
    var t6: Value; // IntegerType()
    var t7: Value; // Vector_T_type_value(GasSchedule_Cost_type_value())
    var t8: Value; // Vector_T_type_value(GasSchedule_Cost_type_value())
    var t9: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t10: Value; // IntegerType()
    var t11: Value; // IntegerType()
    var t12: Value; // GasSchedule_Cost_type_value()
    var t13: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Value; // GasSchedule_Cost_type_value()
    var t17: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t18: Value; // IntegerType()
    var t19: Value; // IntegerType()
    var t20: Value; // GasSchedule_Cost_type_value()
    var t21: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t22: Value; // IntegerType()
    var t23: Value; // IntegerType()
    var t24: Value; // GasSchedule_Cost_type_value()
    var t25: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t26: Value; // IntegerType()
    var t27: Value; // IntegerType()
    var t28: Value; // GasSchedule_Cost_type_value()
    var t29: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t30: Value; // IntegerType()
    var t31: Value; // IntegerType()
    var t32: Value; // GasSchedule_Cost_type_value()
    var t33: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t34: Value; // IntegerType()
    var t35: Value; // IntegerType()
    var t36: Value; // GasSchedule_Cost_type_value()
    var t37: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t38: Value; // IntegerType()
    var t39: Value; // IntegerType()
    var t40: Value; // GasSchedule_Cost_type_value()
    var t41: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t42: Value; // IntegerType()
    var t43: Value; // IntegerType()
    var t44: Value; // GasSchedule_Cost_type_value()
    var t45: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t46: Value; // IntegerType()
    var t47: Value; // IntegerType()
    var t48: Value; // GasSchedule_Cost_type_value()
    var t49: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t50: Value; // IntegerType()
    var t51: Value; // IntegerType()
    var t52: Value; // GasSchedule_Cost_type_value()
    var t53: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t54: Value; // IntegerType()
    var t55: Value; // IntegerType()
    var t56: Value; // GasSchedule_Cost_type_value()
    var t57: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t58: Value; // IntegerType()
    var t59: Value; // IntegerType()
    var t60: Value; // GasSchedule_Cost_type_value()
    var t61: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t62: Value; // IntegerType()
    var t63: Value; // IntegerType()
    var t64: Value; // GasSchedule_Cost_type_value()
    var t65: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t66: Value; // IntegerType()
    var t67: Value; // IntegerType()
    var t68: Value; // GasSchedule_Cost_type_value()
    var t69: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t70: Value; // IntegerType()
    var t71: Value; // IntegerType()
    var t72: Value; // GasSchedule_Cost_type_value()
    var t73: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t74: Value; // IntegerType()
    var t75: Value; // IntegerType()
    var t76: Value; // GasSchedule_Cost_type_value()
    var t77: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t78: Value; // IntegerType()
    var t79: Value; // IntegerType()
    var t80: Value; // GasSchedule_Cost_type_value()
    var t81: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t82: Value; // IntegerType()
    var t83: Value; // IntegerType()
    var t84: Value; // GasSchedule_Cost_type_value()
    var t85: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t86: Value; // IntegerType()
    var t87: Value; // IntegerType()
    var t88: Value; // GasSchedule_Cost_type_value()
    var t89: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t90: Value; // IntegerType()
    var t91: Value; // IntegerType()
    var t92: Value; // GasSchedule_Cost_type_value()
    var t93: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t94: Value; // IntegerType()
    var t95: Value; // IntegerType()
    var t96: Value; // GasSchedule_Cost_type_value()
    var t97: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t98: Value; // IntegerType()
    var t99: Value; // IntegerType()
    var t100: Value; // GasSchedule_Cost_type_value()
    var t101: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t102: Value; // IntegerType()
    var t103: Value; // IntegerType()
    var t104: Value; // GasSchedule_Cost_type_value()
    var t105: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t106: Value; // IntegerType()
    var t107: Value; // IntegerType()
    var t108: Value; // GasSchedule_Cost_type_value()
    var t109: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t110: Value; // IntegerType()
    var t111: Value; // IntegerType()
    var t112: Value; // GasSchedule_Cost_type_value()
    var t113: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t114: Value; // IntegerType()
    var t115: Value; // IntegerType()
    var t116: Value; // GasSchedule_Cost_type_value()
    var t117: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t118: Value; // IntegerType()
    var t119: Value; // IntegerType()
    var t120: Value; // GasSchedule_Cost_type_value()
    var t121: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t122: Value; // IntegerType()
    var t123: Value; // IntegerType()
    var t124: Value; // GasSchedule_Cost_type_value()
    var t125: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t126: Value; // IntegerType()
    var t127: Value; // IntegerType()
    var t128: Value; // GasSchedule_Cost_type_value()
    var t129: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t130: Value; // IntegerType()
    var t131: Value; // IntegerType()
    var t132: Value; // GasSchedule_Cost_type_value()
    var t133: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t134: Value; // IntegerType()
    var t135: Value; // IntegerType()
    var t136: Value; // GasSchedule_Cost_type_value()
    var t137: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t138: Value; // IntegerType()
    var t139: Value; // IntegerType()
    var t140: Value; // GasSchedule_Cost_type_value()
    var t141: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t142: Value; // IntegerType()
    var t143: Value; // IntegerType()
    var t144: Value; // GasSchedule_Cost_type_value()
    var t145: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t146: Value; // IntegerType()
    var t147: Value; // IntegerType()
    var t148: Value; // GasSchedule_Cost_type_value()
    var t149: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t150: Value; // IntegerType()
    var t151: Value; // IntegerType()
    var t152: Value; // GasSchedule_Cost_type_value()
    var t153: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t154: Value; // IntegerType()
    var t155: Value; // IntegerType()
    var t156: Value; // GasSchedule_Cost_type_value()
    var t157: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t158: Value; // IntegerType()
    var t159: Value; // IntegerType()
    var t160: Value; // GasSchedule_Cost_type_value()
    var t161: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t162: Value; // IntegerType()
    var t163: Value; // IntegerType()
    var t164: Value; // GasSchedule_Cost_type_value()
    var t165: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t166: Value; // IntegerType()
    var t167: Value; // IntegerType()
    var t168: Value; // GasSchedule_Cost_type_value()
    var t169: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t170: Value; // IntegerType()
    var t171: Value; // IntegerType()
    var t172: Value; // GasSchedule_Cost_type_value()
    var t173: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t174: Value; // IntegerType()
    var t175: Value; // IntegerType()
    var t176: Value; // GasSchedule_Cost_type_value()
    var t177: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t178: Value; // IntegerType()
    var t179: Value; // IntegerType()
    var t180: Value; // GasSchedule_Cost_type_value()
    var t181: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t182: Value; // IntegerType()
    var t183: Value; // IntegerType()
    var t184: Value; // GasSchedule_Cost_type_value()
    var t185: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t186: Value; // IntegerType()
    var t187: Value; // IntegerType()
    var t188: Value; // GasSchedule_Cost_type_value()
    var t189: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t190: Value; // IntegerType()
    var t191: Value; // IntegerType()
    var t192: Value; // GasSchedule_Cost_type_value()
    var t193: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t194: Value; // IntegerType()
    var t195: Value; // IntegerType()
    var t196: Value; // GasSchedule_Cost_type_value()
    var t197: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t198: Value; // IntegerType()
    var t199: Value; // IntegerType()
    var t200: Value; // GasSchedule_Cost_type_value()
    var t201: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t202: Value; // IntegerType()
    var t203: Value; // IntegerType()
    var t204: Value; // GasSchedule_Cost_type_value()
    var t205: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t206: Value; // IntegerType()
    var t207: Value; // IntegerType()
    var t208: Value; // GasSchedule_Cost_type_value()
    var t209: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t210: Value; // IntegerType()
    var t211: Value; // IntegerType()
    var t212: Value; // GasSchedule_Cost_type_value()
    var t213: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t214: Value; // IntegerType()
    var t215: Value; // IntegerType()
    var t216: Value; // GasSchedule_Cost_type_value()
    var t217: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t218: Value; // IntegerType()
    var t219: Value; // IntegerType()
    var t220: Value; // GasSchedule_Cost_type_value()
    var t221: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t222: Value; // IntegerType()
    var t223: Value; // IntegerType()
    var t224: Value; // GasSchedule_Cost_type_value()
    var t225: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t226: Value; // IntegerType()
    var t227: Value; // IntegerType()
    var t228: Value; // GasSchedule_Cost_type_value()
    var t229: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t230: Value; // IntegerType()
    var t231: Value; // IntegerType()
    var t232: Value; // GasSchedule_Cost_type_value()
    var t233: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t234: Value; // IntegerType()
    var t235: Value; // IntegerType()
    var t236: Value; // GasSchedule_Cost_type_value()
    var t237: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t238: Value; // IntegerType()
    var t239: Value; // IntegerType()
    var t240: Value; // GasSchedule_Cost_type_value()
    var t241: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t242: Value; // IntegerType()
    var t243: Value; // IntegerType()
    var t244: Value; // GasSchedule_Cost_type_value()
    var t245: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t246: Value; // IntegerType()
    var t247: Value; // IntegerType()
    var t248: Value; // GasSchedule_Cost_type_value()
    var t249: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t250: Value; // IntegerType()
    var t251: Value; // IntegerType()
    var t252: Value; // GasSchedule_Cost_type_value()
    var t253: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t254: Value; // IntegerType()
    var t255: Value; // IntegerType()
    var t256: Value; // GasSchedule_Cost_type_value()
    var t257: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t258: Value; // IntegerType()
    var t259: Value; // IntegerType()
    var t260: Value; // GasSchedule_Cost_type_value()
    var t261: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t262: Value; // IntegerType()
    var t263: Value; // IntegerType()
    var t264: Value; // GasSchedule_Cost_type_value()
    var t265: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t266: Value; // IntegerType()
    var t267: Value; // IntegerType()
    var t268: Value; // GasSchedule_Cost_type_value()
    var t269: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t270: Value; // IntegerType()
    var t271: Value; // IntegerType()
    var t272: Value; // GasSchedule_Cost_type_value()
    var t273: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t274: Value; // IntegerType()
    var t275: Value; // IntegerType()
    var t276: Value; // GasSchedule_Cost_type_value()
    var t277: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t278: Value; // IntegerType()
    var t279: Value; // IntegerType()
    var t280: Value; // GasSchedule_Cost_type_value()
    var t281: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t282: Value; // IntegerType()
    var t283: Value; // IntegerType()
    var t284: Value; // GasSchedule_Cost_type_value()
    var t285: Reference; // ReferenceType(Vector_T_type_value(GasSchedule_Cost_type_value()))
    var t286: Value; // IntegerType()
    var t287: Value; // IntegerType()
    var t288: Value; // GasSchedule_Cost_type_value()
    var t289: Value; // Vector_T_type_value(GasSchedule_Cost_type_value())
    var t290: Value; // Vector_T_type_value(GasSchedule_Cost_type_value())
    var t291: Value; // GasSchedule_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 292;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    tmp := Boolean(is_equal(AddressType(), contents#Memory(m)[old_size+2], contents#Memory(m)[old_size+3]));
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 5];
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    assert false;

Label_7:
    call t7 := Vector_empty(GasSchedule_Cost_type_value());
    assume has_type(Vector_T_type_value(GasSchedule_Cost_type_value()), t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call t8 := Vector_empty(GasSchedule_Cost_type_value());
    assume has_type(Vector_T_type_value(GasSchedule_Cost_type_value()), t8);

    m := Memory(domain#Memory(m)[old_size+8 := true], contents#Memory(m)[old_size+8 := t8]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t9 := BorrowLoc(old_size+0);

    call tmp := LdConst(27);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+10]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+11]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+10], contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t9, contents#Memory(m)[old_size+12]);

    call t13 := BorrowLoc(old_size+0);

    call tmp := LdConst(28);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+14]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+15]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+14], contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t13, contents#Memory(m)[old_size+16]);

    call t17 := BorrowLoc(old_size+0);

    call tmp := LdConst(31);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+18]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+19]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+18], contents#Memory(m)[old_size+19]);
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t17, contents#Memory(m)[old_size+20]);

    call t21 := BorrowLoc(old_size+0);

    call tmp := LdConst(29);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+22]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+23]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+22], contents#Memory(m)[old_size+23]);
    m := Memory(domain#Memory(m)[24+old_size := true], contents#Memory(m)[24+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t21, contents#Memory(m)[old_size+24]);

    call t25 := BorrowLoc(old_size+0);

    call tmp := LdConst(10);
    m := Memory(domain#Memory(m)[26+old_size := true], contents#Memory(m)[26+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[27+old_size := true], contents#Memory(m)[27+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+26]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+27]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+26], contents#Memory(m)[old_size+27]);
    m := Memory(domain#Memory(m)[28+old_size := true], contents#Memory(m)[28+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t25, contents#Memory(m)[old_size+28]);

    call t29 := BorrowLoc(old_size+0);

    call tmp := LdConst(29);
    m := Memory(domain#Memory(m)[30+old_size := true], contents#Memory(m)[30+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[31+old_size := true], contents#Memory(m)[31+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+30]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+31]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+30], contents#Memory(m)[old_size+31]);
    m := Memory(domain#Memory(m)[32+old_size := true], contents#Memory(m)[32+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t29, contents#Memory(m)[old_size+32]);

    call t33 := BorrowLoc(old_size+0);

    call tmp := LdConst(36);
    m := Memory(domain#Memory(m)[34+old_size := true], contents#Memory(m)[34+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[35+old_size := true], contents#Memory(m)[35+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+34]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+35]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+34], contents#Memory(m)[old_size+35]);
    m := Memory(domain#Memory(m)[36+old_size := true], contents#Memory(m)[36+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t33, contents#Memory(m)[old_size+36]);

    call t37 := BorrowLoc(old_size+0);

    call tmp := LdConst(52);
    m := Memory(domain#Memory(m)[38+old_size := true], contents#Memory(m)[38+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[39+old_size := true], contents#Memory(m)[39+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+38]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+39]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+38], contents#Memory(m)[old_size+39]);
    m := Memory(domain#Memory(m)[40+old_size := true], contents#Memory(m)[40+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t37, contents#Memory(m)[old_size+40]);

    call t41 := BorrowLoc(old_size+0);

    call tmp := LdConst(29);
    m := Memory(domain#Memory(m)[42+old_size := true], contents#Memory(m)[42+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[43+old_size := true], contents#Memory(m)[43+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+42]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+43]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+42], contents#Memory(m)[old_size+43]);
    m := Memory(domain#Memory(m)[44+old_size := true], contents#Memory(m)[44+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t41, contents#Memory(m)[old_size+44]);

    call t45 := BorrowLoc(old_size+0);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[46+old_size := true], contents#Memory(m)[46+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[47+old_size := true], contents#Memory(m)[47+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+46]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+47]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+46], contents#Memory(m)[old_size+47]);
    m := Memory(domain#Memory(m)[48+old_size := true], contents#Memory(m)[48+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t45, contents#Memory(m)[old_size+48]);

    call t49 := BorrowLoc(old_size+0);

    call tmp := LdConst(41);
    m := Memory(domain#Memory(m)[50+old_size := true], contents#Memory(m)[50+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[51+old_size := true], contents#Memory(m)[51+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+50]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+51]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+50], contents#Memory(m)[old_size+51]);
    m := Memory(domain#Memory(m)[52+old_size := true], contents#Memory(m)[52+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t49, contents#Memory(m)[old_size+52]);

    call t53 := BorrowLoc(old_size+0);

    call tmp := LdConst(41);
    m := Memory(domain#Memory(m)[54+old_size := true], contents#Memory(m)[54+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[55+old_size := true], contents#Memory(m)[55+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+54]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+55]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+54], contents#Memory(m)[old_size+55]);
    m := Memory(domain#Memory(m)[56+old_size := true], contents#Memory(m)[56+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t53, contents#Memory(m)[old_size+56]);

    call t57 := BorrowLoc(old_size+0);

    call tmp := LdConst(28);
    m := Memory(domain#Memory(m)[58+old_size := true], contents#Memory(m)[58+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[59+old_size := true], contents#Memory(m)[59+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+58]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+59]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+58], contents#Memory(m)[old_size+59]);
    m := Memory(domain#Memory(m)[60+old_size := true], contents#Memory(m)[60+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t57, contents#Memory(m)[old_size+60]);

    call t61 := BorrowLoc(old_size+0);

    call tmp := LdConst(45);
    m := Memory(domain#Memory(m)[62+old_size := true], contents#Memory(m)[62+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[63+old_size := true], contents#Memory(m)[63+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+62]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+63]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+62], contents#Memory(m)[old_size+63]);
    m := Memory(domain#Memory(m)[64+old_size := true], contents#Memory(m)[64+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t61, contents#Memory(m)[old_size+64]);

    call t65 := BorrowLoc(old_size+0);

    call tmp := LdConst(45);
    m := Memory(domain#Memory(m)[66+old_size := true], contents#Memory(m)[66+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[67+old_size := true], contents#Memory(m)[67+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+66]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+67]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+66], contents#Memory(m)[old_size+67]);
    m := Memory(domain#Memory(m)[68+old_size := true], contents#Memory(m)[68+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t65, contents#Memory(m)[old_size+68]);

    call t69 := BorrowLoc(old_size+0);

    call tmp := LdConst(58);
    m := Memory(domain#Memory(m)[70+old_size := true], contents#Memory(m)[70+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[71+old_size := true], contents#Memory(m)[71+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+70]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+71]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+70], contents#Memory(m)[old_size+71]);
    m := Memory(domain#Memory(m)[72+old_size := true], contents#Memory(m)[72+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t69, contents#Memory(m)[old_size+72]);

    call t73 := BorrowLoc(old_size+0);

    call tmp := LdConst(58);
    m := Memory(domain#Memory(m)[74+old_size := true], contents#Memory(m)[74+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[75+old_size := true], contents#Memory(m)[75+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+74]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+75]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+74], contents#Memory(m)[old_size+75]);
    m := Memory(domain#Memory(m)[76+old_size := true], contents#Memory(m)[76+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t73, contents#Memory(m)[old_size+76]);

    call t77 := BorrowLoc(old_size+0);

    call tmp := LdConst(56);
    m := Memory(domain#Memory(m)[78+old_size := true], contents#Memory(m)[78+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[79+old_size := true], contents#Memory(m)[79+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+78]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+79]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+78], contents#Memory(m)[old_size+79]);
    m := Memory(domain#Memory(m)[80+old_size := true], contents#Memory(m)[80+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t77, contents#Memory(m)[old_size+80]);

    call t81 := BorrowLoc(old_size+0);

    call tmp := LdConst(197);
    m := Memory(domain#Memory(m)[82+old_size := true], contents#Memory(m)[82+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[83+old_size := true], contents#Memory(m)[83+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+82]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+83]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+82], contents#Memory(m)[old_size+83]);
    m := Memory(domain#Memory(m)[84+old_size := true], contents#Memory(m)[84+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t81, contents#Memory(m)[old_size+84]);

    call t85 := BorrowLoc(old_size+0);

    call tmp := LdConst(73);
    m := Memory(domain#Memory(m)[86+old_size := true], contents#Memory(m)[86+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[87+old_size := true], contents#Memory(m)[87+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+86]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+87]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+86], contents#Memory(m)[old_size+87]);
    m := Memory(domain#Memory(m)[88+old_size := true], contents#Memory(m)[88+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t85, contents#Memory(m)[old_size+88]);

    call t89 := BorrowLoc(old_size+0);

    call tmp := LdConst(94);
    m := Memory(domain#Memory(m)[90+old_size := true], contents#Memory(m)[90+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[91+old_size := true], contents#Memory(m)[91+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+90]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+91]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+90], contents#Memory(m)[old_size+91]);
    m := Memory(domain#Memory(m)[92+old_size := true], contents#Memory(m)[92+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t89, contents#Memory(m)[old_size+92]);

    call t93 := BorrowLoc(old_size+0);

    call tmp := LdConst(51);
    m := Memory(domain#Memory(m)[94+old_size := true], contents#Memory(m)[94+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[95+old_size := true], contents#Memory(m)[95+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+94]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+95]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+94], contents#Memory(m)[old_size+95]);
    m := Memory(domain#Memory(m)[96+old_size := true], contents#Memory(m)[96+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t93, contents#Memory(m)[old_size+96]);

    call t97 := BorrowLoc(old_size+0);

    call tmp := LdConst(65);
    m := Memory(domain#Memory(m)[98+old_size := true], contents#Memory(m)[98+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[99+old_size := true], contents#Memory(m)[99+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+98]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+99]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+98], contents#Memory(m)[old_size+99]);
    m := Memory(domain#Memory(m)[100+old_size := true], contents#Memory(m)[100+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t97, contents#Memory(m)[old_size+100]);

    call t101 := BorrowLoc(old_size+0);

    call tmp := LdConst(45);
    m := Memory(domain#Memory(m)[102+old_size := true], contents#Memory(m)[102+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[103+old_size := true], contents#Memory(m)[103+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+102]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+103]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+102], contents#Memory(m)[old_size+103]);
    m := Memory(domain#Memory(m)[104+old_size := true], contents#Memory(m)[104+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t101, contents#Memory(m)[old_size+104]);

    call t105 := BorrowLoc(old_size+0);

    call tmp := LdConst(44);
    m := Memory(domain#Memory(m)[106+old_size := true], contents#Memory(m)[106+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[107+old_size := true], contents#Memory(m)[107+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+106]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+107]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+106], contents#Memory(m)[old_size+107]);
    m := Memory(domain#Memory(m)[108+old_size := true], contents#Memory(m)[108+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t105, contents#Memory(m)[old_size+108]);

    call t109 := BorrowLoc(old_size+0);

    call tmp := LdConst(41);
    m := Memory(domain#Memory(m)[110+old_size := true], contents#Memory(m)[110+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[111+old_size := true], contents#Memory(m)[111+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+110]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+111]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+110], contents#Memory(m)[old_size+111]);
    m := Memory(domain#Memory(m)[112+old_size := true], contents#Memory(m)[112+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t109, contents#Memory(m)[old_size+112]);

    call t113 := BorrowLoc(old_size+0);

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[114+old_size := true], contents#Memory(m)[114+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[115+old_size := true], contents#Memory(m)[115+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+114]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+115]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+114], contents#Memory(m)[old_size+115]);
    m := Memory(domain#Memory(m)[116+old_size := true], contents#Memory(m)[116+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t113, contents#Memory(m)[old_size+116]);

    call t117 := BorrowLoc(old_size+0);

    call tmp := LdConst(41);
    m := Memory(domain#Memory(m)[118+old_size := true], contents#Memory(m)[118+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[119+old_size := true], contents#Memory(m)[119+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+118]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+119]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+118], contents#Memory(m)[old_size+119]);
    m := Memory(domain#Memory(m)[120+old_size := true], contents#Memory(m)[120+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t117, contents#Memory(m)[old_size+120]);

    call t121 := BorrowLoc(old_size+0);

    call tmp := LdConst(45);
    m := Memory(domain#Memory(m)[122+old_size := true], contents#Memory(m)[122+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[123+old_size := true], contents#Memory(m)[123+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+122]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+123]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+122], contents#Memory(m)[old_size+123]);
    m := Memory(domain#Memory(m)[124+old_size := true], contents#Memory(m)[124+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t121, contents#Memory(m)[old_size+124]);

    call t125 := BorrowLoc(old_size+0);

    call tmp := LdConst(44);
    m := Memory(domain#Memory(m)[126+old_size := true], contents#Memory(m)[126+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[127+old_size := true], contents#Memory(m)[127+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+126]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+127]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+126], contents#Memory(m)[old_size+127]);
    m := Memory(domain#Memory(m)[128+old_size := true], contents#Memory(m)[128+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t125, contents#Memory(m)[old_size+128]);

    call t129 := BorrowLoc(old_size+0);

    call tmp := LdConst(46);
    m := Memory(domain#Memory(m)[130+old_size := true], contents#Memory(m)[130+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[131+old_size := true], contents#Memory(m)[131+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+130]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+131]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+130], contents#Memory(m)[old_size+131]);
    m := Memory(domain#Memory(m)[132+old_size := true], contents#Memory(m)[132+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t129, contents#Memory(m)[old_size+132]);

    call t133 := BorrowLoc(old_size+0);

    call tmp := LdConst(43);
    m := Memory(domain#Memory(m)[134+old_size := true], contents#Memory(m)[134+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[135+old_size := true], contents#Memory(m)[135+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+134]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+135]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+134], contents#Memory(m)[old_size+135]);
    m := Memory(domain#Memory(m)[136+old_size := true], contents#Memory(m)[136+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t133, contents#Memory(m)[old_size+136]);

    call t137 := BorrowLoc(old_size+0);

    call tmp := LdConst(49);
    m := Memory(domain#Memory(m)[138+old_size := true], contents#Memory(m)[138+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[139+old_size := true], contents#Memory(m)[139+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+138]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+139]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+138], contents#Memory(m)[old_size+139]);
    m := Memory(domain#Memory(m)[140+old_size := true], contents#Memory(m)[140+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t137, contents#Memory(m)[old_size+140]);

    call t141 := BorrowLoc(old_size+0);

    call tmp := LdConst(35);
    m := Memory(domain#Memory(m)[142+old_size := true], contents#Memory(m)[142+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[143+old_size := true], contents#Memory(m)[143+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+142]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+143]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+142], contents#Memory(m)[old_size+143]);
    m := Memory(domain#Memory(m)[144+old_size := true], contents#Memory(m)[144+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t141, contents#Memory(m)[old_size+144]);

    call t145 := BorrowLoc(old_size+0);

    call tmp := LdConst(48);
    m := Memory(domain#Memory(m)[146+old_size := true], contents#Memory(m)[146+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[147+old_size := true], contents#Memory(m)[147+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+146]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+147]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+146], contents#Memory(m)[old_size+147]);
    m := Memory(domain#Memory(m)[148+old_size := true], contents#Memory(m)[148+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t145, contents#Memory(m)[old_size+148]);

    call t149 := BorrowLoc(old_size+0);

    call tmp := LdConst(51);
    m := Memory(domain#Memory(m)[150+old_size := true], contents#Memory(m)[150+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[151+old_size := true], contents#Memory(m)[151+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+150]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+151]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+150], contents#Memory(m)[old_size+151]);
    m := Memory(domain#Memory(m)[152+old_size := true], contents#Memory(m)[152+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t149, contents#Memory(m)[old_size+152]);

    call t153 := BorrowLoc(old_size+0);

    call tmp := LdConst(49);
    m := Memory(domain#Memory(m)[154+old_size := true], contents#Memory(m)[154+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[155+old_size := true], contents#Memory(m)[155+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+154]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+155]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+154], contents#Memory(m)[old_size+155]);
    m := Memory(domain#Memory(m)[156+old_size := true], contents#Memory(m)[156+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t153, contents#Memory(m)[old_size+156]);

    call t157 := BorrowLoc(old_size+0);

    call tmp := LdConst(46);
    m := Memory(domain#Memory(m)[158+old_size := true], contents#Memory(m)[158+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[159+old_size := true], contents#Memory(m)[159+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+158]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+159]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+158], contents#Memory(m)[old_size+159]);
    m := Memory(domain#Memory(m)[160+old_size := true], contents#Memory(m)[160+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t157, contents#Memory(m)[old_size+160]);

    call t161 := BorrowLoc(old_size+0);

    call tmp := LdConst(47);
    m := Memory(domain#Memory(m)[162+old_size := true], contents#Memory(m)[162+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[163+old_size := true], contents#Memory(m)[163+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+162]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+163]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+162], contents#Memory(m)[old_size+163]);
    m := Memory(domain#Memory(m)[164+old_size := true], contents#Memory(m)[164+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t161, contents#Memory(m)[old_size+164]);

    call t165 := BorrowLoc(old_size+0);

    call tmp := LdConst(46);
    m := Memory(domain#Memory(m)[166+old_size := true], contents#Memory(m)[166+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[167+old_size := true], contents#Memory(m)[167+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+166]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+167]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+166], contents#Memory(m)[old_size+167]);
    m := Memory(domain#Memory(m)[168+old_size := true], contents#Memory(m)[168+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t165, contents#Memory(m)[old_size+168]);

    call t169 := BorrowLoc(old_size+0);

    call tmp := LdConst(39);
    m := Memory(domain#Memory(m)[170+old_size := true], contents#Memory(m)[170+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[171+old_size := true], contents#Memory(m)[171+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+170]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+171]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+170], contents#Memory(m)[old_size+171]);
    m := Memory(domain#Memory(m)[172+old_size := true], contents#Memory(m)[172+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t169, contents#Memory(m)[old_size+172]);

    call t173 := BorrowLoc(old_size+0);

    call tmp := LdConst(29);
    m := Memory(domain#Memory(m)[174+old_size := true], contents#Memory(m)[174+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[175+old_size := true], contents#Memory(m)[175+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+174]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+175]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+174], contents#Memory(m)[old_size+175]);
    m := Memory(domain#Memory(m)[176+old_size := true], contents#Memory(m)[176+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t173, contents#Memory(m)[old_size+176]);

    call t177 := BorrowLoc(old_size+0);

    call tmp := LdConst(34);
    m := Memory(domain#Memory(m)[178+old_size := true], contents#Memory(m)[178+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[179+old_size := true], contents#Memory(m)[179+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+178]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+179]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+178], contents#Memory(m)[old_size+179]);
    m := Memory(domain#Memory(m)[180+old_size := true], contents#Memory(m)[180+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t177, contents#Memory(m)[old_size+180]);

    call t181 := BorrowLoc(old_size+0);

    call tmp := LdConst(32);
    m := Memory(domain#Memory(m)[182+old_size := true], contents#Memory(m)[182+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[183+old_size := true], contents#Memory(m)[183+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+182]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+183]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+182], contents#Memory(m)[old_size+183]);
    m := Memory(domain#Memory(m)[184+old_size := true], contents#Memory(m)[184+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t181, contents#Memory(m)[old_size+184]);

    call t185 := BorrowLoc(old_size+0);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[186+old_size := true], contents#Memory(m)[186+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[187+old_size := true], contents#Memory(m)[187+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+186]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+187]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+186], contents#Memory(m)[old_size+187]);
    m := Memory(domain#Memory(m)[188+old_size := true], contents#Memory(m)[188+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t185, contents#Memory(m)[old_size+188]);

    call t189 := BorrowLoc(old_size+0);

    call tmp := LdConst(856);
    m := Memory(domain#Memory(m)[190+old_size := true], contents#Memory(m)[190+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[191+old_size := true], contents#Memory(m)[191+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+190]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+191]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+190], contents#Memory(m)[old_size+191]);
    m := Memory(domain#Memory(m)[192+old_size := true], contents#Memory(m)[192+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t189, contents#Memory(m)[old_size+192]);

    call t193 := BorrowLoc(old_size+0);

    call tmp := LdConst(929);
    m := Memory(domain#Memory(m)[194+old_size := true], contents#Memory(m)[194+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[195+old_size := true], contents#Memory(m)[195+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+194]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+195]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+194], contents#Memory(m)[old_size+195]);
    m := Memory(domain#Memory(m)[196+old_size := true], contents#Memory(m)[196+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t193, contents#Memory(m)[old_size+196]);

    call t197 := BorrowLoc(old_size+0);

    call tmp := LdConst(929);
    m := Memory(domain#Memory(m)[198+old_size := true], contents#Memory(m)[198+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[199+old_size := true], contents#Memory(m)[199+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+198]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+199]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+198], contents#Memory(m)[old_size+199]);
    m := Memory(domain#Memory(m)[200+old_size := true], contents#Memory(m)[200+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t197, contents#Memory(m)[old_size+200]);

    call t201 := BorrowLoc(old_size+0);

    call tmp := LdConst(917);
    m := Memory(domain#Memory(m)[202+old_size := true], contents#Memory(m)[202+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[203+old_size := true], contents#Memory(m)[203+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+202]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+203]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+202], contents#Memory(m)[old_size+203]);
    m := Memory(domain#Memory(m)[204+old_size := true], contents#Memory(m)[204+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t201, contents#Memory(m)[old_size+204]);

    call t205 := BorrowLoc(old_size+0);

    call tmp := LdConst(774);
    m := Memory(domain#Memory(m)[206+old_size := true], contents#Memory(m)[206+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[207+old_size := true], contents#Memory(m)[207+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+206]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+207]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+206], contents#Memory(m)[old_size+207]);
    m := Memory(domain#Memory(m)[208+old_size := true], contents#Memory(m)[208+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t205, contents#Memory(m)[old_size+208]);

    call t209 := BorrowLoc(old_size+0);

    call tmp := LdConst(29);
    m := Memory(domain#Memory(m)[210+old_size := true], contents#Memory(m)[210+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[211+old_size := true], contents#Memory(m)[211+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+210]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+211]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+210], contents#Memory(m)[old_size+211]);
    m := Memory(domain#Memory(m)[212+old_size := true], contents#Memory(m)[212+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t209, contents#Memory(m)[old_size+212]);

    call t213 := BorrowLoc(old_size+0);

    call tmp := LdConst(41);
    m := Memory(domain#Memory(m)[214+old_size := true], contents#Memory(m)[214+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[215+old_size := true], contents#Memory(m)[215+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+214]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+215]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+214], contents#Memory(m)[old_size+215]);
    m := Memory(domain#Memory(m)[216+old_size := true], contents#Memory(m)[216+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t213, contents#Memory(m)[old_size+216]);

    call t217 := BorrowLoc(old_size+0);

    call tmp := LdConst(10);
    m := Memory(domain#Memory(m)[218+old_size := true], contents#Memory(m)[218+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[219+old_size := true], contents#Memory(m)[219+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+218]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+219]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+218], contents#Memory(m)[old_size+219]);
    m := Memory(domain#Memory(m)[220+old_size := true], contents#Memory(m)[220+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t217, contents#Memory(m)[old_size+220]);

    call t221 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[222+old_size := true], contents#Memory(m)[222+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[223+old_size := true], contents#Memory(m)[223+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+222]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+223]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+222], contents#Memory(m)[old_size+223]);
    m := Memory(domain#Memory(m)[224+old_size := true], contents#Memory(m)[224+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t221, contents#Memory(m)[old_size+224]);

    call t225 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[226+old_size := true], contents#Memory(m)[226+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[227+old_size := true], contents#Memory(m)[227+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+226]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+227]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+226], contents#Memory(m)[old_size+227]);
    m := Memory(domain#Memory(m)[228+old_size := true], contents#Memory(m)[228+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t225, contents#Memory(m)[old_size+228]);

    call t229 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[230+old_size := true], contents#Memory(m)[230+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[231+old_size := true], contents#Memory(m)[231+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+230]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+231]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+230], contents#Memory(m)[old_size+231]);
    m := Memory(domain#Memory(m)[232+old_size := true], contents#Memory(m)[232+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t229, contents#Memory(m)[old_size+232]);

    call t233 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[234+old_size := true], contents#Memory(m)[234+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[235+old_size := true], contents#Memory(m)[235+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+234]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+235]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+234], contents#Memory(m)[old_size+235]);
    m := Memory(domain#Memory(m)[236+old_size := true], contents#Memory(m)[236+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t233, contents#Memory(m)[old_size+236]);

    call t237 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[238+old_size := true], contents#Memory(m)[238+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[239+old_size := true], contents#Memory(m)[239+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+238]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+239]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+238], contents#Memory(m)[old_size+239]);
    m := Memory(domain#Memory(m)[240+old_size := true], contents#Memory(m)[240+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t237, contents#Memory(m)[old_size+240]);

    call t241 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[242+old_size := true], contents#Memory(m)[242+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[243+old_size := true], contents#Memory(m)[243+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+242]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+243]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+242], contents#Memory(m)[old_size+243]);
    m := Memory(domain#Memory(m)[244+old_size := true], contents#Memory(m)[244+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t241, contents#Memory(m)[old_size+244]);

    call t245 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[246+old_size := true], contents#Memory(m)[246+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[247+old_size := true], contents#Memory(m)[247+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+246]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+247]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+246], contents#Memory(m)[old_size+247]);
    m := Memory(domain#Memory(m)[248+old_size := true], contents#Memory(m)[248+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t245, contents#Memory(m)[old_size+248]);

    call t249 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[250+old_size := true], contents#Memory(m)[250+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[251+old_size := true], contents#Memory(m)[251+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+250]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+251]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+250], contents#Memory(m)[old_size+251]);
    m := Memory(domain#Memory(m)[252+old_size := true], contents#Memory(m)[252+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t249, contents#Memory(m)[old_size+252]);

    call t253 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[254+old_size := true], contents#Memory(m)[254+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[255+old_size := true], contents#Memory(m)[255+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+254]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+255]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+254], contents#Memory(m)[old_size+255]);
    m := Memory(domain#Memory(m)[256+old_size := true], contents#Memory(m)[256+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t253, contents#Memory(m)[old_size+256]);

    call t257 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[258+old_size := true], contents#Memory(m)[258+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[259+old_size := true], contents#Memory(m)[259+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+258]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+259]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+258], contents#Memory(m)[old_size+259]);
    m := Memory(domain#Memory(m)[260+old_size := true], contents#Memory(m)[260+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t257, contents#Memory(m)[old_size+260]);

    call t261 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[262+old_size := true], contents#Memory(m)[262+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[263+old_size := true], contents#Memory(m)[263+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+262]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+263]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+262], contents#Memory(m)[old_size+263]);
    m := Memory(domain#Memory(m)[264+old_size := true], contents#Memory(m)[264+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t261, contents#Memory(m)[old_size+264]);

    call t265 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[266+old_size := true], contents#Memory(m)[266+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[267+old_size := true], contents#Memory(m)[267+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+266]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+267]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+266], contents#Memory(m)[old_size+267]);
    m := Memory(domain#Memory(m)[268+old_size := true], contents#Memory(m)[268+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t265, contents#Memory(m)[old_size+268]);

    call t269 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[270+old_size := true], contents#Memory(m)[270+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[271+old_size := true], contents#Memory(m)[271+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+270]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+271]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+270], contents#Memory(m)[old_size+271]);
    m := Memory(domain#Memory(m)[272+old_size := true], contents#Memory(m)[272+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t269, contents#Memory(m)[old_size+272]);

    call t273 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[274+old_size := true], contents#Memory(m)[274+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[275+old_size := true], contents#Memory(m)[275+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+274]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+275]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+274], contents#Memory(m)[old_size+275]);
    m := Memory(domain#Memory(m)[276+old_size := true], contents#Memory(m)[276+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t273, contents#Memory(m)[old_size+276]);

    call t277 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[278+old_size := true], contents#Memory(m)[278+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[279+old_size := true], contents#Memory(m)[279+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+278]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+279]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+278], contents#Memory(m)[old_size+279]);
    m := Memory(domain#Memory(m)[280+old_size := true], contents#Memory(m)[280+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t277, contents#Memory(m)[old_size+280]);

    call t281 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[282+old_size := true], contents#Memory(m)[282+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[283+old_size := true], contents#Memory(m)[283+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+282]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+283]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+282], contents#Memory(m)[old_size+283]);
    m := Memory(domain#Memory(m)[284+old_size := true], contents#Memory(m)[284+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t281, contents#Memory(m)[old_size+284]);

    call t285 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[286+old_size := true], contents#Memory(m)[286+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[287+old_size := true], contents#Memory(m)[287+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+286]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+287]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+286], contents#Memory(m)[old_size+287]);
    m := Memory(domain#Memory(m)[288+old_size := true], contents#Memory(m)[288+old_size := tmp]);

    call Vector_push_back(GasSchedule_Cost_type_value(), t285, contents#Memory(m)[old_size+288]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[289+old_size := true], contents#Memory(m)[289+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[290+old_size := true], contents#Memory(m)[290+old_size := tmp]);

    assume has_type(Vector_T_type_value(GasSchedule_Cost_type_value()), contents#Memory(m)[old_size+289]);

    assume has_type(Vector_T_type_value(GasSchedule_Cost_type_value()), contents#Memory(m)[old_size+290]);

    call tmp := Pack_GasSchedule_T(contents#Memory(m)[old_size+289], contents#Memory(m)[old_size+290]);
    m := Memory(domain#Memory(m)[291+old_size := true], contents#Memory(m)[291+old_size := tmp]);

    call MoveToSender(GasSchedule_T_type_value(), contents#Memory(m)[old_size+291]);

    return;

}

procedure GasSchedule_initialize_verify () returns ()
{
    call GasSchedule_initialize();
}

procedure {:inline 1} GasSchedule_new_cost (arg0: Value, arg1: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // GasSchedule_Cost_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);
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

    assume has_type(IntegerType(), contents#Memory(m)[old_size+2]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+3]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+2], contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+4];
    return;

}

procedure GasSchedule_new_cost_verify (arg0: Value, arg1: Value) returns (ret0: Value)
{
    call ret0 := GasSchedule_new_cost(arg0, arg1);
}

procedure {:inline 1} GasSchedule_has_gas_schedule_resource () returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 2;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := Exists(contents#Memory(m)[old_size+0], GasSchedule_T_type_value());
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+1];
    return;

}

procedure GasSchedule_has_gas_schedule_resource_verify () returns (ret0: Value)
{
    call ret0 := GasSchedule_has_gas_schedule_resource();
}

procedure {:inline 1} GasSchedule_instruction_table_size () returns (ret0: Value)
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

procedure {:inline 1} LibraAccount_make (arg0: Value) returns (ret0: Value)
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

procedure {:inline 1} LibraAccount_save_account (arg0: Value, arg1: Value) returns ();
procedure {:inline 1} LibraAccount_balance_for_account (arg0: Reference) returns (ret0: Value)
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

procedure {:inline 1} LibraAccount_write_to_event_store (tv0: TypeValue, arg0: Value, arg1: Value, arg2: Value) returns ();
procedure {:inline 1} LibraAccount_destroy_handle (tv0: TypeValue, arg0: Value) returns ()
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

procedure {:inline 1} LibraSystem_initialize_validator_set () returns ()
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Value; // BooleanType()
    var t3: Value; // BooleanType()
    var t4: Value; // IntegerType()
    var t5: Value; // Vector_T_type_value(LibraSystem_ValidatorInfo_type_value())
    var t6: Value; // Vector_T_type_value(AddressType())
    var t7: Value; // LibraAccount_EventHandle_type_value(LibraSystem_ValidatorSetChangeEvent_type_value())
    var t8: Value; // LibraSystem_ValidatorSet_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 9;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := LdAddr(472);
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
    call t5 := Vector_empty(LibraSystem_ValidatorInfo_type_value());
    assume has_type(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()), t5);

    m := Memory(domain#Memory(m)[old_size+5 := true], contents#Memory(m)[old_size+5 := t5]);

    call t6 := Vector_empty(AddressType());
    assume has_type(Vector_T_type_value(AddressType()), t6);

    m := Memory(domain#Memory(m)[old_size+6 := true], contents#Memory(m)[old_size+6 := t6]);

    call t7 := LibraAccount_new_event_handle(LibraSystem_ValidatorSetChangeEvent_type_value());
    assume has_type(LibraAccount_EventHandle_type_value(LibraSystem_ValidatorSetChangeEvent_type_value()), t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    assume has_type(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()), contents#Memory(m)[old_size+5]);

    assume has_type(Vector_T_type_value(AddressType()), contents#Memory(m)[old_size+6]);

    assume has_type(LibraAccount_EventHandle_type_value(LibraSystem_ValidatorSetChangeEvent_type_value()), contents#Memory(m)[old_size+7]);

    call tmp := Pack_LibraSystem_ValidatorSet(contents#Memory(m)[old_size+5], contents#Memory(m)[old_size+6], contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call MoveToSender(LibraSystem_ValidatorSet_type_value(), contents#Memory(m)[old_size+8]);

    return;

}

procedure LibraSystem_initialize_validator_set_verify () returns ()
{
    call LibraSystem_initialize_validator_set();
}

procedure {:inline 1} LibraSystem_initialize_block_metadata () returns ()
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Value; // BooleanType()
    var t3: Value; // BooleanType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // ByteArrayType()
    var t8: Value; // AddressType()
    var t9: Value; // LibraSystem_BlockMetadata_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 10;

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
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    // unimplemented instruction

    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+5]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+6]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+7]);

    assume has_type(AddressType(), contents#Memory(m)[old_size+8]);

    call tmp := Pack_LibraSystem_BlockMetadata(contents#Memory(m)[old_size+5], contents#Memory(m)[old_size+6], contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call MoveToSender(LibraSystem_BlockMetadata_type_value(), contents#Memory(m)[old_size+9]);

    return;

}

procedure LibraSystem_initialize_block_metadata_verify () returns ()
{
    call LibraSystem_initialize_block_metadata();
}

procedure {:inline 1} LibraSystem_get_address (arg0: Reference) returns (ret0: Reference)
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t1: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
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

    call t2 := BorrowField(t1, LibraSystem_ValidatorInfo_addr);

    ret0 := t2;
    return;

}

procedure LibraSystem_get_address_verify (arg0: Reference) returns (ret0: Reference)
{
    call ret0 := LibraSystem_get_address(arg0);
}

procedure {:inline 1} LibraSystem_get_consensus_pubkey (arg0: Reference) returns (ret0: Reference)
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t1: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 3;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, LibraSystem_ValidatorInfo_consensus_pubkey);

    ret0 := t2;
    return;

}

procedure LibraSystem_get_consensus_pubkey_verify (arg0: Reference) returns (ret0: Reference)
{
    call ret0 := LibraSystem_get_consensus_pubkey(arg0);
}

procedure {:inline 1} LibraSystem_get_consensus_voting_power (arg0: Reference) returns (ret0: Reference)
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t1: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t2: Reference; // ReferenceType(IntegerType())

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 3;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, LibraSystem_ValidatorInfo_consensus_voting_power);

    ret0 := t2;
    return;

}

procedure LibraSystem_get_consensus_voting_power_verify (arg0: Reference) returns (ret0: Reference)
{
    call ret0 := LibraSystem_get_consensus_voting_power(arg0);
}

procedure {:inline 1} LibraSystem_get_network_signing_pubkey (arg0: Reference) returns (ret0: Reference)
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t1: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 3;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, LibraSystem_ValidatorInfo_network_signing_pubkey);

    ret0 := t2;
    return;

}

procedure LibraSystem_get_network_signing_pubkey_verify (arg0: Reference) returns (ret0: Reference)
{
    call ret0 := LibraSystem_get_network_signing_pubkey(arg0);
}

procedure {:inline 1} LibraSystem_get_network_identity_pubkey (arg0: Reference) returns (ret0: Reference)
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t1: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 3;
    t0 := arg0;

    // bytecode translation starts here
    call t1 := CopyOrMoveRef(t0);

    call t2 := BorrowField(t1, LibraSystem_ValidatorInfo_network_identity_pubkey);

    ret0 := t2;
    return;

}

procedure LibraSystem_get_network_identity_pubkey_verify (arg0: Reference) returns (ret0: Reference)
{
    call ret0 := LibraSystem_get_network_identity_pubkey(arg0);
}

procedure {:inline 1} LibraSystem_block_prologue (arg0: Value, arg1: Value, arg2: Value, arg3: Value) returns ()
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // ByteArrayType()
    var t2: Value; // ByteArrayType()
    var t3: Value; // AddressType()
    var t4: Value; // IntegerType()
    var t5: Value; // ByteArrayType()
    var t6: Value; // ByteArrayType()
    var t7: Value; // AddressType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);
    assume has_type(ByteArrayType(), arg1);
    assume has_type(ByteArrayType(), arg2);
    assume has_type(AddressType(), arg3);

    old_size := m_size;
    m_size := m_size + 8;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size :=  arg2]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size :=  arg3]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call LibraSystem_process_block_prologue(contents#Memory(m)[old_size+4], contents#Memory(m)[old_size+5], contents#Memory(m)[old_size+6], contents#Memory(m)[old_size+7]);

    call LibraSystem_reconfigure();

    return;

}

procedure LibraSystem_block_prologue_verify (arg0: Value, arg1: Value, arg2: Value, arg3: Value) returns ()
{
    call LibraSystem_block_prologue(arg0, arg1, arg2, arg3);
}

procedure {:inline 1} LibraSystem_process_block_prologue (arg0: Value, arg1: Value, arg2: Value, arg3: Value) returns ()
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // ByteArrayType()
    var t2: Value; // ByteArrayType()
    var t3: Value; // AddressType()
    var t4: Reference; // ReferenceType(LibraSystem_BlockMetadata_type_value())
    var t5: Value; // AddressType()
    var t6: Reference; // ReferenceType(LibraSystem_BlockMetadata_type_value())
    var t7: Value; // IntegerType()
    var t8: Reference; // ReferenceType(LibraSystem_BlockMetadata_type_value())
    var t9: Reference; // ReferenceType(IntegerType())
    var t10: Value; // IntegerType()
    var t11: Value; // BooleanType()
    var t12: Value; // BooleanType()
    var t13: Value; // IntegerType()
    var t14: Value; // AddressType()
    var t15: Value; // BooleanType()
    var t16: Value; // BooleanType()
    var t17: Value; // IntegerType()
    var t18: Value; // ByteArrayType()
    var t19: Reference; // ReferenceType(LibraSystem_BlockMetadata_type_value())
    var t20: Reference; // ReferenceType(ByteArrayType())
    var t21: Value; // IntegerType()
    var t22: Reference; // ReferenceType(LibraSystem_BlockMetadata_type_value())
    var t23: Reference; // ReferenceType(IntegerType())
    var t24: Value; // AddressType()
    var t25: Reference; // ReferenceType(LibraSystem_BlockMetadata_type_value())
    var t26: Reference; // ReferenceType(AddressType())
    var t27: Reference; // ReferenceType(LibraSystem_BlockMetadata_type_value())
    var t28: Reference; // ReferenceType(IntegerType())
    var t29: Value; // IntegerType()
    var t30: Value; // IntegerType()
    var t31: Value; // IntegerType()
    var t32: Reference; // ReferenceType(LibraSystem_BlockMetadata_type_value())
    var t33: Reference; // ReferenceType(IntegerType())

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);
    assume has_type(ByteArrayType(), arg1);
    assume has_type(ByteArrayType(), arg2);
    assume has_type(AddressType(), arg3);

    old_size := m_size;
    m_size := m_size + 34;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size :=  arg2]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size :=  arg3]);

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := BorrowGlobal(contents#Memory(m)[old_size+5], LibraSystem_BlockMetadata_type_value());

    call t4 := CopyOrMoveRef(t6);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t8 := CopyOrMoveRef(t4);

    call t9 := BorrowField(t8, LibraSystem_BlockMetadata_timestamp);

    call tmp := ReadRef(t9);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := Gt(contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+10]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 12];
    if (!b#Boolean(tmp)) { goto Label_12; }

    call tmp := LdConst(5001);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    assert false;

Label_12:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call t15 := LibraSystem_is_validator(contents#Memory(m)[old_size+14]);
    assume has_type(BooleanType(), t15);

    m := Memory(domain#Memory(m)[old_size+15 := true], contents#Memory(m)[old_size+15 := t15]);

    call tmp := Not(contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 16];
    if (!b#Boolean(tmp)) { goto Label_18; }

    call tmp := LdConst(5002);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    assert false;

Label_18:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call t19 := CopyOrMoveRef(t4);

    call t20 := BorrowField(t19, LibraSystem_BlockMetadata_id);

    call WriteRef(t20, contents#Memory(m)[old_size+18]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    call t22 := CopyOrMoveRef(t4);

    call t23 := BorrowField(t22, LibraSystem_BlockMetadata_timestamp);

    call WriteRef(t23, contents#Memory(m)[old_size+21]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[24+old_size := true], contents#Memory(m)[24+old_size := tmp]);

    call t25 := CopyOrMoveRef(t4);

    call t26 := BorrowField(t25, LibraSystem_BlockMetadata_proposer);

    call WriteRef(t26, contents#Memory(m)[old_size+24]);

    call t27 := CopyOrMoveRef(t4);

    call t28 := BorrowField(t27, LibraSystem_BlockMetadata_height);

    call tmp := ReadRef(t28);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[29+old_size := true], contents#Memory(m)[29+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[30+old_size := true], contents#Memory(m)[30+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+29], contents#Memory(m)[old_size+30]);
    m := Memory(domain#Memory(m)[31+old_size := true], contents#Memory(m)[31+old_size := tmp]);

    call t32 := CopyOrMoveRef(t4);

    call t33 := BorrowField(t32, LibraSystem_BlockMetadata_height);

    call WriteRef(t33, contents#Memory(m)[old_size+31]);

    return;

}

procedure LibraSystem_process_block_prologue_verify (arg0: Value, arg1: Value, arg2: Value, arg3: Value) returns ()
{
    call LibraSystem_process_block_prologue(arg0, arg1, arg2, arg3);
}

procedure {:inline 1} LibraSystem_get_current_block_height () returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraSystem_BlockMetadata_type_value())
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

    call t1 := BorrowGlobal(contents#Memory(m)[old_size+0], LibraSystem_BlockMetadata_type_value());

    call t2 := BorrowField(t1, LibraSystem_BlockMetadata_height);

    call tmp := ReadRef(t2);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure LibraSystem_get_current_block_height_verify () returns (ret0: Value)
{
    call ret0 := LibraSystem_get_current_block_height();
}

procedure {:inline 1} LibraSystem_get_current_block_id () returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraSystem_BlockMetadata_type_value())
    var t2: Reference; // ReferenceType(ByteArrayType())
    var t3: Value; // ByteArrayType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call t1 := BorrowGlobal(contents#Memory(m)[old_size+0], LibraSystem_BlockMetadata_type_value());

    call t2 := BorrowField(t1, LibraSystem_BlockMetadata_id);

    call tmp := ReadRef(t2);
    assume has_type(ByteArrayType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure LibraSystem_get_current_block_id_verify () returns (ret0: Value)
{
    call ret0 := LibraSystem_get_current_block_id();
}

procedure {:inline 1} LibraSystem_get_current_timestamp () returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraSystem_BlockMetadata_type_value())
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

    call t1 := BorrowGlobal(contents#Memory(m)[old_size+0], LibraSystem_BlockMetadata_type_value());

    call t2 := BorrowField(t1, LibraSystem_BlockMetadata_timestamp);

    call tmp := ReadRef(t2);
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure LibraSystem_get_current_timestamp_verify () returns (ret0: Value)
{
    call ret0 := LibraSystem_get_current_timestamp();
}

procedure {:inline 1} LibraSystem_get_current_proposer () returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraSystem_BlockMetadata_type_value())
    var t2: Reference; // ReferenceType(AddressType())
    var t3: Value; // AddressType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call t1 := BorrowGlobal(contents#Memory(m)[old_size+0], LibraSystem_BlockMetadata_type_value());

    call t2 := BorrowField(t1, LibraSystem_BlockMetadata_proposer);

    call tmp := ReadRef(t2);
    assume has_type(AddressType(), tmp);

    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+3];
    return;

}

procedure LibraSystem_get_current_proposer_verify () returns (ret0: Value)
{
    call ret0 := LibraSystem_get_current_proposer();
}

procedure {:inline 1} LibraSystem_validator_set_size () returns (ret0: Value)
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t1: Value; // AddressType()
    var t2: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t3: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t4: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t5: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 6;

    // bytecode translation starts here
    call tmp := LdAddr(472);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t2 := BorrowGlobal(contents#Memory(m)[old_size+1], LibraSystem_ValidatorSet_type_value());

    call t0 := CopyOrMoveRef(t2);

    call t3 := CopyOrMoveRef(t0);

    call t4 := BorrowField(t3, LibraSystem_ValidatorSet_validators);

    call t5 := Vector_length(LibraSystem_ValidatorInfo_type_value(), t4);
    assume has_type(IntegerType(), t5);

    m := Memory(domain#Memory(m)[old_size+5 := true], contents#Memory(m)[old_size+5 := t5]);

    ret0 := contents#Memory(m)[old_size+5];
    return;

}

procedure LibraSystem_validator_set_size_verify () returns (ret0: Value)
{
    call ret0 := LibraSystem_validator_set_size();
}

procedure {:inline 1} LibraSystem_is_validator_ (arg0: Reference, arg1: Reference) returns (ret0: Value)
{
    // declare local variables
    var t0: Reference; // ReferenceType(AddressType())
    var t1: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t5: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // BooleanType()
    var t10: Value; // BooleanType()
    var t11: Value; // IntegerType()
    var t12: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t13: Value; // IntegerType()
    var t14: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t15: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t16: Reference; // ReferenceType(AddressType())
    var t17: Reference; // ReferenceType(AddressType())
    var t18: Value; // BooleanType()
    var t19: Value; // BooleanType()
    var t20: Value; // IntegerType()
    var t21: Value; // IntegerType()
    var t22: Value; // IntegerType()
    var t23: Value; // IntegerType()
    var t24: Value; // IntegerType()
    var t25: Value; // BooleanType()
    var t26: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t27: Value; // IntegerType()
    var t28: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t29: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 30;
    t0 := arg0;
    t1 := arg1;

    // bytecode translation starts here
    call t5 := CopyOrMoveRef(t1);

    call t6 := Vector_length(LibraSystem_ValidatorInfo_type_value(), t5);
    assume has_type(IntegerType(), t6);

    m := Memory(domain#Memory(m)[old_size+6 := true], contents#Memory(m)[old_size+6 := t6]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    tmp := Boolean(is_equal(IntegerType(), contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+8]));
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 9];
    if (!b#Boolean(tmp)) { goto Label_9; }

    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+10];
    return;

Label_9:
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t12 := CopyOrMoveRef(t1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    call t14 := Vector_borrow(LibraSystem_ValidatorInfo_type_value(), t12, contents#Memory(m)[old_size+13]);


    call t4 := CopyOrMoveRef(t14);

Label_15:
    call t15 := CopyOrMoveRef(t4);

    call t16 := BorrowField(t15, LibraSystem_ValidatorInfo_addr);

    call t17 := CopyOrMoveRef(t0);

    tmp := Boolean(is_equal(ReferenceType(AddressType()), contents#Memory(m)[old_size+16], contents#Memory(m)[old_size+17]));
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 18];
    if (!b#Boolean(tmp)) { goto Label_22; }

    call tmp := LdTrue();
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+19];
    return;

Label_22:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+20], contents#Memory(m)[old_size+21]);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+22]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[24+old_size := true], contents#Memory(m)[24+old_size := tmp]);

    call tmp := Ge(contents#Memory(m)[old_size+23], contents#Memory(m)[old_size+24]);
    m := Memory(domain#Memory(m)[25+old_size := true], contents#Memory(m)[25+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 25];
    if (!b#Boolean(tmp)) { goto Label_31; }

    goto Label_36;

Label_31:
    call t26 := CopyOrMoveRef(t1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[27+old_size := true], contents#Memory(m)[27+old_size := tmp]);

    call t28 := Vector_borrow(LibraSystem_ValidatorInfo_type_value(), t26, contents#Memory(m)[old_size+27]);


    call t4 := CopyOrMoveRef(t28);

    goto Label_15;

Label_36:
    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[29+old_size := true], contents#Memory(m)[29+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+29];
    return;

}

procedure LibraSystem_is_validator__verify (arg0: Reference, arg1: Reference) returns (ret0: Value)
{
    call ret0 := LibraSystem_is_validator_(arg0, arg1);
}

procedure {:inline 1} LibraSystem_is_validator (arg0: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(AddressType())
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t4: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t5: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 6;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call t1 := BorrowLoc(old_size+0);

    call tmp := LdAddr(472);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], LibraSystem_ValidatorSet_type_value());

    call t4 := BorrowField(t3, LibraSystem_ValidatorSet_validators);

    call t5 := LibraSystem_is_validator_(t1, t4);
    assume has_type(BooleanType(), t5);

    m := Memory(domain#Memory(m)[old_size+5 := true], contents#Memory(m)[old_size+5 := t5]);

    ret0 := contents#Memory(m)[old_size+5];
    return;

}

procedure LibraSystem_is_validator_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraSystem_is_validator(arg0);
}

procedure {:inline 1} LibraSystem_add_validator (arg0: Value) returns ()
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t2: Reference; // ReferenceType(Vector_T_type_value(AddressType()))
    var t3: Value; // AddressType()
    var t4: Value; // AddressType()
    var t5: Value; // BooleanType()
    var t6: Value; // BooleanType()
    var t7: Value; // IntegerType()
    var t8: Value; // AddressType()
    var t9: Value; // BooleanType()
    var t10: Value; // BooleanType()
    var t11: Value; // IntegerType()
    var t12: Value; // AddressType()
    var t13: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t14: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t15: Reference; // ReferenceType(Vector_T_type_value(AddressType()))
    var t16: Reference; // ReferenceType(AddressType())
    var t17: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t18: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t19: Value; // BooleanType()
    var t20: Value; // BooleanType()
    var t21: Value; // BooleanType()
    var t22: Value; // IntegerType()
    var t23: Reference; // ReferenceType(Vector_T_type_value(AddressType()))
    var t24: Reference; // ReferenceType(Vector_T_type_value(AddressType()))
    var t25: Reference; // ReferenceType(AddressType())
    var t26: Value; // BooleanType()
    var t27: Value; // BooleanType()
    var t28: Value; // BooleanType()
    var t29: Value; // IntegerType()
    var t30: Reference; // ReferenceType(Vector_T_type_value(AddressType()))
    var t31: Value; // AddressType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 32;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    tmp := Boolean(is_equal(AddressType(), contents#Memory(m)[old_size+3], contents#Memory(m)[old_size+4]));
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 6];
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    assert false;

Label_7:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call t9 := ValidatorConfig_has(contents#Memory(m)[old_size+8]);
    assume has_type(BooleanType(), t9);

    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    call tmp := Not(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 10];
    if (!b#Boolean(tmp)) { goto Label_13; }

    call tmp := LdConst(17);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    assert false;

Label_13:
    call tmp := LdAddr(472);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call t13 := BorrowGlobal(contents#Memory(m)[old_size+12], LibraSystem_ValidatorSet_type_value());

    call t1 := CopyOrMoveRef(t13);

    call t14 := CopyOrMoveRef(t1);

    call t15 := BorrowField(t14, LibraSystem_ValidatorSet_additions);

    call t2 := CopyOrMoveRef(t15);

    call t16 := BorrowLoc(old_size+0);

    call t17 := CopyOrMoveRef(t1);

    call t18 := BorrowField(t17, LibraSystem_ValidatorSet_validators);

    call t19 := LibraSystem_is_validator_(t16, t18);
    assume has_type(BooleanType(), t19);

    m := Memory(domain#Memory(m)[old_size+19 := true], contents#Memory(m)[old_size+19 := t19]);

    call tmp := Not(contents#Memory(m)[old_size+19]);
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+20]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 21];
    if (!b#Boolean(tmp)) { goto Label_28; }

    call tmp := LdConst(18);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    assert false;

Label_28:
    call t23 := CopyOrMoveRef(t2);

    call t24 := FreezeRef(t23);

    call t25 := BorrowLoc(old_size+0);

    call t26 := Vector_contains(AddressType(), t24, t25);
    assume has_type(BooleanType(), t26);

    m := Memory(domain#Memory(m)[old_size+26 := true], contents#Memory(m)[old_size+26 := t26]);

    call tmp := Not(contents#Memory(m)[old_size+26]);
    m := Memory(domain#Memory(m)[27+old_size := true], contents#Memory(m)[27+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+27]);
    m := Memory(domain#Memory(m)[28+old_size := true], contents#Memory(m)[28+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 28];
    if (!b#Boolean(tmp)) { goto Label_37; }

    call tmp := LdConst(19);
    m := Memory(domain#Memory(m)[29+old_size := true], contents#Memory(m)[29+old_size := tmp]);

    assert false;

Label_37:
    call t30 := CopyOrMoveRef(t2);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[31+old_size := true], contents#Memory(m)[31+old_size := tmp]);

    call Vector_push_back(AddressType(), t30, contents#Memory(m)[old_size+31]);

    return;

}

procedure LibraSystem_add_validator_verify (arg0: Value) returns ()
{
    call LibraSystem_add_validator(arg0);
}

procedure {:inline 1} LibraSystem_copy_validator_info (arg0: Reference) returns (ret0: Value)
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t1: Value; // ByteArrayType()
    var t2: Value; // ByteArrayType()
    var t3: Value; // ByteArrayType()
    var t4: Value; // ValidatorConfig_Config_type_value()
    var t5: Value; // BooleanType()
    var t6: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t7: Reference; // ReferenceType(AddressType())
    var t8: Value; // AddressType()
    var t9: Value; // ValidatorConfig_Config_type_value()
    var t10: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t11: Value; // ByteArrayType()
    var t12: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t13: Value; // ByteArrayType()
    var t14: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t15: Value; // ByteArrayType()
    var t16: Value; // BooleanType()
    var t17: Reference; // ReferenceType(ByteArrayType())
    var t18: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t19: Reference; // ReferenceType(ByteArrayType())
    var t20: Value; // BooleanType()
    var t21: Value; // ByteArrayType()
    var t22: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t23: Reference; // ReferenceType(ByteArrayType())
    var t24: Value; // BooleanType()
    var t25: Reference; // ReferenceType(ByteArrayType())
    var t26: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t27: Reference; // ReferenceType(ByteArrayType())
    var t28: Value; // BooleanType()
    var t29: Value; // ByteArrayType()
    var t30: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t31: Reference; // ReferenceType(ByteArrayType())
    var t32: Value; // BooleanType()
    var t33: Reference; // ReferenceType(ByteArrayType())
    var t34: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t35: Reference; // ReferenceType(ByteArrayType())
    var t36: Value; // BooleanType()
    var t37: Value; // ByteArrayType()
    var t38: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t39: Reference; // ReferenceType(ByteArrayType())
    var t40: Value; // BooleanType()
    var t41: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 42;
    t0 := arg0;

    // bytecode translation starts here
    call t6 := CopyOrMoveRef(t0);

    call t7 := BorrowField(t6, LibraSystem_ValidatorInfo_addr);

    call tmp := ReadRef(t7);
    assume has_type(AddressType(), tmp);

    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call t9 := ValidatorConfig_config(contents#Memory(m)[old_size+8]);
    assume has_type(ValidatorConfig_Config_type_value(), t9);

    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t10 := BorrowLoc(old_size+4);

    call t11 := ValidatorConfig_consensus_pubkey(t10);
    assume has_type(ByteArrayType(), t11);

    m := Memory(domain#Memory(m)[old_size+11 := true], contents#Memory(m)[old_size+11 := t11]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t12 := BorrowLoc(old_size+4);

    call t13 := ValidatorConfig_network_signing_pubkey(t12);
    assume has_type(ByteArrayType(), t13);

    m := Memory(domain#Memory(m)[old_size+13 := true], contents#Memory(m)[old_size+13 := t13]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+13]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t14 := BorrowLoc(old_size+4);

    call t15 := ValidatorConfig_network_identity_pubkey(t14);
    assume has_type(ByteArrayType(), t15);

    m := Memory(domain#Memory(m)[old_size+15 := true], contents#Memory(m)[old_size+15 := t15]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+16]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t17 := BorrowLoc(old_size+1);

    call t18 := CopyOrMoveRef(t0);

    call t19 := BorrowField(t18, LibraSystem_ValidatorInfo_consensus_pubkey);

    tmp := Boolean(!is_equal(ReferenceType(ByteArrayType()), contents#Memory(m)[old_size+17], contents#Memory(m)[old_size+19]));
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 20];
    if (!b#Boolean(tmp)) { goto Label_27; }

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    call t22 := CopyOrMoveRef(t0);

    call t23 := BorrowField(t22, LibraSystem_ValidatorInfo_consensus_pubkey);

    call WriteRef(t23, contents#Memory(m)[old_size+21]);

    call tmp := LdTrue();
    m := Memory(domain#Memory(m)[24+old_size := true], contents#Memory(m)[24+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+24]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

Label_27:
    call t25 := BorrowLoc(old_size+2);

    call t26 := CopyOrMoveRef(t0);

    call t27 := BorrowField(t26, LibraSystem_ValidatorInfo_network_signing_pubkey);

    tmp := Boolean(!is_equal(ReferenceType(ByteArrayType()), contents#Memory(m)[old_size+25], contents#Memory(m)[old_size+27]));
    m := Memory(domain#Memory(m)[28+old_size := true], contents#Memory(m)[28+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 28];
    if (!b#Boolean(tmp)) { goto Label_38; }

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[29+old_size := true], contents#Memory(m)[29+old_size := tmp]);

    call t30 := CopyOrMoveRef(t0);

    call t31 := BorrowField(t30, LibraSystem_ValidatorInfo_network_signing_pubkey);

    call WriteRef(t31, contents#Memory(m)[old_size+29]);

    call tmp := LdTrue();
    m := Memory(domain#Memory(m)[32+old_size := true], contents#Memory(m)[32+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+32]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

Label_38:
    call t33 := BorrowLoc(old_size+3);

    call t34 := CopyOrMoveRef(t0);

    call t35 := BorrowField(t34, LibraSystem_ValidatorInfo_network_identity_pubkey);

    tmp := Boolean(!is_equal(ReferenceType(ByteArrayType()), contents#Memory(m)[old_size+33], contents#Memory(m)[old_size+35]));
    m := Memory(domain#Memory(m)[36+old_size := true], contents#Memory(m)[36+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 36];
    if (!b#Boolean(tmp)) { goto Label_49; }

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[37+old_size := true], contents#Memory(m)[37+old_size := tmp]);

    call t38 := CopyOrMoveRef(t0);

    call t39 := BorrowField(t38, LibraSystem_ValidatorInfo_network_identity_pubkey);

    call WriteRef(t39, contents#Memory(m)[old_size+37]);

    call tmp := LdTrue();
    m := Memory(domain#Memory(m)[40+old_size := true], contents#Memory(m)[40+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+40]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

Label_49:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[41+old_size := true], contents#Memory(m)[41+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+41];
    return;

}

procedure LibraSystem_copy_validator_info_verify (arg0: Reference) returns (ret0: Value)
{
    call ret0 := LibraSystem_copy_validator_info(arg0);
}

procedure {:inline 1} LibraSystem_make_info (arg0: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // ValidatorConfig_Config_type_value()
    var t2: Value; // AddressType()
    var t3: Value; // ValidatorConfig_Config_type_value()
    var t4: Value; // AddressType()
    var t5: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t6: Value; // ByteArrayType()
    var t7: Value; // IntegerType()
    var t8: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t9: Value; // ByteArrayType()
    var t10: Reference; // ReferenceType(ValidatorConfig_Config_type_value())
    var t11: Value; // ByteArrayType()
    var t12: Value; // LibraSystem_ValidatorInfo_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 13;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := ValidatorConfig_config(contents#Memory(m)[old_size+2]);
    assume has_type(ValidatorConfig_Config_type_value(), t3);

    m := Memory(domain#Memory(m)[old_size+3 := true], contents#Memory(m)[old_size+3 := t3]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t5 := BorrowLoc(old_size+1);

    call t6 := ValidatorConfig_consensus_pubkey(t5);
    assume has_type(ByteArrayType(), t6);

    m := Memory(domain#Memory(m)[old_size+6 := true], contents#Memory(m)[old_size+6 := t6]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t8 := BorrowLoc(old_size+1);

    call t9 := ValidatorConfig_network_signing_pubkey(t8);
    assume has_type(ByteArrayType(), t9);

    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    call t10 := BorrowLoc(old_size+1);

    call t11 := ValidatorConfig_network_identity_pubkey(t10);
    assume has_type(ByteArrayType(), t11);

    m := Memory(domain#Memory(m)[old_size+11 := true], contents#Memory(m)[old_size+11 := t11]);

    assume has_type(AddressType(), contents#Memory(m)[old_size+4]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+6]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+7]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+9]);

    assume has_type(ByteArrayType(), contents#Memory(m)[old_size+11]);

    call tmp := Pack_LibraSystem_ValidatorInfo(contents#Memory(m)[old_size+4], contents#Memory(m)[old_size+6], contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+9], contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+12];
    return;

}

procedure LibraSystem_make_info_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraSystem_make_info(arg0);
}

procedure {:inline 1} LibraSystem_reconfigure () returns ()
{
    // declare local variables
    var t0: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t1: Reference; // ReferenceType(Vector_T_type_value(AddressType()))
    var t2: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t3: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t4: Reference; // ReferenceType(AddressType())
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // BooleanType()
    var t8: Value; // AddressType()
    var t9: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t10: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t11: Reference; // ReferenceType(Vector_T_type_value(AddressType()))
    var t12: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t13: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t14: Value; // IntegerType()
    var t15: Reference; // ReferenceType(Vector_T_type_value(AddressType()))
    var t16: Reference; // ReferenceType(Vector_T_type_value(AddressType()))
    var t17: Value; // IntegerType()
    var t18: Value; // IntegerType()
    var t19: Value; // IntegerType()
    var t20: Value; // BooleanType()
    var t21: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t22: Reference; // ReferenceType(Vector_T_type_value(AddressType()))
    var t23: Value; // AddressType()
    var t24: Value; // LibraSystem_ValidatorInfo_type_value()
    var t25: Value; // IntegerType()
    var t26: Value; // IntegerType()
    var t27: Value; // IntegerType()
    var t28: Value; // IntegerType()
    var t29: Value; // IntegerType()
    var t30: Value; // BooleanType()
    var t31: Value; // BooleanType()
    var t32: Value; // BooleanType()
    var t33: Value; // IntegerType()
    var t34: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t35: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t36: Value; // IntegerType()
    var t37: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t38: Value; // IntegerType()
    var t39: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t40: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t41: Value; // BooleanType()
    var t42: Value; // BooleanType()
    var t43: Value; // IntegerType()
    var t44: Value; // IntegerType()
    var t45: Value; // IntegerType()
    var t46: Value; // IntegerType()
    var t47: Value; // IntegerType()
    var t48: Value; // BooleanType()
    var t49: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t50: Value; // IntegerType()
    var t51: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t52: Value; // BooleanType()
    var t53: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t54: Reference; // ReferenceType(LibraAccount_EventHandle_type_value(LibraSystem_ValidatorSetChangeEvent_type_value()))
    var t55: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t56: Value; // Vector_T_type_value(LibraSystem_ValidatorInfo_type_value())
    var t57: Value; // LibraSystem_ValidatorSetChangeEvent_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 58;

    // bytecode translation starts here
    call tmp := LdAddr(472);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call t9 := BorrowGlobal(contents#Memory(m)[old_size+8], LibraSystem_ValidatorSet_type_value());

    call t0 := CopyOrMoveRef(t9);

    call t10 := CopyOrMoveRef(t0);

    call t11 := BorrowField(t10, LibraSystem_ValidatorSet_additions);

    call t1 := CopyOrMoveRef(t11);

    call t12 := CopyOrMoveRef(t0);

    call t13 := BorrowField(t12, LibraSystem_ValidatorSet_validators);

    call t2 := CopyOrMoveRef(t13);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+14]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t15 := CopyOrMoveRef(t1);

    call t16 := FreezeRef(t15);

    call t17 := Vector_length(AddressType(), t16);
    assume has_type(IntegerType(), t17);

    m := Memory(domain#Memory(m)[old_size+17 := true], contents#Memory(m)[old_size+17 := t17]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+17]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    call tmp := Gt(contents#Memory(m)[old_size+18], contents#Memory(m)[old_size+19]);
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 20];
    if (!b#Boolean(tmp)) { goto Label_37; }

Label_19:
    call t21 := CopyOrMoveRef(t2);

    call t22 := CopyOrMoveRef(t1);

    call t23 := Vector_pop_back(AddressType(), t22);
    assume has_type(AddressType(), t23);

    m := Memory(domain#Memory(m)[old_size+23 := true], contents#Memory(m)[old_size+23 := t23]);

    call t24 := LibraSystem_make_info(contents#Memory(m)[old_size+23]);
    assume has_type(LibraSystem_ValidatorInfo_type_value(), t24);

    m := Memory(domain#Memory(m)[old_size+24 := true], contents#Memory(m)[old_size+24 := t24]);

    call Vector_push_back(LibraSystem_ValidatorInfo_type_value(), t21, contents#Memory(m)[old_size+24]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[25+old_size := true], contents#Memory(m)[25+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[26+old_size := true], contents#Memory(m)[26+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+25], contents#Memory(m)[old_size+26]);
    m := Memory(domain#Memory(m)[27+old_size := true], contents#Memory(m)[27+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+27]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[28+old_size := true], contents#Memory(m)[28+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[29+old_size := true], contents#Memory(m)[29+old_size := tmp]);

    call tmp := Ge(contents#Memory(m)[old_size+28], contents#Memory(m)[old_size+29]);
    m := Memory(domain#Memory(m)[30+old_size := true], contents#Memory(m)[30+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 30];
    if (!b#Boolean(tmp)) { goto Label_33; }

    goto Label_34;

Label_33:
    goto Label_19;

Label_34:
    call tmp := LdTrue();
    m := Memory(domain#Memory(m)[31+old_size := true], contents#Memory(m)[31+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+31]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    goto Label_39;

Label_37:
    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[32+old_size := true], contents#Memory(m)[32+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+32]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

Label_39:
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[33+old_size := true], contents#Memory(m)[33+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+33]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t34 := CopyOrMoveRef(t2);

    call t35 := FreezeRef(t34);

    call t36 := Vector_length(LibraSystem_ValidatorInfo_type_value(), t35);
    assume has_type(IntegerType(), t36);

    m := Memory(domain#Memory(m)[old_size+36 := true], contents#Memory(m)[old_size+36 := t36]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+36]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call t37 := CopyOrMoveRef(t2);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[38+old_size := true], contents#Memory(m)[38+old_size := tmp]);

    call t39 := Vector_borrow_mut(LibraSystem_ValidatorInfo_type_value(), t37, contents#Memory(m)[old_size+38]);


    call t3 := CopyOrMoveRef(t39);

Label_49:
    call t40 := CopyOrMoveRef(t3);

    call t41 := LibraSystem_copy_validator_info(t40);
    assume has_type(BooleanType(), t41);

    m := Memory(domain#Memory(m)[old_size+41 := true], contents#Memory(m)[old_size+41 := t41]);

    tmp := contents#Memory(m)[old_size + 41];
    if (!b#Boolean(tmp)) { goto Label_54; }

    call tmp := LdTrue();
    m := Memory(domain#Memory(m)[42+old_size := true], contents#Memory(m)[42+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+42]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

Label_54:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[43+old_size := true], contents#Memory(m)[43+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[44+old_size := true], contents#Memory(m)[44+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+43], contents#Memory(m)[old_size+44]);
    m := Memory(domain#Memory(m)[45+old_size := true], contents#Memory(m)[45+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+45]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[46+old_size := true], contents#Memory(m)[46+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[47+old_size := true], contents#Memory(m)[47+old_size := tmp]);

    call tmp := Ge(contents#Memory(m)[old_size+46], contents#Memory(m)[old_size+47]);
    m := Memory(domain#Memory(m)[48+old_size := true], contents#Memory(m)[48+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 48];
    if (!b#Boolean(tmp)) { goto Label_63; }

    goto Label_68;

Label_63:
    call t49 := CopyOrMoveRef(t2);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[50+old_size := true], contents#Memory(m)[50+old_size := tmp]);

    call t51 := Vector_borrow_mut(LibraSystem_ValidatorInfo_type_value(), t49, contents#Memory(m)[old_size+50]);


    call t3 := CopyOrMoveRef(t51);

    goto Label_49;

Label_68:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[52+old_size := true], contents#Memory(m)[52+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 52];
    if (!b#Boolean(tmp)) { goto Label_77; }

    call t53 := CopyOrMoveRef(t0);

    call t54 := BorrowField(t53, LibraSystem_ValidatorSet_change_events);

    call t55 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t55);
    assume has_type(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()), tmp);

    m := Memory(domain#Memory(m)[56+old_size := true], contents#Memory(m)[56+old_size := tmp]);

    assume has_type(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()), contents#Memory(m)[old_size+56]);

    call tmp := Pack_LibraSystem_ValidatorSetChangeEvent(contents#Memory(m)[old_size+56]);
    m := Memory(domain#Memory(m)[57+old_size := true], contents#Memory(m)[57+old_size := tmp]);

    call LibraAccount_emit_event(LibraSystem_ValidatorSetChangeEvent_type_value(), t54, contents#Memory(m)[old_size+57]);

    return;

Label_77:
    return;

}

procedure LibraSystem_reconfigure_verify () returns ()
{
    call LibraSystem_reconfigure();
}

procedure {:inline 1} LibraSystem_get_ith_validator_info (arg0: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t2: Value; // AddressType()
    var t3: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t4: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t5: Value; // IntegerType()
    var t6: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t7: Value; // IntegerType()
    var t8: Value; // BooleanType()
    var t9: Value; // BooleanType()
    var t10: Value; // IntegerType()
    var t11: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t12: Value; // IntegerType()
    var t13: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t14: Value; // LibraSystem_ValidatorInfo_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);

    old_size := m_size;
    m_size := m_size + 15;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := LdAddr(472);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], LibraSystem_ValidatorSet_type_value());

    call t4 := BorrowField(t3, LibraSystem_ValidatorSet_validators);

    call t1 := CopyOrMoveRef(t4);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := CopyOrMoveRef(t1);

    call t7 := Vector_length(LibraSystem_ValidatorInfo_type_value(), t6);
    assume has_type(IntegerType(), t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    call tmp := Lt(contents#Memory(m)[old_size+5], contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 9];
    if (!b#Boolean(tmp)) { goto Label_12; }

    call tmp := LdConst(3);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    assert false;

Label_12:
    call t11 := CopyOrMoveRef(t1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call t13 := Vector_borrow(LibraSystem_ValidatorInfo_type_value(), t11, contents#Memory(m)[old_size+12]);


    call tmp := ReadRef(t13);
    assume has_type(LibraSystem_ValidatorInfo_type_value(), tmp);

    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+14];
    return;

}

procedure LibraSystem_get_ith_validator_info_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraSystem_get_ith_validator_info(arg0);
}

procedure {:inline 1} LibraSystem_get_ith_validator_address (arg0: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t3: Value; // AddressType()
    var t4: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t5: Value; // AddressType()
    var t6: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t7: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t8: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t9: Value; // IntegerType()
    var t10: Value; // IntegerType()
    var t11: Value; // IntegerType()
    var t12: Value; // BooleanType()
    var t13: Value; // BooleanType()
    var t14: Value; // IntegerType()
    var t15: Reference; // ReferenceType(LibraSystem_ValidatorSet_type_value())
    var t16: Reference; // ReferenceType(Vector_T_type_value(LibraSystem_ValidatorInfo_type_value()))
    var t17: Value; // IntegerType()
    var t18: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t19: Reference; // ReferenceType(LibraSystem_ValidatorInfo_type_value())
    var t20: Reference; // ReferenceType(AddressType())
    var t21: Value; // AddressType()
    var t22: Value; // AddressType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);

    old_size := m_size;
    m_size := m_size + 23;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := LdAddr(472);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := BorrowGlobal(contents#Memory(m)[old_size+5], LibraSystem_ValidatorSet_type_value());

    call t2 := CopyOrMoveRef(t6);

    call t7 := CopyOrMoveRef(t2);

    call t8 := BorrowField(t7, LibraSystem_ValidatorSet_validators);

    call t9 := Vector_length(LibraSystem_ValidatorInfo_type_value(), t8);
    assume has_type(IntegerType(), t9);

    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := Lt(contents#Memory(m)[old_size+10], contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+12]);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 13];
    if (!b#Boolean(tmp)) { goto Label_14; }

    call tmp := LdConst(3);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    assert false;

Label_14:
    call t15 := CopyOrMoveRef(t2);

    call t16 := BorrowField(t15, LibraSystem_ValidatorSet_validators);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    call t18 := Vector_borrow(LibraSystem_ValidatorInfo_type_value(), t16, contents#Memory(m)[old_size+17]);


    call t4 := CopyOrMoveRef(t18);

    call t19 := CopyOrMoveRef(t4);

    call t20 := BorrowField(t19, LibraSystem_ValidatorInfo_addr);

    call tmp := ReadRef(t20);
    assume has_type(AddressType(), tmp);

    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+21]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+22];
    return;

}

procedure LibraSystem_get_ith_validator_address_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraSystem_get_ith_validator_address(arg0);
}

procedure {:inline 1} TransactionFeeDistribution_distribute_transaction_fees () returns ()
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // LibraCoin_T_type_value()
    var t3: Value; // IntegerType()
    var t4: Reference; // ReferenceType(TransactionFeeDistribution_T_type_value())
    var t5: Value; // AddressType()
    var t6: Reference; // ReferenceType(TransactionFeeDistribution_T_type_value())
    var t7: Value; // IntegerType()
    var t8: Value; // AddressType()
    var t9: Value; // IntegerType()
    var t10: Reference; // ReferenceType(TransactionFeeDistribution_T_type_value())
    var t11: Reference; // ReferenceType(LibraAccount_WithdrawalCapability_type_value())
    var t12: Value; // IntegerType()
    var t13: Value; // LibraCoin_T_type_value()
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()
    var t16: Value; // IntegerType()
    var t17: Value; // LibraCoin_T_type_value()
    var t18: Value; // IntegerType()
    var t19: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 20;

    // bytecode translation starts here
    call tmp := LdAddr(4078);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := BorrowGlobal(contents#Memory(m)[old_size+5], TransactionFeeDistribution_T_type_value());

    call t4 := CopyOrMoveRef(t6);

    call t7 := LibraSystem_validator_set_size();
    assume has_type(IntegerType(), t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := LdAddr(4078);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call t9 := LibraAccount_balance(contents#Memory(m)[old_size+8]);
    assume has_type(IntegerType(), t9);

    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t10 := CopyOrMoveRef(t4);

    call t11 := BorrowField(t10, TransactionFeeDistribution_T_fee_withdrawal_capability);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call t13 := LibraAccount_withdraw_with_capability(t11, contents#Memory(m)[old_size+12]);
    assume has_type(LibraCoin_T_type_value(), t13);

    m := Memory(domain#Memory(m)[old_size+13 := true], contents#Memory(m)[old_size+13 := t13]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+13]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call t16 := TransactionFeeDistribution_per_validator_distribution_amount(contents#Memory(m)[old_size+14], contents#Memory(m)[old_size+15]);
    assume has_type(IntegerType(), t16);

    m := Memory(domain#Memory(m)[old_size+16 := true], contents#Memory(m)[old_size+16 := t16]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+16]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    call TransactionFeeDistribution_distribute_transaction_fees_internal(contents#Memory(m)[old_size+17], contents#Memory(m)[old_size+18], contents#Memory(m)[old_size+19]);

    return;

}

procedure TransactionFeeDistribution_distribute_transaction_fees_verify () returns ()
{
    call TransactionFeeDistribution_distribute_transaction_fees();
}

procedure {:inline 1} TransactionFeeDistribution_initialize () returns ()
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // AddressType()
    var t2: Value; // BooleanType()
    var t3: Value; // BooleanType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // LibraAccount_WithdrawalCapability_type_value()
    var t7: Value; // TransactionFeeDistribution_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 8;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := LdAddr(4078);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    tmp := Boolean(is_equal(AddressType(), contents#Memory(m)[old_size+0], contents#Memory(m)[old_size+1]));
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 3];
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    assert false;

Label_7:
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := LibraAccount_extract_sender_withdrawal_capability();
    assume has_type(LibraAccount_WithdrawalCapability_type_value(), t6);

    m := Memory(domain#Memory(m)[old_size+6 := true], contents#Memory(m)[old_size+6 := t6]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+5]);

    assume has_type(LibraAccount_WithdrawalCapability_type_value(), contents#Memory(m)[old_size+6]);

    call tmp := Pack_TransactionFeeDistribution_T(contents#Memory(m)[old_size+5], contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call MoveToSender(TransactionFeeDistribution_T_type_value(), contents#Memory(m)[old_size+7]);

    return;

}

procedure TransactionFeeDistribution_initialize_verify () returns ()
{
    call TransactionFeeDistribution_initialize();
}

procedure {:inline 1} TransactionFeeDistribution_distribute_transaction_fees_internal (arg0: Value, arg1: Value, arg2: Value) returns ()
{
    // declare local variables
    var t0: Value; // LibraCoin_T_type_value()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // AddressType()
    var t5: Value; // LibraCoin_T_type_value()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // BooleanType()
    var t10: Value; // IntegerType()
    var t11: Value; // AddressType()
    var t12: Value; // IntegerType()
    var t13: Value; // IntegerType()
    var t14: Value; // IntegerType()
    var t15: Value; // LibraCoin_T_type_value()
    var t16: Value; // IntegerType()
    var t17: Value; // LibraCoin_T_type_value()
    var t18: Value; // LibraCoin_T_type_value()
    var t19: Value; // AddressType()
    var t20: Value; // LibraCoin_T_type_value()
    var t21: Reference; // ReferenceType(LibraCoin_T_type_value())
    var t22: Value; // IntegerType()
    var t23: Value; // IntegerType()
    var t24: Value; // BooleanType()
    var t25: Value; // LibraCoin_T_type_value()
    var t26: Value; // AddressType()
    var t27: Value; // LibraCoin_T_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(LibraCoin_T_type_value(), arg0);
    assume has_type(IntegerType(), arg1);
    assume has_type(IntegerType(), arg2);

    old_size := m_size;
    m_size := m_size + 28;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size :=  arg2]);

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

Label_2:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := Lt(contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 9];
    if (!b#Boolean(tmp)) { goto Label_22; }

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call t11 := LibraSystem_get_ith_validator_address(contents#Memory(m)[old_size+10]);
    assume has_type(AddressType(), t11);

    m := Memory(domain#Memory(m)[old_size+11 := true], contents#Memory(m)[old_size+11 := t11]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+12], contents#Memory(m)[old_size+13]);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+14]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call t18, t17 := LibraCoin_split(contents#Memory(m)[old_size+15], contents#Memory(m)[old_size+16]);
    assume has_type(LibraCoin_T_type_value(), t18);

    assume has_type(LibraCoin_T_type_value(), t17);

    m := Memory(domain#Memory(m)[old_size+18 := true], contents#Memory(m)[old_size+18 := t18]);
    m := Memory(domain#Memory(m)[old_size+17 := true], contents#Memory(m)[old_size+17 := t17]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+18]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+17]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call LibraAccount_deposit(contents#Memory(m)[old_size+19], contents#Memory(m)[old_size+20]);

    goto Label_2;

Label_22:
    call t21 := BorrowLoc(old_size+0);

    call t22 := LibraCoin_value(t21);
    assume has_type(IntegerType(), t22);

    m := Memory(domain#Memory(m)[old_size+22 := true], contents#Memory(m)[old_size+22 := t22]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    tmp := Boolean(is_equal(IntegerType(), contents#Memory(m)[old_size+22], contents#Memory(m)[old_size+23]));
    m := Memory(domain#Memory(m)[24+old_size := true], contents#Memory(m)[24+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 24];
    if (!b#Boolean(tmp)) { goto Label_30; }

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[25+old_size := true], contents#Memory(m)[25+old_size := tmp]);

    call LibraCoin_destroy_zero(contents#Memory(m)[old_size+25]);

    goto Label_33;

Label_30:
    call tmp := LdAddr(4078);
    m := Memory(domain#Memory(m)[26+old_size := true], contents#Memory(m)[26+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[27+old_size := true], contents#Memory(m)[27+old_size := tmp]);

    call LibraAccount_deposit(contents#Memory(m)[old_size+26], contents#Memory(m)[old_size+27]);

Label_33:
    return;

}

procedure TransactionFeeDistribution_distribute_transaction_fees_internal_verify (arg0: Value, arg1: Value, arg2: Value) returns ()
{
    call TransactionFeeDistribution_distribute_transaction_fees_internal(arg0, arg1, arg2);
}

procedure {:inline 1} TransactionFeeDistribution_per_validator_distribution_amount (arg0: Value, arg1: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // BooleanType()
    var t6: Value; // BooleanType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // IntegerType()
    var t11: Value; // IntegerType()
    var t12: Value; // IntegerType()
    var t13: Value; // IntegerType()
    var t14: Value; // IntegerType()
    var t15: Value; // BooleanType()
    var t16: Value; // BooleanType()
    var t17: Value; // IntegerType()
    var t18: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);
    assume has_type(IntegerType(), arg1);

    old_size := m_size;
    m_size := m_size + 19;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    tmp := Boolean(!is_equal(IntegerType(), contents#Memory(m)[old_size+3], contents#Memory(m)[old_size+4]));
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 6];
    if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    assert false;

Label_7:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := Div(contents#Memory(m)[old_size+8], contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+10]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call tmp := Mul(contents#Memory(m)[old_size+11], contents#Memory(m)[old_size+12]);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := Le(contents#Memory(m)[old_size+13], contents#Memory(m)[old_size+14]);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 16];
    if (!b#Boolean(tmp)) { goto Label_20; }

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    assert false;

Label_20:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+18];
    return;

}

procedure TransactionFeeDistribution_per_validator_distribution_amount_verify (arg0: Value, arg1: Value) returns (ret0: Value)
{
    call ret0 := TransactionFeeDistribution_per_validator_distribution_amount(arg0, arg1);
}
