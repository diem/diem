

// everything below is auto generated

const unique LibraCoin_T: TypeName;
const unique LibraCoin_T_value: FieldName;

procedure {:inline 1} Pack_LibraCoin_T(v0: Value) returns (v: Value)
{
    assert is#Integer(v0);
    v := Map(DefaultMap[Field(LibraCoin_T_value) := v0]);
}

procedure {:inline 1} Unpack_LibraCoin_T(v: Value) returns (v0: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(LibraCoin_T_value)];
}

procedure {:inline 1} Eq_LibraCoin_T(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_int(m#Map(v1)[Field(LibraCoin_T_value)], m#Map(v2)[Field(LibraCoin_T_value)]);
    res := Boolean(true && b#Boolean(b0));
}

procedure {:inline 1} Neq_LibraCoin_T(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraCoin_T(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique LibraCoin_MintCapability: TypeName;

procedure {:inline 1} Pack_LibraCoin_MintCapability() returns (v: Value)
{
    v := Map(DefaultMap);
}

procedure {:inline 1} Unpack_LibraCoin_MintCapability(v: Value) returns ()
{
    assert is#Map(v);
}

procedure {:inline 1} Eq_LibraCoin_MintCapability(v1: Value, v2: Value) returns (res: Value)
{
    assert is#Map(v1) && is#Map(v2);
    res := Boolean(true);
}

procedure {:inline 1} Neq_LibraCoin_MintCapability(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraCoin_MintCapability(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique LibraCoin_MarketCap: TypeName;
const unique LibraCoin_MarketCap_total_value: FieldName;

procedure {:inline 1} Pack_LibraCoin_MarketCap(v0: Value) returns (v: Value)
{
    assert is#Integer(v0);
    v := Map(DefaultMap[Field(LibraCoin_MarketCap_total_value) := v0]);
}

procedure {:inline 1} Unpack_LibraCoin_MarketCap(v: Value) returns (v0: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(LibraCoin_MarketCap_total_value)];
}

procedure {:inline 1} Eq_LibraCoin_MarketCap(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_int(m#Map(v1)[Field(LibraCoin_MarketCap_total_value)], m#Map(v2)[Field(LibraCoin_MarketCap_total_value)]);
    res := Boolean(true && b#Boolean(b0));
}

procedure {:inline 1} Neq_LibraCoin_MarketCap(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraCoin_MarketCap(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique Vector_T: TypeName;
const unique ValidatorConfig_Config: TypeName;
const unique ValidatorConfig_Config_consensus_pubkey: FieldName;
const unique ValidatorConfig_Config_network_identity_pubkey: FieldName;
const unique ValidatorConfig_Config_network_signing_pubkey: FieldName;

procedure {:inline 1} Pack_ValidatorConfig_Config(v0: Value, v1: Value, v2: Value) returns (v: Value)
{
    assert is#ByteArray(v0);
    assert is#ByteArray(v1);
    assert is#ByteArray(v2);
    v := Map(DefaultMap[Field(ValidatorConfig_Config_consensus_pubkey) := v0][Field(ValidatorConfig_Config_network_identity_pubkey) := v1][Field(ValidatorConfig_Config_network_signing_pubkey) := v2]);
}

procedure {:inline 1} Unpack_ValidatorConfig_Config(v: Value) returns (v0: Value, v1: Value, v2: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(ValidatorConfig_Config_consensus_pubkey)];
    v1 := m#Map(v)[Field(ValidatorConfig_Config_network_identity_pubkey)];
    v2 := m#Map(v)[Field(ValidatorConfig_Config_network_signing_pubkey)];
}

procedure {:inline 1} Eq_ValidatorConfig_Config(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    var b2: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_bytearray(m#Map(v1)[Field(ValidatorConfig_Config_consensus_pubkey)], m#Map(v2)[Field(ValidatorConfig_Config_consensus_pubkey)]);
    call b1 := Eq_bytearray(m#Map(v1)[Field(ValidatorConfig_Config_network_identity_pubkey)], m#Map(v2)[Field(ValidatorConfig_Config_network_identity_pubkey)]);
    call b2 := Eq_bytearray(m#Map(v1)[Field(ValidatorConfig_Config_network_signing_pubkey)], m#Map(v2)[Field(ValidatorConfig_Config_network_signing_pubkey)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1) && b#Boolean(b2));
}

procedure {:inline 1} Neq_ValidatorConfig_Config(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_ValidatorConfig_Config(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique ValidatorConfig_T: TypeName;
const unique ValidatorConfig_T_config: FieldName;

procedure {:inline 1} Pack_ValidatorConfig_T(v0: Value) returns (v: Value)
{
    assert is#Map(v0);
    v := Map(DefaultMap[Field(ValidatorConfig_T_config) := v0]);
}

procedure {:inline 1} Unpack_ValidatorConfig_T(v: Value) returns (v0: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(ValidatorConfig_T_config)];
}

procedure {:inline 1} Eq_ValidatorConfig_T(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_ValidatorConfig_Config(m#Map(v1)[Field(ValidatorConfig_T_config)], m#Map(v2)[Field(ValidatorConfig_T_config)]);
    res := Boolean(true && b#Boolean(b0));
}

procedure {:inline 1} Neq_ValidatorConfig_T(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_ValidatorConfig_T(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique GasSchedule_Cost: TypeName;
const unique GasSchedule_Cost_cpu: FieldName;
const unique GasSchedule_Cost_storage: FieldName;

procedure {:inline 1} Pack_GasSchedule_Cost(v0: Value, v1: Value) returns (v: Value)
{
    assert is#Integer(v0);
    assert is#Integer(v1);
    v := Map(DefaultMap[Field(GasSchedule_Cost_cpu) := v0][Field(GasSchedule_Cost_storage) := v1]);
}

procedure {:inline 1} Unpack_GasSchedule_Cost(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(GasSchedule_Cost_cpu)];
    v1 := m#Map(v)[Field(GasSchedule_Cost_storage)];
}

procedure {:inline 1} Eq_GasSchedule_Cost(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_int(m#Map(v1)[Field(GasSchedule_Cost_cpu)], m#Map(v2)[Field(GasSchedule_Cost_cpu)]);
    call b1 := Eq_int(m#Map(v1)[Field(GasSchedule_Cost_storage)], m#Map(v2)[Field(GasSchedule_Cost_storage)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1));
}

procedure {:inline 1} Neq_GasSchedule_Cost(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_GasSchedule_Cost(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique GasSchedule_T: TypeName;
const unique GasSchedule_T_instruction_schedule: FieldName;
const unique GasSchedule_T_native_schedule: FieldName;

procedure {:inline 1} Pack_GasSchedule_T(v0: Value, v1: Value) returns (v: Value)
{
    assert is#Vector(v0);
    assert is#Vector(v1);
    v := Map(DefaultMap[Field(GasSchedule_T_instruction_schedule) := v0][Field(GasSchedule_T_native_schedule) := v1]);
}

procedure {:inline 1} Unpack_GasSchedule_T(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(GasSchedule_T_instruction_schedule)];
    v1 := m#Map(v)[Field(GasSchedule_T_native_schedule)];
}

procedure {:inline 1} Eq_GasSchedule_T(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_Vector_T(m#Map(v1)[Field(GasSchedule_T_instruction_schedule)], m#Map(v2)[Field(GasSchedule_T_instruction_schedule)]);
    call b1 := Eq_Vector_T(m#Map(v1)[Field(GasSchedule_T_native_schedule)], m#Map(v2)[Field(GasSchedule_T_native_schedule)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1));
}

procedure {:inline 1} Neq_GasSchedule_T(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_GasSchedule_T(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique LibraAccount_T: TypeName;
const unique LibraAccount_T_authentication_key: FieldName;
const unique LibraAccount_T_balance: FieldName;
const unique LibraAccount_T_delegated_key_rotation_capability: FieldName;
const unique LibraAccount_T_delegated_withdrawal_capability: FieldName;
const unique LibraAccount_T_event_generator: FieldName;
const unique LibraAccount_T_received_events: FieldName;
const unique LibraAccount_T_sent_events: FieldName;
const unique LibraAccount_T_sequence_number: FieldName;

procedure {:inline 1} Pack_LibraAccount_T(v0: Value, v1: Value, v2: Value, v3: Value, v4: Value, v5: Value, v6: Value, v7: Value) returns (v: Value)
{
    assert is#ByteArray(v0);
    assert is#Map(v1);
    assert is#Boolean(v2);
    assert is#Boolean(v3);
    assert is#Map(v4);
    assert is#Map(v5);
    assert is#Map(v6);
    assert is#Integer(v7);
    v := Map(DefaultMap[Field(LibraAccount_T_authentication_key) := v0][Field(LibraAccount_T_balance) := v1][Field(LibraAccount_T_delegated_key_rotation_capability) := v2][Field(LibraAccount_T_delegated_withdrawal_capability) := v3][Field(LibraAccount_T_event_generator) := v4][Field(LibraAccount_T_received_events) := v5][Field(LibraAccount_T_sent_events) := v6][Field(LibraAccount_T_sequence_number) := v7]);
}

procedure {:inline 1} Unpack_LibraAccount_T(v: Value) returns (v0: Value, v1: Value, v2: Value, v3: Value, v4: Value, v5: Value, v6: Value, v7: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(LibraAccount_T_authentication_key)];
    v1 := m#Map(v)[Field(LibraAccount_T_balance)];
    v2 := m#Map(v)[Field(LibraAccount_T_delegated_key_rotation_capability)];
    v3 := m#Map(v)[Field(LibraAccount_T_delegated_withdrawal_capability)];
    v4 := m#Map(v)[Field(LibraAccount_T_event_generator)];
    v5 := m#Map(v)[Field(LibraAccount_T_received_events)];
    v6 := m#Map(v)[Field(LibraAccount_T_sent_events)];
    v7 := m#Map(v)[Field(LibraAccount_T_sequence_number)];
}

procedure {:inline 1} Eq_LibraAccount_T(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    var b2: Value;
    var b3: Value;
    var b4: Value;
    var b5: Value;
    var b6: Value;
    var b7: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_bytearray(m#Map(v1)[Field(LibraAccount_T_authentication_key)], m#Map(v2)[Field(LibraAccount_T_authentication_key)]);
    call b1 := Eq_LibraCoin_T(m#Map(v1)[Field(LibraAccount_T_balance)], m#Map(v2)[Field(LibraAccount_T_balance)]);
    call b2 := Eq_bool(m#Map(v1)[Field(LibraAccount_T_delegated_key_rotation_capability)], m#Map(v2)[Field(LibraAccount_T_delegated_key_rotation_capability)]);
    call b3 := Eq_bool(m#Map(v1)[Field(LibraAccount_T_delegated_withdrawal_capability)], m#Map(v2)[Field(LibraAccount_T_delegated_withdrawal_capability)]);
    call b4 := Eq_LibraAccount_EventHandleGenerator(m#Map(v1)[Field(LibraAccount_T_event_generator)], m#Map(v2)[Field(LibraAccount_T_event_generator)]);
    call b5 := Eq_LibraAccount_EventHandle(m#Map(v1)[Field(LibraAccount_T_received_events)], m#Map(v2)[Field(LibraAccount_T_received_events)]);
    call b6 := Eq_LibraAccount_EventHandle(m#Map(v1)[Field(LibraAccount_T_sent_events)], m#Map(v2)[Field(LibraAccount_T_sent_events)]);
    call b7 := Eq_int(m#Map(v1)[Field(LibraAccount_T_sequence_number)], m#Map(v2)[Field(LibraAccount_T_sequence_number)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1) && b#Boolean(b2) && b#Boolean(b3) && b#Boolean(b4) && b#Boolean(b5) && b#Boolean(b6) && b#Boolean(b7));
}

procedure {:inline 1} Neq_LibraAccount_T(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraAccount_T(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique LibraAccount_WithdrawalCapability: TypeName;
const unique LibraAccount_WithdrawalCapability_account_address: FieldName;

procedure {:inline 1} Pack_LibraAccount_WithdrawalCapability(v0: Value) returns (v: Value)
{
    assert is#Address(v0);
    v := Map(DefaultMap[Field(LibraAccount_WithdrawalCapability_account_address) := v0]);
}

procedure {:inline 1} Unpack_LibraAccount_WithdrawalCapability(v: Value) returns (v0: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(LibraAccount_WithdrawalCapability_account_address)];
}

procedure {:inline 1} Eq_LibraAccount_WithdrawalCapability(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_address(m#Map(v1)[Field(LibraAccount_WithdrawalCapability_account_address)], m#Map(v2)[Field(LibraAccount_WithdrawalCapability_account_address)]);
    res := Boolean(true && b#Boolean(b0));
}

procedure {:inline 1} Neq_LibraAccount_WithdrawalCapability(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraAccount_WithdrawalCapability(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique LibraAccount_KeyRotationCapability: TypeName;
const unique LibraAccount_KeyRotationCapability_account_address: FieldName;

procedure {:inline 1} Pack_LibraAccount_KeyRotationCapability(v0: Value) returns (v: Value)
{
    assert is#Address(v0);
    v := Map(DefaultMap[Field(LibraAccount_KeyRotationCapability_account_address) := v0]);
}

procedure {:inline 1} Unpack_LibraAccount_KeyRotationCapability(v: Value) returns (v0: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(LibraAccount_KeyRotationCapability_account_address)];
}

procedure {:inline 1} Eq_LibraAccount_KeyRotationCapability(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_address(m#Map(v1)[Field(LibraAccount_KeyRotationCapability_account_address)], m#Map(v2)[Field(LibraAccount_KeyRotationCapability_account_address)]);
    res := Boolean(true && b#Boolean(b0));
}

procedure {:inline 1} Neq_LibraAccount_KeyRotationCapability(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraAccount_KeyRotationCapability(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique LibraAccount_SentPaymentEvent: TypeName;
const unique LibraAccount_SentPaymentEvent_amount: FieldName;
const unique LibraAccount_SentPaymentEvent_metadata: FieldName;
const unique LibraAccount_SentPaymentEvent_payee: FieldName;

procedure {:inline 1} Pack_LibraAccount_SentPaymentEvent(v0: Value, v1: Value, v2: Value) returns (v: Value)
{
    assert is#Integer(v0);
    assert is#ByteArray(v1);
    assert is#Address(v2);
    v := Map(DefaultMap[Field(LibraAccount_SentPaymentEvent_amount) := v0][Field(LibraAccount_SentPaymentEvent_metadata) := v1][Field(LibraAccount_SentPaymentEvent_payee) := v2]);
}

procedure {:inline 1} Unpack_LibraAccount_SentPaymentEvent(v: Value) returns (v0: Value, v1: Value, v2: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(LibraAccount_SentPaymentEvent_amount)];
    v1 := m#Map(v)[Field(LibraAccount_SentPaymentEvent_metadata)];
    v2 := m#Map(v)[Field(LibraAccount_SentPaymentEvent_payee)];
}

procedure {:inline 1} Eq_LibraAccount_SentPaymentEvent(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    var b2: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_int(m#Map(v1)[Field(LibraAccount_SentPaymentEvent_amount)], m#Map(v2)[Field(LibraAccount_SentPaymentEvent_amount)]);
    call b1 := Eq_bytearray(m#Map(v1)[Field(LibraAccount_SentPaymentEvent_metadata)], m#Map(v2)[Field(LibraAccount_SentPaymentEvent_metadata)]);
    call b2 := Eq_address(m#Map(v1)[Field(LibraAccount_SentPaymentEvent_payee)], m#Map(v2)[Field(LibraAccount_SentPaymentEvent_payee)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1) && b#Boolean(b2));
}

procedure {:inline 1} Neq_LibraAccount_SentPaymentEvent(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraAccount_SentPaymentEvent(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique LibraAccount_ReceivedPaymentEvent: TypeName;
const unique LibraAccount_ReceivedPaymentEvent_amount: FieldName;
const unique LibraAccount_ReceivedPaymentEvent_metadata: FieldName;
const unique LibraAccount_ReceivedPaymentEvent_payer: FieldName;

procedure {:inline 1} Pack_LibraAccount_ReceivedPaymentEvent(v0: Value, v1: Value, v2: Value) returns (v: Value)
{
    assert is#Integer(v0);
    assert is#ByteArray(v1);
    assert is#Address(v2);
    v := Map(DefaultMap[Field(LibraAccount_ReceivedPaymentEvent_amount) := v0][Field(LibraAccount_ReceivedPaymentEvent_metadata) := v1][Field(LibraAccount_ReceivedPaymentEvent_payer) := v2]);
}

procedure {:inline 1} Unpack_LibraAccount_ReceivedPaymentEvent(v: Value) returns (v0: Value, v1: Value, v2: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(LibraAccount_ReceivedPaymentEvent_amount)];
    v1 := m#Map(v)[Field(LibraAccount_ReceivedPaymentEvent_metadata)];
    v2 := m#Map(v)[Field(LibraAccount_ReceivedPaymentEvent_payer)];
}

procedure {:inline 1} Eq_LibraAccount_ReceivedPaymentEvent(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    var b2: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_int(m#Map(v1)[Field(LibraAccount_ReceivedPaymentEvent_amount)], m#Map(v2)[Field(LibraAccount_ReceivedPaymentEvent_amount)]);
    call b1 := Eq_bytearray(m#Map(v1)[Field(LibraAccount_ReceivedPaymentEvent_metadata)], m#Map(v2)[Field(LibraAccount_ReceivedPaymentEvent_metadata)]);
    call b2 := Eq_address(m#Map(v1)[Field(LibraAccount_ReceivedPaymentEvent_payer)], m#Map(v2)[Field(LibraAccount_ReceivedPaymentEvent_payer)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1) && b#Boolean(b2));
}

procedure {:inline 1} Neq_LibraAccount_ReceivedPaymentEvent(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraAccount_ReceivedPaymentEvent(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique LibraAccount_EventHandleGenerator: TypeName;
const unique LibraAccount_EventHandleGenerator_counter: FieldName;

procedure {:inline 1} Pack_LibraAccount_EventHandleGenerator(v0: Value) returns (v: Value)
{
    assert is#Integer(v0);
    v := Map(DefaultMap[Field(LibraAccount_EventHandleGenerator_counter) := v0]);
}

procedure {:inline 1} Unpack_LibraAccount_EventHandleGenerator(v: Value) returns (v0: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(LibraAccount_EventHandleGenerator_counter)];
}

procedure {:inline 1} Eq_LibraAccount_EventHandleGenerator(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_int(m#Map(v1)[Field(LibraAccount_EventHandleGenerator_counter)], m#Map(v2)[Field(LibraAccount_EventHandleGenerator_counter)]);
    res := Boolean(true && b#Boolean(b0));
}

procedure {:inline 1} Neq_LibraAccount_EventHandleGenerator(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraAccount_EventHandleGenerator(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique LibraAccount_EventHandle: TypeName;
const unique LibraAccount_EventHandle_counter: FieldName;
const unique LibraAccount_EventHandle_guid: FieldName;

procedure {:inline 1} Pack_LibraAccount_EventHandle(v0: Value, v1: Value) returns (v: Value)
{
    assert is#Integer(v0);
    assert is#ByteArray(v1);
    v := Map(DefaultMap[Field(LibraAccount_EventHandle_counter) := v0][Field(LibraAccount_EventHandle_guid) := v1]);
}

procedure {:inline 1} Unpack_LibraAccount_EventHandle(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(LibraAccount_EventHandle_counter)];
    v1 := m#Map(v)[Field(LibraAccount_EventHandle_guid)];
}

procedure {:inline 1} Eq_LibraAccount_EventHandle(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_int(m#Map(v1)[Field(LibraAccount_EventHandle_counter)], m#Map(v2)[Field(LibraAccount_EventHandle_counter)]);
    call b1 := Eq_bytearray(m#Map(v1)[Field(LibraAccount_EventHandle_guid)], m#Map(v2)[Field(LibraAccount_EventHandle_guid)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1));
}

procedure {:inline 1} Neq_LibraAccount_EventHandle(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraAccount_EventHandle(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique LibraSystem_ValidatorInfo: TypeName;
const unique LibraSystem_ValidatorInfo_addr: FieldName;
const unique LibraSystem_ValidatorInfo_consensus_pubkey: FieldName;
const unique LibraSystem_ValidatorInfo_consensus_voting_power: FieldName;
const unique LibraSystem_ValidatorInfo_network_identity_pubkey: FieldName;
const unique LibraSystem_ValidatorInfo_network_signing_pubkey: FieldName;

procedure {:inline 1} Pack_LibraSystem_ValidatorInfo(v0: Value, v1: Value, v2: Value, v3: Value, v4: Value) returns (v: Value)
{
    assert is#Address(v0);
    assert is#ByteArray(v1);
    assert is#Integer(v2);
    assert is#ByteArray(v3);
    assert is#ByteArray(v4);
    v := Map(DefaultMap[Field(LibraSystem_ValidatorInfo_addr) := v0][Field(LibraSystem_ValidatorInfo_consensus_pubkey) := v1][Field(LibraSystem_ValidatorInfo_consensus_voting_power) := v2][Field(LibraSystem_ValidatorInfo_network_identity_pubkey) := v3][Field(LibraSystem_ValidatorInfo_network_signing_pubkey) := v4]);
}

procedure {:inline 1} Unpack_LibraSystem_ValidatorInfo(v: Value) returns (v0: Value, v1: Value, v2: Value, v3: Value, v4: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(LibraSystem_ValidatorInfo_addr)];
    v1 := m#Map(v)[Field(LibraSystem_ValidatorInfo_consensus_pubkey)];
    v2 := m#Map(v)[Field(LibraSystem_ValidatorInfo_consensus_voting_power)];
    v3 := m#Map(v)[Field(LibraSystem_ValidatorInfo_network_identity_pubkey)];
    v4 := m#Map(v)[Field(LibraSystem_ValidatorInfo_network_signing_pubkey)];
}

procedure {:inline 1} Eq_LibraSystem_ValidatorInfo(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    var b2: Value;
    var b3: Value;
    var b4: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_address(m#Map(v1)[Field(LibraSystem_ValidatorInfo_addr)], m#Map(v2)[Field(LibraSystem_ValidatorInfo_addr)]);
    call b1 := Eq_bytearray(m#Map(v1)[Field(LibraSystem_ValidatorInfo_consensus_pubkey)], m#Map(v2)[Field(LibraSystem_ValidatorInfo_consensus_pubkey)]);
    call b2 := Eq_int(m#Map(v1)[Field(LibraSystem_ValidatorInfo_consensus_voting_power)], m#Map(v2)[Field(LibraSystem_ValidatorInfo_consensus_voting_power)]);
    call b3 := Eq_bytearray(m#Map(v1)[Field(LibraSystem_ValidatorInfo_network_identity_pubkey)], m#Map(v2)[Field(LibraSystem_ValidatorInfo_network_identity_pubkey)]);
    call b4 := Eq_bytearray(m#Map(v1)[Field(LibraSystem_ValidatorInfo_network_signing_pubkey)], m#Map(v2)[Field(LibraSystem_ValidatorInfo_network_signing_pubkey)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1) && b#Boolean(b2) && b#Boolean(b3) && b#Boolean(b4));
}

procedure {:inline 1} Neq_LibraSystem_ValidatorInfo(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraSystem_ValidatorInfo(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique LibraSystem_ValidatorSetChangeEvent: TypeName;
const unique LibraSystem_ValidatorSetChangeEvent_new_validator_set: FieldName;

procedure {:inline 1} Pack_LibraSystem_ValidatorSetChangeEvent(v0: Value) returns (v: Value)
{
    assert is#Vector(v0);
    v := Map(DefaultMap[Field(LibraSystem_ValidatorSetChangeEvent_new_validator_set) := v0]);
}

procedure {:inline 1} Unpack_LibraSystem_ValidatorSetChangeEvent(v: Value) returns (v0: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(LibraSystem_ValidatorSetChangeEvent_new_validator_set)];
}

procedure {:inline 1} Eq_LibraSystem_ValidatorSetChangeEvent(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_Vector_T(m#Map(v1)[Field(LibraSystem_ValidatorSetChangeEvent_new_validator_set)], m#Map(v2)[Field(LibraSystem_ValidatorSetChangeEvent_new_validator_set)]);
    res := Boolean(true && b#Boolean(b0));
}

procedure {:inline 1} Neq_LibraSystem_ValidatorSetChangeEvent(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraSystem_ValidatorSetChangeEvent(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique LibraSystem_ValidatorSet: TypeName;
const unique LibraSystem_ValidatorSet_change_events: FieldName;
const unique LibraSystem_ValidatorSet_validators: FieldName;

procedure {:inline 1} Pack_LibraSystem_ValidatorSet(v0: Value, v1: Value) returns (v: Value)
{
    assert is#Map(v0);
    assert is#Vector(v1);
    v := Map(DefaultMap[Field(LibraSystem_ValidatorSet_change_events) := v0][Field(LibraSystem_ValidatorSet_validators) := v1]);
}

procedure {:inline 1} Unpack_LibraSystem_ValidatorSet(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(LibraSystem_ValidatorSet_change_events)];
    v1 := m#Map(v)[Field(LibraSystem_ValidatorSet_validators)];
}

procedure {:inline 1} Eq_LibraSystem_ValidatorSet(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_LibraAccount_EventHandle(m#Map(v1)[Field(LibraSystem_ValidatorSet_change_events)], m#Map(v2)[Field(LibraSystem_ValidatorSet_change_events)]);
    call b1 := Eq_Vector_T(m#Map(v1)[Field(LibraSystem_ValidatorSet_validators)], m#Map(v2)[Field(LibraSystem_ValidatorSet_validators)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1));
}

procedure {:inline 1} Neq_LibraSystem_ValidatorSet(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraSystem_ValidatorSet(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique LibraSystem_BlockMetadata: TypeName;
const unique LibraSystem_BlockMetadata_height: FieldName;
const unique LibraSystem_BlockMetadata_id: FieldName;
const unique LibraSystem_BlockMetadata_proposer: FieldName;
const unique LibraSystem_BlockMetadata_timestamp: FieldName;

procedure {:inline 1} Pack_LibraSystem_BlockMetadata(v0: Value, v1: Value, v2: Value, v3: Value) returns (v: Value)
{
    assert is#Integer(v0);
    assert is#ByteArray(v1);
    assert is#Address(v2);
    assert is#Integer(v3);
    v := Map(DefaultMap[Field(LibraSystem_BlockMetadata_height) := v0][Field(LibraSystem_BlockMetadata_id) := v1][Field(LibraSystem_BlockMetadata_proposer) := v2][Field(LibraSystem_BlockMetadata_timestamp) := v3]);
}

procedure {:inline 1} Unpack_LibraSystem_BlockMetadata(v: Value) returns (v0: Value, v1: Value, v2: Value, v3: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(LibraSystem_BlockMetadata_height)];
    v1 := m#Map(v)[Field(LibraSystem_BlockMetadata_id)];
    v2 := m#Map(v)[Field(LibraSystem_BlockMetadata_proposer)];
    v3 := m#Map(v)[Field(LibraSystem_BlockMetadata_timestamp)];
}

procedure {:inline 1} Eq_LibraSystem_BlockMetadata(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    var b2: Value;
    var b3: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_int(m#Map(v1)[Field(LibraSystem_BlockMetadata_height)], m#Map(v2)[Field(LibraSystem_BlockMetadata_height)]);
    call b1 := Eq_bytearray(m#Map(v1)[Field(LibraSystem_BlockMetadata_id)], m#Map(v2)[Field(LibraSystem_BlockMetadata_id)]);
    call b2 := Eq_address(m#Map(v1)[Field(LibraSystem_BlockMetadata_proposer)], m#Map(v2)[Field(LibraSystem_BlockMetadata_proposer)]);
    call b3 := Eq_int(m#Map(v1)[Field(LibraSystem_BlockMetadata_timestamp)], m#Map(v2)[Field(LibraSystem_BlockMetadata_timestamp)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1) && b#Boolean(b2) && b#Boolean(b3));
}

procedure {:inline 1} Neq_LibraSystem_BlockMetadata(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_LibraSystem_BlockMetadata(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique TransactionFeeDistribution_T: TypeName;
const unique TransactionFeeDistribution_T_fee_withdrawal_capability: FieldName;
const unique TransactionFeeDistribution_T_last_epoch_paid: FieldName;

procedure {:inline 1} Pack_TransactionFeeDistribution_T(v0: Value, v1: Value) returns (v: Value)
{
    assert is#Map(v0);
    assert is#Integer(v1);
    v := Map(DefaultMap[Field(TransactionFeeDistribution_T_fee_withdrawal_capability) := v0][Field(TransactionFeeDistribution_T_last_epoch_paid) := v1]);
}

procedure {:inline 1} Unpack_TransactionFeeDistribution_T(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(TransactionFeeDistribution_T_fee_withdrawal_capability)];
    v1 := m#Map(v)[Field(TransactionFeeDistribution_T_last_epoch_paid)];
}

procedure {:inline 1} Eq_TransactionFeeDistribution_T(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_LibraAccount_WithdrawalCapability(m#Map(v1)[Field(TransactionFeeDistribution_T_fee_withdrawal_capability)], m#Map(v2)[Field(TransactionFeeDistribution_T_fee_withdrawal_capability)]);
    call b1 := Eq_int(m#Map(v1)[Field(TransactionFeeDistribution_T_last_epoch_paid)], m#Map(v2)[Field(TransactionFeeDistribution_T_last_epoch_paid)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1));
}

procedure {:inline 1} Neq_TransactionFeeDistribution_T(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_TransactionFeeDistribution_T(v1, v2);
    res := Boolean(!b#Boolean(res_val));
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
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
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
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
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
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
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
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := UpdateValue0(p, i+1, v', new_v);
        if (is#Map(v)) { v' := Map(m#Map(v)[e := v']);}
        if (is#Vector(v)) { v' := Vector(v#Vector(v)[e := v'], l#Vector(v));}
    }
}

procedure {:inline 1} UpdateValue2(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := UpdateValue1(p, i+1, v', new_v);
        if (is#Map(v)) { v' := Map(m#Map(v)[e := v']);}
        if (is#Vector(v)) { v' := Vector(v#Vector(v)[e := v'], l#Vector(v));}
    }
}

procedure {:inline 1} UpdateValueMax(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := UpdateValue2(p, i+1, v', new_v);
        if (is#Map(v)) { v' := Map(m#Map(v)[e := v']);}
        if (is#Vector(v)) { v' := Vector(v#Vector(v)[e := v'], l#Vector(v));}
    }
}

procedure {:inline 1} AddressUtil_address_to_bytes (arg0: Value) returns (ret0: Value);
procedure {:inline 1} BytearrayUtil_bytearray_concat (arg0: Value, arg1: Value) returns (ret0: Value);
procedure {:inline 1} LibraCoin_mint_with_default_capability (arg0: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // int
    var t1: Value; // int
    var t2: Value; // address
    var t3: Reference; // LibraCoin_MintCapability_ref
    var t4: Value; // LibraCoin_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Integer(arg0);

    old_size := m_size;
    m_size := m_size + 5;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], LibraCoin_MintCapability);

    call t4 := LibraCoin_mint(contents#Memory(m)[old_size+1], t3);
    assume is#Map(t4);

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
    var t0: Value; // int
    var t1: Reference; // LibraCoin_MintCapability_ref
    var t2: Reference; // LibraCoin_MarketCap_ref
    var t3: Value; // int
    var t4: Reference; // LibraCoin_MintCapability_ref
    var t5: Value; // int
    var t6: Value; // int
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // bool
    var t10: Value; // bool
    var t11: Value; // int
    var t12: Value; // address
    var t13: Reference; // LibraCoin_MarketCap_ref
    var t14: Reference; // LibraCoin_MarketCap_ref
    var t15: Reference; // int_ref
    var t16: Value; // int
    var t17: Value; // int
    var t18: Value; // int
    var t19: Value; // int
    var t20: Reference; // LibraCoin_MarketCap_ref
    var t21: Reference; // int_ref
    var t22: Value; // int
    var t23: Value; // LibraCoin_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Integer(arg0);

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

    call t13 := BorrowGlobal(contents#Memory(m)[old_size+12], LibraCoin_MarketCap);

    call t2 := CopyOrMoveRef(t13);

    call t14 := CopyOrMoveRef(t2);

    call t15 := BorrowField(t14, LibraCoin_MarketCap_total_value);

    call tmp := ReadRef(t15);
    assume is#Integer(tmp);

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

    assume is#Integer(contents#Memory(m)[old_size+22]);

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
    var t0: Value; // address
    var t1: Value; // address
    var t2: Value; // bool
    var t3: Value; // bool
    var t4: Value; // int
    var t5: Value; // LibraCoin_MintCapability
    var t6: Value; // int
    var t7: Value; // LibraCoin_MarketCap

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

    call tmp := Eq(contents#Memory(m)[old_size+0], contents#Memory(m)[old_size+1]);
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

    call MoveToSender(LibraCoin_MintCapability, contents#Memory(m)[old_size+5]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+6]);

    call tmp := Pack_LibraCoin_MarketCap(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call MoveToSender(LibraCoin_MarketCap, contents#Memory(m)[old_size+7]);

    return;

}

procedure LibraCoin_initialize_verify () returns ()
{
    call LibraCoin_initialize();
}

procedure {:inline 1} LibraCoin_market_cap () returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // address
    var t1: Reference; // LibraCoin_MarketCap_ref
    var t2: Reference; // int_ref
    var t3: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call t1 := BorrowGlobal(contents#Memory(m)[old_size+0], LibraCoin_MarketCap);

    call t2 := BorrowField(t1, LibraCoin_MarketCap_total_value);

    call tmp := ReadRef(t2);
    assume is#Integer(tmp);

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
    var t0: Value; // int
    var t1: Value; // LibraCoin_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 2;

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+0]);

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
    var t0: Reference; // LibraCoin_T_ref
    var t1: Reference; // LibraCoin_T_ref
    var t2: Reference; // int_ref
    var t3: Value; // int

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
    assume is#Integer(tmp);

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
    var t0: Value; // LibraCoin_T
    var t1: Value; // int
    var t2: Value; // LibraCoin_T
    var t3: Reference; // LibraCoin_T_ref
    var t4: Value; // int
    var t5: Value; // LibraCoin_T
    var t6: Value; // LibraCoin_T
    var t7: Value; // LibraCoin_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Map(arg0);
    assume is#Integer(arg1);

    old_size := m_size;
    m_size := m_size + 8;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t3 := BorrowLoc(old_size+0);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t5 := LibraCoin_withdraw(t3, contents#Memory(m)[old_size+4]);
    assume is#Map(t5);

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
    var t0: Reference; // LibraCoin_T_ref
    var t1: Value; // int
    var t2: Value; // int
    var t3: Reference; // LibraCoin_T_ref
    var t4: Reference; // int_ref
    var t5: Value; // int
    var t6: Value; // int
    var t7: Value; // int
    var t8: Value; // bool
    var t9: Value; // bool
    var t10: Value; // int
    var t11: Value; // int
    var t12: Value; // int
    var t13: Value; // int
    var t14: Reference; // LibraCoin_T_ref
    var t15: Reference; // int_ref
    var t16: Value; // int
    var t17: Value; // LibraCoin_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
        assume is#Integer(arg1);

    old_size := m_size;
    m_size := m_size + 18;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(t0);

    call t4 := BorrowField(t3, LibraCoin_T_value);

    call tmp := ReadRef(t4);
    assume is#Integer(tmp);

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

    assume is#Integer(contents#Memory(m)[old_size+16]);

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
    var t0: Value; // LibraCoin_T
    var t1: Value; // LibraCoin_T
    var t2: Reference; // LibraCoin_T_ref
    var t3: Value; // LibraCoin_T
    var t4: Value; // LibraCoin_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Map(arg0);
    assume is#Map(arg1);

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
    var t0: Reference; // LibraCoin_T_ref
    var t1: Value; // LibraCoin_T
    var t2: Value; // int
    var t3: Value; // int
    var t4: Reference; // LibraCoin_T_ref
    var t5: Reference; // int_ref
    var t6: Value; // int
    var t7: Value; // LibraCoin_T
    var t8: Value; // int
    var t9: Value; // int
    var t10: Value; // int
    var t11: Value; // int
    var t12: Reference; // LibraCoin_T_ref
    var t13: Reference; // int_ref

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
        assume is#Map(arg1);

    old_size := m_size;
    m_size := m_size + 14;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, LibraCoin_T_value);

    call tmp := ReadRef(t5);
    assume is#Integer(tmp);

    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t8 := Unpack_LibraCoin_T(contents#Memory(m)[old_size+7]);
    assume is#Integer(t8);

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
    var t0: Value; // LibraCoin_T
    var t1: Value; // int
    var t2: Value; // LibraCoin_T
    var t3: Value; // int
    var t4: Value; // int
    var t5: Value; // int
    var t6: Value; // bool
    var t7: Value; // bool
    var t8: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Map(arg0);

    old_size := m_size;
    m_size := m_size + 9;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := Unpack_LibraCoin_T(contents#Memory(m)[old_size+2]);
    assume is#Integer(t3);

    m := Memory(domain#Memory(m)[old_size+3 := true], contents#Memory(m)[old_size+3 := t3]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+4], contents#Memory(m)[old_size+5]);
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
    var t0: Value; // address
    var t1: Value; // address
    var t2: Value; // bool

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 3;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := Exists(contents#Memory(m)[old_size+1], ValidatorConfig_T);
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
    var t0: Value; // address
    var t1: Reference; // ValidatorConfig_T_ref
    var t2: Value; // address
    var t3: Reference; // ValidatorConfig_T_ref
    var t4: Reference; // ValidatorConfig_T_ref
    var t5: Reference; // ValidatorConfig_Config_ref
    var t6: Value; // ValidatorConfig_Config

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 7;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], ValidatorConfig_T);

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, ValidatorConfig_T_config);

    call tmp := ReadRef(t5);
    assume is#Map(tmp);

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
    var t0: Reference; // ValidatorConfig_Config_ref
    var t1: Reference; // ValidatorConfig_Config_ref
    var t2: Reference; // bytearray_ref
    var t3: Value; // bytearray

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
    assume is#ByteArray(tmp);

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
    var t0: Reference; // ValidatorConfig_Config_ref
    var t1: Reference; // ValidatorConfig_Config_ref
    var t2: Reference; // bytearray_ref
    var t3: Value; // bytearray

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
    assume is#ByteArray(tmp);

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
    var t0: Reference; // ValidatorConfig_Config_ref
    var t1: Reference; // ValidatorConfig_Config_ref
    var t2: Reference; // bytearray_ref
    var t3: Value; // bytearray

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
    assume is#ByteArray(tmp);

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
    var t0: Value; // bytearray
    var t1: Value; // bytearray
    var t2: Value; // bytearray
    var t3: Value; // bytearray
    var t4: Value; // bytearray
    var t5: Value; // bytearray
    var t6: Value; // ValidatorConfig_Config
    var t7: Value; // ValidatorConfig_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#ByteArray(arg0);
    assume is#ByteArray(arg1);
    assume is#ByteArray(arg2);

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

    assume is#ByteArray(contents#Memory(m)[old_size+3]);

    assume is#ByteArray(contents#Memory(m)[old_size+4]);

    assume is#ByteArray(contents#Memory(m)[old_size+5]);

    call tmp := Pack_ValidatorConfig_Config(contents#Memory(m)[old_size+3], contents#Memory(m)[old_size+4], contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    assume is#Map(contents#Memory(m)[old_size+6]);

    call tmp := Pack_ValidatorConfig_T(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call MoveToSender(ValidatorConfig_T, contents#Memory(m)[old_size+7]);

    return;

}

procedure ValidatorConfig_register_candidate_validator_verify (arg0: Value, arg1: Value, arg2: Value) returns ()
{
    call ValidatorConfig_register_candidate_validator(arg0, arg1, arg2);
}

procedure {:inline 1} ValidatorConfig_rotate_consensus_pubkey (arg0: Value) returns ()
{
    // declare local variables
    var t0: Value; // bytearray
    var t1: Reference; // ValidatorConfig_T_ref
    var t2: Reference; // ValidatorConfig_Config_ref
    var t3: Reference; // bytearray_ref
    var t4: Value; // address
    var t5: Reference; // ValidatorConfig_T_ref
    var t6: Reference; // ValidatorConfig_T_ref
    var t7: Reference; // ValidatorConfig_Config_ref
    var t8: Reference; // ValidatorConfig_Config_ref
    var t9: Reference; // bytearray_ref
    var t10: Value; // bytearray
    var t11: Reference; // bytearray_ref

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#ByteArray(arg0);

    old_size := m_size;
    m_size := m_size + 12;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t5 := BorrowGlobal(contents#Memory(m)[old_size+4], ValidatorConfig_T);

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
    var t0: Value; // Vector_T
    var t1: Value; // Vector_T
    var t2: Value; // address
    var t3: Value; // address
    var t4: Value; // bool
    var t5: Value; // bool
    var t6: Value; // int
    var t7: Value; // Vector_T
    var t8: Value; // Vector_T
    var t9: Reference; // Vector_T_ref
    var t10: Value; // int
    var t11: Value; // int
    var t12: Value; // GasSchedule_Cost
    var t13: Reference; // Vector_T_ref
    var t14: Value; // int
    var t15: Value; // int
    var t16: Value; // GasSchedule_Cost
    var t17: Reference; // Vector_T_ref
    var t18: Value; // int
    var t19: Value; // int
    var t20: Value; // GasSchedule_Cost
    var t21: Reference; // Vector_T_ref
    var t22: Value; // int
    var t23: Value; // int
    var t24: Value; // GasSchedule_Cost
    var t25: Reference; // Vector_T_ref
    var t26: Value; // int
    var t27: Value; // int
    var t28: Value; // GasSchedule_Cost
    var t29: Reference; // Vector_T_ref
    var t30: Value; // int
    var t31: Value; // int
    var t32: Value; // GasSchedule_Cost
    var t33: Reference; // Vector_T_ref
    var t34: Value; // int
    var t35: Value; // int
    var t36: Value; // GasSchedule_Cost
    var t37: Reference; // Vector_T_ref
    var t38: Value; // int
    var t39: Value; // int
    var t40: Value; // GasSchedule_Cost
    var t41: Reference; // Vector_T_ref
    var t42: Value; // int
    var t43: Value; // int
    var t44: Value; // GasSchedule_Cost
    var t45: Reference; // Vector_T_ref
    var t46: Value; // int
    var t47: Value; // int
    var t48: Value; // GasSchedule_Cost
    var t49: Reference; // Vector_T_ref
    var t50: Value; // int
    var t51: Value; // int
    var t52: Value; // GasSchedule_Cost
    var t53: Reference; // Vector_T_ref
    var t54: Value; // int
    var t55: Value; // int
    var t56: Value; // GasSchedule_Cost
    var t57: Reference; // Vector_T_ref
    var t58: Value; // int
    var t59: Value; // int
    var t60: Value; // GasSchedule_Cost
    var t61: Reference; // Vector_T_ref
    var t62: Value; // int
    var t63: Value; // int
    var t64: Value; // GasSchedule_Cost
    var t65: Reference; // Vector_T_ref
    var t66: Value; // int
    var t67: Value; // int
    var t68: Value; // GasSchedule_Cost
    var t69: Reference; // Vector_T_ref
    var t70: Value; // int
    var t71: Value; // int
    var t72: Value; // GasSchedule_Cost
    var t73: Reference; // Vector_T_ref
    var t74: Value; // int
    var t75: Value; // int
    var t76: Value; // GasSchedule_Cost
    var t77: Reference; // Vector_T_ref
    var t78: Value; // int
    var t79: Value; // int
    var t80: Value; // GasSchedule_Cost
    var t81: Reference; // Vector_T_ref
    var t82: Value; // int
    var t83: Value; // int
    var t84: Value; // GasSchedule_Cost
    var t85: Reference; // Vector_T_ref
    var t86: Value; // int
    var t87: Value; // int
    var t88: Value; // GasSchedule_Cost
    var t89: Reference; // Vector_T_ref
    var t90: Value; // int
    var t91: Value; // int
    var t92: Value; // GasSchedule_Cost
    var t93: Reference; // Vector_T_ref
    var t94: Value; // int
    var t95: Value; // int
    var t96: Value; // GasSchedule_Cost
    var t97: Reference; // Vector_T_ref
    var t98: Value; // int
    var t99: Value; // int
    var t100: Value; // GasSchedule_Cost
    var t101: Reference; // Vector_T_ref
    var t102: Value; // int
    var t103: Value; // int
    var t104: Value; // GasSchedule_Cost
    var t105: Reference; // Vector_T_ref
    var t106: Value; // int
    var t107: Value; // int
    var t108: Value; // GasSchedule_Cost
    var t109: Reference; // Vector_T_ref
    var t110: Value; // int
    var t111: Value; // int
    var t112: Value; // GasSchedule_Cost
    var t113: Reference; // Vector_T_ref
    var t114: Value; // int
    var t115: Value; // int
    var t116: Value; // GasSchedule_Cost
    var t117: Reference; // Vector_T_ref
    var t118: Value; // int
    var t119: Value; // int
    var t120: Value; // GasSchedule_Cost
    var t121: Reference; // Vector_T_ref
    var t122: Value; // int
    var t123: Value; // int
    var t124: Value; // GasSchedule_Cost
    var t125: Reference; // Vector_T_ref
    var t126: Value; // int
    var t127: Value; // int
    var t128: Value; // GasSchedule_Cost
    var t129: Reference; // Vector_T_ref
    var t130: Value; // int
    var t131: Value; // int
    var t132: Value; // GasSchedule_Cost
    var t133: Reference; // Vector_T_ref
    var t134: Value; // int
    var t135: Value; // int
    var t136: Value; // GasSchedule_Cost
    var t137: Reference; // Vector_T_ref
    var t138: Value; // int
    var t139: Value; // int
    var t140: Value; // GasSchedule_Cost
    var t141: Reference; // Vector_T_ref
    var t142: Value; // int
    var t143: Value; // int
    var t144: Value; // GasSchedule_Cost
    var t145: Reference; // Vector_T_ref
    var t146: Value; // int
    var t147: Value; // int
    var t148: Value; // GasSchedule_Cost
    var t149: Reference; // Vector_T_ref
    var t150: Value; // int
    var t151: Value; // int
    var t152: Value; // GasSchedule_Cost
    var t153: Reference; // Vector_T_ref
    var t154: Value; // int
    var t155: Value; // int
    var t156: Value; // GasSchedule_Cost
    var t157: Reference; // Vector_T_ref
    var t158: Value; // int
    var t159: Value; // int
    var t160: Value; // GasSchedule_Cost
    var t161: Reference; // Vector_T_ref
    var t162: Value; // int
    var t163: Value; // int
    var t164: Value; // GasSchedule_Cost
    var t165: Reference; // Vector_T_ref
    var t166: Value; // int
    var t167: Value; // int
    var t168: Value; // GasSchedule_Cost
    var t169: Reference; // Vector_T_ref
    var t170: Value; // int
    var t171: Value; // int
    var t172: Value; // GasSchedule_Cost
    var t173: Reference; // Vector_T_ref
    var t174: Value; // int
    var t175: Value; // int
    var t176: Value; // GasSchedule_Cost
    var t177: Reference; // Vector_T_ref
    var t178: Value; // int
    var t179: Value; // int
    var t180: Value; // GasSchedule_Cost
    var t181: Reference; // Vector_T_ref
    var t182: Value; // int
    var t183: Value; // int
    var t184: Value; // GasSchedule_Cost
    var t185: Reference; // Vector_T_ref
    var t186: Value; // int
    var t187: Value; // int
    var t188: Value; // GasSchedule_Cost
    var t189: Reference; // Vector_T_ref
    var t190: Value; // int
    var t191: Value; // int
    var t192: Value; // GasSchedule_Cost
    var t193: Reference; // Vector_T_ref
    var t194: Value; // int
    var t195: Value; // int
    var t196: Value; // GasSchedule_Cost
    var t197: Reference; // Vector_T_ref
    var t198: Value; // int
    var t199: Value; // int
    var t200: Value; // GasSchedule_Cost
    var t201: Reference; // Vector_T_ref
    var t202: Value; // int
    var t203: Value; // int
    var t204: Value; // GasSchedule_Cost
    var t205: Reference; // Vector_T_ref
    var t206: Value; // int
    var t207: Value; // int
    var t208: Value; // GasSchedule_Cost
    var t209: Reference; // Vector_T_ref
    var t210: Value; // int
    var t211: Value; // int
    var t212: Value; // GasSchedule_Cost
    var t213: Reference; // Vector_T_ref
    var t214: Value; // int
    var t215: Value; // int
    var t216: Value; // GasSchedule_Cost
    var t217: Reference; // Vector_T_ref
    var t218: Value; // int
    var t219: Value; // int
    var t220: Value; // GasSchedule_Cost
    var t221: Reference; // Vector_T_ref
    var t222: Value; // int
    var t223: Value; // int
    var t224: Value; // GasSchedule_Cost
    var t225: Reference; // Vector_T_ref
    var t226: Value; // int
    var t227: Value; // int
    var t228: Value; // GasSchedule_Cost
    var t229: Reference; // Vector_T_ref
    var t230: Value; // int
    var t231: Value; // int
    var t232: Value; // GasSchedule_Cost
    var t233: Reference; // Vector_T_ref
    var t234: Value; // int
    var t235: Value; // int
    var t236: Value; // GasSchedule_Cost
    var t237: Reference; // Vector_T_ref
    var t238: Value; // int
    var t239: Value; // int
    var t240: Value; // GasSchedule_Cost
    var t241: Reference; // Vector_T_ref
    var t242: Value; // int
    var t243: Value; // int
    var t244: Value; // GasSchedule_Cost
    var t245: Reference; // Vector_T_ref
    var t246: Value; // int
    var t247: Value; // int
    var t248: Value; // GasSchedule_Cost
    var t249: Reference; // Vector_T_ref
    var t250: Value; // int
    var t251: Value; // int
    var t252: Value; // GasSchedule_Cost
    var t253: Reference; // Vector_T_ref
    var t254: Value; // int
    var t255: Value; // int
    var t256: Value; // GasSchedule_Cost
    var t257: Reference; // Vector_T_ref
    var t258: Value; // int
    var t259: Value; // int
    var t260: Value; // GasSchedule_Cost
    var t261: Reference; // Vector_T_ref
    var t262: Value; // int
    var t263: Value; // int
    var t264: Value; // GasSchedule_Cost
    var t265: Reference; // Vector_T_ref
    var t266: Value; // int
    var t267: Value; // int
    var t268: Value; // GasSchedule_Cost
    var t269: Reference; // Vector_T_ref
    var t270: Value; // int
    var t271: Value; // int
    var t272: Value; // GasSchedule_Cost
    var t273: Reference; // Vector_T_ref
    var t274: Value; // int
    var t275: Value; // int
    var t276: Value; // GasSchedule_Cost
    var t277: Reference; // Vector_T_ref
    var t278: Value; // int
    var t279: Value; // int
    var t280: Value; // GasSchedule_Cost
    var t281: Reference; // Vector_T_ref
    var t282: Value; // int
    var t283: Value; // int
    var t284: Value; // GasSchedule_Cost
    var t285: Reference; // Vector_T_ref
    var t286: Value; // int
    var t287: Value; // int
    var t288: Value; // GasSchedule_Cost
    var t289: Value; // Vector_T
    var t290: Value; // Vector_T
    var t291: Value; // GasSchedule_T

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

    call tmp := Eq(contents#Memory(m)[old_size+2], contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 5];
if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    assert false;

Label_7:
    call t7 := Vector_empty();
    assume is#Vector(t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call t8 := Vector_empty();
    assume is#Vector(t8);

    m := Memory(domain#Memory(m)[old_size+8 := true], contents#Memory(m)[old_size+8 := t8]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t9 := BorrowLoc(old_size+0);

    call tmp := LdConst(27);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+10]);

    assume is#Integer(contents#Memory(m)[old_size+11]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+10], contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call Vector_push_back(t9, contents#Memory(m)[old_size+12]);

    call t13 := BorrowLoc(old_size+0);

    call tmp := LdConst(28);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+14]);

    assume is#Integer(contents#Memory(m)[old_size+15]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+14], contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call Vector_push_back(t13, contents#Memory(m)[old_size+16]);

    call t17 := BorrowLoc(old_size+0);

    call tmp := LdConst(31);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+18]);

    assume is#Integer(contents#Memory(m)[old_size+19]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+18], contents#Memory(m)[old_size+19]);
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call Vector_push_back(t17, contents#Memory(m)[old_size+20]);

    call t21 := BorrowLoc(old_size+0);

    call tmp := LdConst(29);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+22]);

    assume is#Integer(contents#Memory(m)[old_size+23]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+22], contents#Memory(m)[old_size+23]);
    m := Memory(domain#Memory(m)[24+old_size := true], contents#Memory(m)[24+old_size := tmp]);

    call Vector_push_back(t21, contents#Memory(m)[old_size+24]);

    call t25 := BorrowLoc(old_size+0);

    call tmp := LdConst(10);
    m := Memory(domain#Memory(m)[26+old_size := true], contents#Memory(m)[26+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[27+old_size := true], contents#Memory(m)[27+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+26]);

    assume is#Integer(contents#Memory(m)[old_size+27]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+26], contents#Memory(m)[old_size+27]);
    m := Memory(domain#Memory(m)[28+old_size := true], contents#Memory(m)[28+old_size := tmp]);

    call Vector_push_back(t25, contents#Memory(m)[old_size+28]);

    call t29 := BorrowLoc(old_size+0);

    call tmp := LdConst(29);
    m := Memory(domain#Memory(m)[30+old_size := true], contents#Memory(m)[30+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[31+old_size := true], contents#Memory(m)[31+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+30]);

    assume is#Integer(contents#Memory(m)[old_size+31]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+30], contents#Memory(m)[old_size+31]);
    m := Memory(domain#Memory(m)[32+old_size := true], contents#Memory(m)[32+old_size := tmp]);

    call Vector_push_back(t29, contents#Memory(m)[old_size+32]);

    call t33 := BorrowLoc(old_size+0);

    call tmp := LdConst(36);
    m := Memory(domain#Memory(m)[34+old_size := true], contents#Memory(m)[34+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[35+old_size := true], contents#Memory(m)[35+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+34]);

    assume is#Integer(contents#Memory(m)[old_size+35]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+34], contents#Memory(m)[old_size+35]);
    m := Memory(domain#Memory(m)[36+old_size := true], contents#Memory(m)[36+old_size := tmp]);

    call Vector_push_back(t33, contents#Memory(m)[old_size+36]);

    call t37 := BorrowLoc(old_size+0);

    call tmp := LdConst(52);
    m := Memory(domain#Memory(m)[38+old_size := true], contents#Memory(m)[38+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[39+old_size := true], contents#Memory(m)[39+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+38]);

    assume is#Integer(contents#Memory(m)[old_size+39]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+38], contents#Memory(m)[old_size+39]);
    m := Memory(domain#Memory(m)[40+old_size := true], contents#Memory(m)[40+old_size := tmp]);

    call Vector_push_back(t37, contents#Memory(m)[old_size+40]);

    call t41 := BorrowLoc(old_size+0);

    call tmp := LdConst(29);
    m := Memory(domain#Memory(m)[42+old_size := true], contents#Memory(m)[42+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[43+old_size := true], contents#Memory(m)[43+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+42]);

    assume is#Integer(contents#Memory(m)[old_size+43]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+42], contents#Memory(m)[old_size+43]);
    m := Memory(domain#Memory(m)[44+old_size := true], contents#Memory(m)[44+old_size := tmp]);

    call Vector_push_back(t41, contents#Memory(m)[old_size+44]);

    call t45 := BorrowLoc(old_size+0);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[46+old_size := true], contents#Memory(m)[46+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[47+old_size := true], contents#Memory(m)[47+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+46]);

    assume is#Integer(contents#Memory(m)[old_size+47]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+46], contents#Memory(m)[old_size+47]);
    m := Memory(domain#Memory(m)[48+old_size := true], contents#Memory(m)[48+old_size := tmp]);

    call Vector_push_back(t45, contents#Memory(m)[old_size+48]);

    call t49 := BorrowLoc(old_size+0);

    call tmp := LdConst(41);
    m := Memory(domain#Memory(m)[50+old_size := true], contents#Memory(m)[50+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[51+old_size := true], contents#Memory(m)[51+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+50]);

    assume is#Integer(contents#Memory(m)[old_size+51]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+50], contents#Memory(m)[old_size+51]);
    m := Memory(domain#Memory(m)[52+old_size := true], contents#Memory(m)[52+old_size := tmp]);

    call Vector_push_back(t49, contents#Memory(m)[old_size+52]);

    call t53 := BorrowLoc(old_size+0);

    call tmp := LdConst(41);
    m := Memory(domain#Memory(m)[54+old_size := true], contents#Memory(m)[54+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[55+old_size := true], contents#Memory(m)[55+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+54]);

    assume is#Integer(contents#Memory(m)[old_size+55]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+54], contents#Memory(m)[old_size+55]);
    m := Memory(domain#Memory(m)[56+old_size := true], contents#Memory(m)[56+old_size := tmp]);

    call Vector_push_back(t53, contents#Memory(m)[old_size+56]);

    call t57 := BorrowLoc(old_size+0);

    call tmp := LdConst(28);
    m := Memory(domain#Memory(m)[58+old_size := true], contents#Memory(m)[58+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[59+old_size := true], contents#Memory(m)[59+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+58]);

    assume is#Integer(contents#Memory(m)[old_size+59]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+58], contents#Memory(m)[old_size+59]);
    m := Memory(domain#Memory(m)[60+old_size := true], contents#Memory(m)[60+old_size := tmp]);

    call Vector_push_back(t57, contents#Memory(m)[old_size+60]);

    call t61 := BorrowLoc(old_size+0);

    call tmp := LdConst(45);
    m := Memory(domain#Memory(m)[62+old_size := true], contents#Memory(m)[62+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[63+old_size := true], contents#Memory(m)[63+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+62]);

    assume is#Integer(contents#Memory(m)[old_size+63]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+62], contents#Memory(m)[old_size+63]);
    m := Memory(domain#Memory(m)[64+old_size := true], contents#Memory(m)[64+old_size := tmp]);

    call Vector_push_back(t61, contents#Memory(m)[old_size+64]);

    call t65 := BorrowLoc(old_size+0);

    call tmp := LdConst(45);
    m := Memory(domain#Memory(m)[66+old_size := true], contents#Memory(m)[66+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[67+old_size := true], contents#Memory(m)[67+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+66]);

    assume is#Integer(contents#Memory(m)[old_size+67]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+66], contents#Memory(m)[old_size+67]);
    m := Memory(domain#Memory(m)[68+old_size := true], contents#Memory(m)[68+old_size := tmp]);

    call Vector_push_back(t65, contents#Memory(m)[old_size+68]);

    call t69 := BorrowLoc(old_size+0);

    call tmp := LdConst(58);
    m := Memory(domain#Memory(m)[70+old_size := true], contents#Memory(m)[70+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[71+old_size := true], contents#Memory(m)[71+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+70]);

    assume is#Integer(contents#Memory(m)[old_size+71]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+70], contents#Memory(m)[old_size+71]);
    m := Memory(domain#Memory(m)[72+old_size := true], contents#Memory(m)[72+old_size := tmp]);

    call Vector_push_back(t69, contents#Memory(m)[old_size+72]);

    call t73 := BorrowLoc(old_size+0);

    call tmp := LdConst(58);
    m := Memory(domain#Memory(m)[74+old_size := true], contents#Memory(m)[74+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[75+old_size := true], contents#Memory(m)[75+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+74]);

    assume is#Integer(contents#Memory(m)[old_size+75]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+74], contents#Memory(m)[old_size+75]);
    m := Memory(domain#Memory(m)[76+old_size := true], contents#Memory(m)[76+old_size := tmp]);

    call Vector_push_back(t73, contents#Memory(m)[old_size+76]);

    call t77 := BorrowLoc(old_size+0);

    call tmp := LdConst(56);
    m := Memory(domain#Memory(m)[78+old_size := true], contents#Memory(m)[78+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[79+old_size := true], contents#Memory(m)[79+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+78]);

    assume is#Integer(contents#Memory(m)[old_size+79]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+78], contents#Memory(m)[old_size+79]);
    m := Memory(domain#Memory(m)[80+old_size := true], contents#Memory(m)[80+old_size := tmp]);

    call Vector_push_back(t77, contents#Memory(m)[old_size+80]);

    call t81 := BorrowLoc(old_size+0);

    call tmp := LdConst(197);
    m := Memory(domain#Memory(m)[82+old_size := true], contents#Memory(m)[82+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[83+old_size := true], contents#Memory(m)[83+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+82]);

    assume is#Integer(contents#Memory(m)[old_size+83]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+82], contents#Memory(m)[old_size+83]);
    m := Memory(domain#Memory(m)[84+old_size := true], contents#Memory(m)[84+old_size := tmp]);

    call Vector_push_back(t81, contents#Memory(m)[old_size+84]);

    call t85 := BorrowLoc(old_size+0);

    call tmp := LdConst(73);
    m := Memory(domain#Memory(m)[86+old_size := true], contents#Memory(m)[86+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[87+old_size := true], contents#Memory(m)[87+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+86]);

    assume is#Integer(contents#Memory(m)[old_size+87]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+86], contents#Memory(m)[old_size+87]);
    m := Memory(domain#Memory(m)[88+old_size := true], contents#Memory(m)[88+old_size := tmp]);

    call Vector_push_back(t85, contents#Memory(m)[old_size+88]);

    call t89 := BorrowLoc(old_size+0);

    call tmp := LdConst(94);
    m := Memory(domain#Memory(m)[90+old_size := true], contents#Memory(m)[90+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[91+old_size := true], contents#Memory(m)[91+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+90]);

    assume is#Integer(contents#Memory(m)[old_size+91]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+90], contents#Memory(m)[old_size+91]);
    m := Memory(domain#Memory(m)[92+old_size := true], contents#Memory(m)[92+old_size := tmp]);

    call Vector_push_back(t89, contents#Memory(m)[old_size+92]);

    call t93 := BorrowLoc(old_size+0);

    call tmp := LdConst(51);
    m := Memory(domain#Memory(m)[94+old_size := true], contents#Memory(m)[94+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[95+old_size := true], contents#Memory(m)[95+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+94]);

    assume is#Integer(contents#Memory(m)[old_size+95]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+94], contents#Memory(m)[old_size+95]);
    m := Memory(domain#Memory(m)[96+old_size := true], contents#Memory(m)[96+old_size := tmp]);

    call Vector_push_back(t93, contents#Memory(m)[old_size+96]);

    call t97 := BorrowLoc(old_size+0);

    call tmp := LdConst(65);
    m := Memory(domain#Memory(m)[98+old_size := true], contents#Memory(m)[98+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[99+old_size := true], contents#Memory(m)[99+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+98]);

    assume is#Integer(contents#Memory(m)[old_size+99]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+98], contents#Memory(m)[old_size+99]);
    m := Memory(domain#Memory(m)[100+old_size := true], contents#Memory(m)[100+old_size := tmp]);

    call Vector_push_back(t97, contents#Memory(m)[old_size+100]);

    call t101 := BorrowLoc(old_size+0);

    call tmp := LdConst(45);
    m := Memory(domain#Memory(m)[102+old_size := true], contents#Memory(m)[102+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[103+old_size := true], contents#Memory(m)[103+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+102]);

    assume is#Integer(contents#Memory(m)[old_size+103]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+102], contents#Memory(m)[old_size+103]);
    m := Memory(domain#Memory(m)[104+old_size := true], contents#Memory(m)[104+old_size := tmp]);

    call Vector_push_back(t101, contents#Memory(m)[old_size+104]);

    call t105 := BorrowLoc(old_size+0);

    call tmp := LdConst(44);
    m := Memory(domain#Memory(m)[106+old_size := true], contents#Memory(m)[106+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[107+old_size := true], contents#Memory(m)[107+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+106]);

    assume is#Integer(contents#Memory(m)[old_size+107]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+106], contents#Memory(m)[old_size+107]);
    m := Memory(domain#Memory(m)[108+old_size := true], contents#Memory(m)[108+old_size := tmp]);

    call Vector_push_back(t105, contents#Memory(m)[old_size+108]);

    call t109 := BorrowLoc(old_size+0);

    call tmp := LdConst(41);
    m := Memory(domain#Memory(m)[110+old_size := true], contents#Memory(m)[110+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[111+old_size := true], contents#Memory(m)[111+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+110]);

    assume is#Integer(contents#Memory(m)[old_size+111]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+110], contents#Memory(m)[old_size+111]);
    m := Memory(domain#Memory(m)[112+old_size := true], contents#Memory(m)[112+old_size := tmp]);

    call Vector_push_back(t109, contents#Memory(m)[old_size+112]);

    call t113 := BorrowLoc(old_size+0);

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[114+old_size := true], contents#Memory(m)[114+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[115+old_size := true], contents#Memory(m)[115+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+114]);

    assume is#Integer(contents#Memory(m)[old_size+115]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+114], contents#Memory(m)[old_size+115]);
    m := Memory(domain#Memory(m)[116+old_size := true], contents#Memory(m)[116+old_size := tmp]);

    call Vector_push_back(t113, contents#Memory(m)[old_size+116]);

    call t117 := BorrowLoc(old_size+0);

    call tmp := LdConst(41);
    m := Memory(domain#Memory(m)[118+old_size := true], contents#Memory(m)[118+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[119+old_size := true], contents#Memory(m)[119+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+118]);

    assume is#Integer(contents#Memory(m)[old_size+119]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+118], contents#Memory(m)[old_size+119]);
    m := Memory(domain#Memory(m)[120+old_size := true], contents#Memory(m)[120+old_size := tmp]);

    call Vector_push_back(t117, contents#Memory(m)[old_size+120]);

    call t121 := BorrowLoc(old_size+0);

    call tmp := LdConst(45);
    m := Memory(domain#Memory(m)[122+old_size := true], contents#Memory(m)[122+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[123+old_size := true], contents#Memory(m)[123+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+122]);

    assume is#Integer(contents#Memory(m)[old_size+123]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+122], contents#Memory(m)[old_size+123]);
    m := Memory(domain#Memory(m)[124+old_size := true], contents#Memory(m)[124+old_size := tmp]);

    call Vector_push_back(t121, contents#Memory(m)[old_size+124]);

    call t125 := BorrowLoc(old_size+0);

    call tmp := LdConst(44);
    m := Memory(domain#Memory(m)[126+old_size := true], contents#Memory(m)[126+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[127+old_size := true], contents#Memory(m)[127+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+126]);

    assume is#Integer(contents#Memory(m)[old_size+127]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+126], contents#Memory(m)[old_size+127]);
    m := Memory(domain#Memory(m)[128+old_size := true], contents#Memory(m)[128+old_size := tmp]);

    call Vector_push_back(t125, contents#Memory(m)[old_size+128]);

    call t129 := BorrowLoc(old_size+0);

    call tmp := LdConst(46);
    m := Memory(domain#Memory(m)[130+old_size := true], contents#Memory(m)[130+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[131+old_size := true], contents#Memory(m)[131+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+130]);

    assume is#Integer(contents#Memory(m)[old_size+131]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+130], contents#Memory(m)[old_size+131]);
    m := Memory(domain#Memory(m)[132+old_size := true], contents#Memory(m)[132+old_size := tmp]);

    call Vector_push_back(t129, contents#Memory(m)[old_size+132]);

    call t133 := BorrowLoc(old_size+0);

    call tmp := LdConst(43);
    m := Memory(domain#Memory(m)[134+old_size := true], contents#Memory(m)[134+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[135+old_size := true], contents#Memory(m)[135+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+134]);

    assume is#Integer(contents#Memory(m)[old_size+135]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+134], contents#Memory(m)[old_size+135]);
    m := Memory(domain#Memory(m)[136+old_size := true], contents#Memory(m)[136+old_size := tmp]);

    call Vector_push_back(t133, contents#Memory(m)[old_size+136]);

    call t137 := BorrowLoc(old_size+0);

    call tmp := LdConst(49);
    m := Memory(domain#Memory(m)[138+old_size := true], contents#Memory(m)[138+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[139+old_size := true], contents#Memory(m)[139+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+138]);

    assume is#Integer(contents#Memory(m)[old_size+139]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+138], contents#Memory(m)[old_size+139]);
    m := Memory(domain#Memory(m)[140+old_size := true], contents#Memory(m)[140+old_size := tmp]);

    call Vector_push_back(t137, contents#Memory(m)[old_size+140]);

    call t141 := BorrowLoc(old_size+0);

    call tmp := LdConst(35);
    m := Memory(domain#Memory(m)[142+old_size := true], contents#Memory(m)[142+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[143+old_size := true], contents#Memory(m)[143+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+142]);

    assume is#Integer(contents#Memory(m)[old_size+143]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+142], contents#Memory(m)[old_size+143]);
    m := Memory(domain#Memory(m)[144+old_size := true], contents#Memory(m)[144+old_size := tmp]);

    call Vector_push_back(t141, contents#Memory(m)[old_size+144]);

    call t145 := BorrowLoc(old_size+0);

    call tmp := LdConst(48);
    m := Memory(domain#Memory(m)[146+old_size := true], contents#Memory(m)[146+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[147+old_size := true], contents#Memory(m)[147+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+146]);

    assume is#Integer(contents#Memory(m)[old_size+147]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+146], contents#Memory(m)[old_size+147]);
    m := Memory(domain#Memory(m)[148+old_size := true], contents#Memory(m)[148+old_size := tmp]);

    call Vector_push_back(t145, contents#Memory(m)[old_size+148]);

    call t149 := BorrowLoc(old_size+0);

    call tmp := LdConst(51);
    m := Memory(domain#Memory(m)[150+old_size := true], contents#Memory(m)[150+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[151+old_size := true], contents#Memory(m)[151+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+150]);

    assume is#Integer(contents#Memory(m)[old_size+151]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+150], contents#Memory(m)[old_size+151]);
    m := Memory(domain#Memory(m)[152+old_size := true], contents#Memory(m)[152+old_size := tmp]);

    call Vector_push_back(t149, contents#Memory(m)[old_size+152]);

    call t153 := BorrowLoc(old_size+0);

    call tmp := LdConst(49);
    m := Memory(domain#Memory(m)[154+old_size := true], contents#Memory(m)[154+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[155+old_size := true], contents#Memory(m)[155+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+154]);

    assume is#Integer(contents#Memory(m)[old_size+155]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+154], contents#Memory(m)[old_size+155]);
    m := Memory(domain#Memory(m)[156+old_size := true], contents#Memory(m)[156+old_size := tmp]);

    call Vector_push_back(t153, contents#Memory(m)[old_size+156]);

    call t157 := BorrowLoc(old_size+0);

    call tmp := LdConst(46);
    m := Memory(domain#Memory(m)[158+old_size := true], contents#Memory(m)[158+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[159+old_size := true], contents#Memory(m)[159+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+158]);

    assume is#Integer(contents#Memory(m)[old_size+159]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+158], contents#Memory(m)[old_size+159]);
    m := Memory(domain#Memory(m)[160+old_size := true], contents#Memory(m)[160+old_size := tmp]);

    call Vector_push_back(t157, contents#Memory(m)[old_size+160]);

    call t161 := BorrowLoc(old_size+0);

    call tmp := LdConst(47);
    m := Memory(domain#Memory(m)[162+old_size := true], contents#Memory(m)[162+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[163+old_size := true], contents#Memory(m)[163+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+162]);

    assume is#Integer(contents#Memory(m)[old_size+163]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+162], contents#Memory(m)[old_size+163]);
    m := Memory(domain#Memory(m)[164+old_size := true], contents#Memory(m)[164+old_size := tmp]);

    call Vector_push_back(t161, contents#Memory(m)[old_size+164]);

    call t165 := BorrowLoc(old_size+0);

    call tmp := LdConst(46);
    m := Memory(domain#Memory(m)[166+old_size := true], contents#Memory(m)[166+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[167+old_size := true], contents#Memory(m)[167+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+166]);

    assume is#Integer(contents#Memory(m)[old_size+167]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+166], contents#Memory(m)[old_size+167]);
    m := Memory(domain#Memory(m)[168+old_size := true], contents#Memory(m)[168+old_size := tmp]);

    call Vector_push_back(t165, contents#Memory(m)[old_size+168]);

    call t169 := BorrowLoc(old_size+0);

    call tmp := LdConst(39);
    m := Memory(domain#Memory(m)[170+old_size := true], contents#Memory(m)[170+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[171+old_size := true], contents#Memory(m)[171+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+170]);

    assume is#Integer(contents#Memory(m)[old_size+171]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+170], contents#Memory(m)[old_size+171]);
    m := Memory(domain#Memory(m)[172+old_size := true], contents#Memory(m)[172+old_size := tmp]);

    call Vector_push_back(t169, contents#Memory(m)[old_size+172]);

    call t173 := BorrowLoc(old_size+0);

    call tmp := LdConst(29);
    m := Memory(domain#Memory(m)[174+old_size := true], contents#Memory(m)[174+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[175+old_size := true], contents#Memory(m)[175+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+174]);

    assume is#Integer(contents#Memory(m)[old_size+175]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+174], contents#Memory(m)[old_size+175]);
    m := Memory(domain#Memory(m)[176+old_size := true], contents#Memory(m)[176+old_size := tmp]);

    call Vector_push_back(t173, contents#Memory(m)[old_size+176]);

    call t177 := BorrowLoc(old_size+0);

    call tmp := LdConst(34);
    m := Memory(domain#Memory(m)[178+old_size := true], contents#Memory(m)[178+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[179+old_size := true], contents#Memory(m)[179+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+178]);

    assume is#Integer(contents#Memory(m)[old_size+179]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+178], contents#Memory(m)[old_size+179]);
    m := Memory(domain#Memory(m)[180+old_size := true], contents#Memory(m)[180+old_size := tmp]);

    call Vector_push_back(t177, contents#Memory(m)[old_size+180]);

    call t181 := BorrowLoc(old_size+0);

    call tmp := LdConst(32);
    m := Memory(domain#Memory(m)[182+old_size := true], contents#Memory(m)[182+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[183+old_size := true], contents#Memory(m)[183+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+182]);

    assume is#Integer(contents#Memory(m)[old_size+183]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+182], contents#Memory(m)[old_size+183]);
    m := Memory(domain#Memory(m)[184+old_size := true], contents#Memory(m)[184+old_size := tmp]);

    call Vector_push_back(t181, contents#Memory(m)[old_size+184]);

    call t185 := BorrowLoc(old_size+0);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[186+old_size := true], contents#Memory(m)[186+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[187+old_size := true], contents#Memory(m)[187+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+186]);

    assume is#Integer(contents#Memory(m)[old_size+187]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+186], contents#Memory(m)[old_size+187]);
    m := Memory(domain#Memory(m)[188+old_size := true], contents#Memory(m)[188+old_size := tmp]);

    call Vector_push_back(t185, contents#Memory(m)[old_size+188]);

    call t189 := BorrowLoc(old_size+0);

    call tmp := LdConst(856);
    m := Memory(domain#Memory(m)[190+old_size := true], contents#Memory(m)[190+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[191+old_size := true], contents#Memory(m)[191+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+190]);

    assume is#Integer(contents#Memory(m)[old_size+191]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+190], contents#Memory(m)[old_size+191]);
    m := Memory(domain#Memory(m)[192+old_size := true], contents#Memory(m)[192+old_size := tmp]);

    call Vector_push_back(t189, contents#Memory(m)[old_size+192]);

    call t193 := BorrowLoc(old_size+0);

    call tmp := LdConst(929);
    m := Memory(domain#Memory(m)[194+old_size := true], contents#Memory(m)[194+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[195+old_size := true], contents#Memory(m)[195+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+194]);

    assume is#Integer(contents#Memory(m)[old_size+195]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+194], contents#Memory(m)[old_size+195]);
    m := Memory(domain#Memory(m)[196+old_size := true], contents#Memory(m)[196+old_size := tmp]);

    call Vector_push_back(t193, contents#Memory(m)[old_size+196]);

    call t197 := BorrowLoc(old_size+0);

    call tmp := LdConst(929);
    m := Memory(domain#Memory(m)[198+old_size := true], contents#Memory(m)[198+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[199+old_size := true], contents#Memory(m)[199+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+198]);

    assume is#Integer(contents#Memory(m)[old_size+199]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+198], contents#Memory(m)[old_size+199]);
    m := Memory(domain#Memory(m)[200+old_size := true], contents#Memory(m)[200+old_size := tmp]);

    call Vector_push_back(t197, contents#Memory(m)[old_size+200]);

    call t201 := BorrowLoc(old_size+0);

    call tmp := LdConst(917);
    m := Memory(domain#Memory(m)[202+old_size := true], contents#Memory(m)[202+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[203+old_size := true], contents#Memory(m)[203+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+202]);

    assume is#Integer(contents#Memory(m)[old_size+203]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+202], contents#Memory(m)[old_size+203]);
    m := Memory(domain#Memory(m)[204+old_size := true], contents#Memory(m)[204+old_size := tmp]);

    call Vector_push_back(t201, contents#Memory(m)[old_size+204]);

    call t205 := BorrowLoc(old_size+0);

    call tmp := LdConst(774);
    m := Memory(domain#Memory(m)[206+old_size := true], contents#Memory(m)[206+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[207+old_size := true], contents#Memory(m)[207+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+206]);

    assume is#Integer(contents#Memory(m)[old_size+207]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+206], contents#Memory(m)[old_size+207]);
    m := Memory(domain#Memory(m)[208+old_size := true], contents#Memory(m)[208+old_size := tmp]);

    call Vector_push_back(t205, contents#Memory(m)[old_size+208]);

    call t209 := BorrowLoc(old_size+0);

    call tmp := LdConst(29);
    m := Memory(domain#Memory(m)[210+old_size := true], contents#Memory(m)[210+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[211+old_size := true], contents#Memory(m)[211+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+210]);

    assume is#Integer(contents#Memory(m)[old_size+211]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+210], contents#Memory(m)[old_size+211]);
    m := Memory(domain#Memory(m)[212+old_size := true], contents#Memory(m)[212+old_size := tmp]);

    call Vector_push_back(t209, contents#Memory(m)[old_size+212]);

    call t213 := BorrowLoc(old_size+0);

    call tmp := LdConst(41);
    m := Memory(domain#Memory(m)[214+old_size := true], contents#Memory(m)[214+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[215+old_size := true], contents#Memory(m)[215+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+214]);

    assume is#Integer(contents#Memory(m)[old_size+215]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+214], contents#Memory(m)[old_size+215]);
    m := Memory(domain#Memory(m)[216+old_size := true], contents#Memory(m)[216+old_size := tmp]);

    call Vector_push_back(t213, contents#Memory(m)[old_size+216]);

    call t217 := BorrowLoc(old_size+0);

    call tmp := LdConst(10);
    m := Memory(domain#Memory(m)[218+old_size := true], contents#Memory(m)[218+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[219+old_size := true], contents#Memory(m)[219+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+218]);

    assume is#Integer(contents#Memory(m)[old_size+219]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+218], contents#Memory(m)[old_size+219]);
    m := Memory(domain#Memory(m)[220+old_size := true], contents#Memory(m)[220+old_size := tmp]);

    call Vector_push_back(t217, contents#Memory(m)[old_size+220]);

    call t221 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[222+old_size := true], contents#Memory(m)[222+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[223+old_size := true], contents#Memory(m)[223+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+222]);

    assume is#Integer(contents#Memory(m)[old_size+223]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+222], contents#Memory(m)[old_size+223]);
    m := Memory(domain#Memory(m)[224+old_size := true], contents#Memory(m)[224+old_size := tmp]);

    call Vector_push_back(t221, contents#Memory(m)[old_size+224]);

    call t225 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[226+old_size := true], contents#Memory(m)[226+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[227+old_size := true], contents#Memory(m)[227+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+226]);

    assume is#Integer(contents#Memory(m)[old_size+227]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+226], contents#Memory(m)[old_size+227]);
    m := Memory(domain#Memory(m)[228+old_size := true], contents#Memory(m)[228+old_size := tmp]);

    call Vector_push_back(t225, contents#Memory(m)[old_size+228]);

    call t229 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[230+old_size := true], contents#Memory(m)[230+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[231+old_size := true], contents#Memory(m)[231+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+230]);

    assume is#Integer(contents#Memory(m)[old_size+231]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+230], contents#Memory(m)[old_size+231]);
    m := Memory(domain#Memory(m)[232+old_size := true], contents#Memory(m)[232+old_size := tmp]);

    call Vector_push_back(t229, contents#Memory(m)[old_size+232]);

    call t233 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[234+old_size := true], contents#Memory(m)[234+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[235+old_size := true], contents#Memory(m)[235+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+234]);

    assume is#Integer(contents#Memory(m)[old_size+235]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+234], contents#Memory(m)[old_size+235]);
    m := Memory(domain#Memory(m)[236+old_size := true], contents#Memory(m)[236+old_size := tmp]);

    call Vector_push_back(t233, contents#Memory(m)[old_size+236]);

    call t237 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[238+old_size := true], contents#Memory(m)[238+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[239+old_size := true], contents#Memory(m)[239+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+238]);

    assume is#Integer(contents#Memory(m)[old_size+239]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+238], contents#Memory(m)[old_size+239]);
    m := Memory(domain#Memory(m)[240+old_size := true], contents#Memory(m)[240+old_size := tmp]);

    call Vector_push_back(t237, contents#Memory(m)[old_size+240]);

    call t241 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[242+old_size := true], contents#Memory(m)[242+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[243+old_size := true], contents#Memory(m)[243+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+242]);

    assume is#Integer(contents#Memory(m)[old_size+243]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+242], contents#Memory(m)[old_size+243]);
    m := Memory(domain#Memory(m)[244+old_size := true], contents#Memory(m)[244+old_size := tmp]);

    call Vector_push_back(t241, contents#Memory(m)[old_size+244]);

    call t245 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[246+old_size := true], contents#Memory(m)[246+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[247+old_size := true], contents#Memory(m)[247+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+246]);

    assume is#Integer(contents#Memory(m)[old_size+247]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+246], contents#Memory(m)[old_size+247]);
    m := Memory(domain#Memory(m)[248+old_size := true], contents#Memory(m)[248+old_size := tmp]);

    call Vector_push_back(t245, contents#Memory(m)[old_size+248]);

    call t249 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[250+old_size := true], contents#Memory(m)[250+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[251+old_size := true], contents#Memory(m)[251+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+250]);

    assume is#Integer(contents#Memory(m)[old_size+251]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+250], contents#Memory(m)[old_size+251]);
    m := Memory(domain#Memory(m)[252+old_size := true], contents#Memory(m)[252+old_size := tmp]);

    call Vector_push_back(t249, contents#Memory(m)[old_size+252]);

    call t253 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[254+old_size := true], contents#Memory(m)[254+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[255+old_size := true], contents#Memory(m)[255+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+254]);

    assume is#Integer(contents#Memory(m)[old_size+255]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+254], contents#Memory(m)[old_size+255]);
    m := Memory(domain#Memory(m)[256+old_size := true], contents#Memory(m)[256+old_size := tmp]);

    call Vector_push_back(t253, contents#Memory(m)[old_size+256]);

    call t257 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[258+old_size := true], contents#Memory(m)[258+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[259+old_size := true], contents#Memory(m)[259+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+258]);

    assume is#Integer(contents#Memory(m)[old_size+259]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+258], contents#Memory(m)[old_size+259]);
    m := Memory(domain#Memory(m)[260+old_size := true], contents#Memory(m)[260+old_size := tmp]);

    call Vector_push_back(t257, contents#Memory(m)[old_size+260]);

    call t261 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[262+old_size := true], contents#Memory(m)[262+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[263+old_size := true], contents#Memory(m)[263+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+262]);

    assume is#Integer(contents#Memory(m)[old_size+263]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+262], contents#Memory(m)[old_size+263]);
    m := Memory(domain#Memory(m)[264+old_size := true], contents#Memory(m)[264+old_size := tmp]);

    call Vector_push_back(t261, contents#Memory(m)[old_size+264]);

    call t265 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[266+old_size := true], contents#Memory(m)[266+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[267+old_size := true], contents#Memory(m)[267+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+266]);

    assume is#Integer(contents#Memory(m)[old_size+267]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+266], contents#Memory(m)[old_size+267]);
    m := Memory(domain#Memory(m)[268+old_size := true], contents#Memory(m)[268+old_size := tmp]);

    call Vector_push_back(t265, contents#Memory(m)[old_size+268]);

    call t269 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[270+old_size := true], contents#Memory(m)[270+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[271+old_size := true], contents#Memory(m)[271+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+270]);

    assume is#Integer(contents#Memory(m)[old_size+271]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+270], contents#Memory(m)[old_size+271]);
    m := Memory(domain#Memory(m)[272+old_size := true], contents#Memory(m)[272+old_size := tmp]);

    call Vector_push_back(t269, contents#Memory(m)[old_size+272]);

    call t273 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[274+old_size := true], contents#Memory(m)[274+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[275+old_size := true], contents#Memory(m)[275+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+274]);

    assume is#Integer(contents#Memory(m)[old_size+275]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+274], contents#Memory(m)[old_size+275]);
    m := Memory(domain#Memory(m)[276+old_size := true], contents#Memory(m)[276+old_size := tmp]);

    call Vector_push_back(t273, contents#Memory(m)[old_size+276]);

    call t277 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[278+old_size := true], contents#Memory(m)[278+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[279+old_size := true], contents#Memory(m)[279+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+278]);

    assume is#Integer(contents#Memory(m)[old_size+279]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+278], contents#Memory(m)[old_size+279]);
    m := Memory(domain#Memory(m)[280+old_size := true], contents#Memory(m)[280+old_size := tmp]);

    call Vector_push_back(t277, contents#Memory(m)[old_size+280]);

    call t281 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[282+old_size := true], contents#Memory(m)[282+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[283+old_size := true], contents#Memory(m)[283+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+282]);

    assume is#Integer(contents#Memory(m)[old_size+283]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+282], contents#Memory(m)[old_size+283]);
    m := Memory(domain#Memory(m)[284+old_size := true], contents#Memory(m)[284+old_size := tmp]);

    call Vector_push_back(t281, contents#Memory(m)[old_size+284]);

    call t285 := BorrowLoc(old_size+1);

    call tmp := LdConst(30);
    m := Memory(domain#Memory(m)[286+old_size := true], contents#Memory(m)[286+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[287+old_size := true], contents#Memory(m)[287+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+286]);

    assume is#Integer(contents#Memory(m)[old_size+287]);

    call tmp := Pack_GasSchedule_Cost(contents#Memory(m)[old_size+286], contents#Memory(m)[old_size+287]);
    m := Memory(domain#Memory(m)[288+old_size := true], contents#Memory(m)[288+old_size := tmp]);

    call Vector_push_back(t285, contents#Memory(m)[old_size+288]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[289+old_size := true], contents#Memory(m)[289+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[290+old_size := true], contents#Memory(m)[290+old_size := tmp]);

    assume is#Vector(contents#Memory(m)[old_size+289]);

    assume is#Vector(contents#Memory(m)[old_size+290]);

    call tmp := Pack_GasSchedule_T(contents#Memory(m)[old_size+289], contents#Memory(m)[old_size+290]);
    m := Memory(domain#Memory(m)[291+old_size := true], contents#Memory(m)[291+old_size := tmp]);

    call MoveToSender(GasSchedule_T, contents#Memory(m)[old_size+291]);

    return;

}

procedure GasSchedule_initialize_verify () returns ()
{
    call GasSchedule_initialize();
}

procedure {:inline 1} GasSchedule_new_cost (arg0: Value, arg1: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // int
    var t1: Value; // int
    var t2: Value; // int
    var t3: Value; // int
    var t4: Value; // GasSchedule_Cost

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#Integer(arg1);

    old_size := m_size;
    m_size := m_size + 5;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+2]);

    assume is#Integer(contents#Memory(m)[old_size+3]);

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
    var t0: Value; // address
    var t1: Value; // bool

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 2;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := Exists(contents#Memory(m)[old_size+0], GasSchedule_T);
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
    var t0: Reference; // GasSchedule_T_ref
    var t1: Value; // int
    var t2: Value; // address
    var t3: Reference; // GasSchedule_T_ref
    var t4: Reference; // GasSchedule_T_ref
    var t5: Reference; // Vector_T_ref
    var t6: Value; // int
    var t7: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 8;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], GasSchedule_T);

    call t0 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, GasSchedule_T_instruction_schedule);

    call t6 := Vector_length(t5);
    assume is#Integer(t6);

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
    var t0: Reference; // GasSchedule_T_ref
    var t1: Value; // int
    var t2: Value; // address
    var t3: Reference; // GasSchedule_T_ref
    var t4: Reference; // GasSchedule_T_ref
    var t5: Reference; // Vector_T_ref
    var t6: Value; // int
    var t7: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 8;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], GasSchedule_T);

    call t0 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, GasSchedule_T_native_schedule);

    call t6 := Vector_length(t5);
    assume is#Integer(t6);

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
    var t0: Value; // address
    var t1: Value; // LibraCoin_T
    var t2: Value; // LibraAccount_EventHandleGenerator
    var t3: Value; // LibraAccount_EventHandle
    var t4: Value; // LibraAccount_EventHandle
    var t5: Value; // bytearray
    var t6: Value; // address
    var t7: Value; // bytearray
    var t8: Value; // int
    var t9: Value; // LibraAccount_EventHandleGenerator
    var t10: Reference; // LibraAccount_EventHandleGenerator_ref
    var t11: Value; // address
    var t12: Value; // LibraAccount_EventHandle
    var t13: Reference; // LibraAccount_EventHandleGenerator_ref
    var t14: Value; // address
    var t15: Value; // LibraAccount_EventHandle
    var t16: Value; // LibraCoin_T
    var t17: Value; // bytearray
    var t18: Value; // LibraCoin_T
    var t19: Value; // bool
    var t20: Value; // bool
    var t21: Value; // LibraAccount_EventHandle
    var t22: Value; // LibraAccount_EventHandle
    var t23: Value; // int
    var t24: Value; // LibraAccount_EventHandleGenerator
    var t25: Value; // LibraAccount_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 26;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call t7 := AddressUtil_address_to_bytes(contents#Memory(m)[old_size+6]);
    assume is#ByteArray(t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+8]);

    call tmp := Pack_LibraAccount_EventHandleGenerator(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t10 := BorrowLoc(old_size+2);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call t12 := LibraAccount_new_event_handle_impl(t10, contents#Memory(m)[old_size+11]);
    assume is#Map(t12);

    m := Memory(domain#Memory(m)[old_size+12 := true], contents#Memory(m)[old_size+12 := t12]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+12]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t13 := BorrowLoc(old_size+2);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call t15 := LibraAccount_new_event_handle_impl(t13, contents#Memory(m)[old_size+14]);
    assume is#Map(t15);

    m := Memory(domain#Memory(m)[old_size+15 := true], contents#Memory(m)[old_size+15 := t15]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t16 := LibraCoin_zero();
    assume is#Map(t16);

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

    assume is#ByteArray(contents#Memory(m)[old_size+17]);

    assume is#Map(contents#Memory(m)[old_size+18]);

    assume is#Boolean(contents#Memory(m)[old_size+19]);

    assume is#Boolean(contents#Memory(m)[old_size+20]);

    assume is#Map(contents#Memory(m)[old_size+21]);

    assume is#Map(contents#Memory(m)[old_size+22]);

    assume is#Integer(contents#Memory(m)[old_size+23]);

    assume is#Map(contents#Memory(m)[old_size+24]);

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
    var t0: Value; // address
    var t1: Value; // LibraCoin_T
    var t2: Value; // address
    var t3: Value; // LibraCoin_T
    var t4: Value; // bytearray

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume is#Map(arg1);

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
    var t0: Value; // address
    var t1: Value; // LibraCoin_T
    var t2: Value; // bytearray
    var t3: Value; // int
    var t4: Reference; // LibraAccount_T_ref
    var t5: Value; // address
    var t6: Reference; // LibraAccount_T_ref
    var t7: Reference; // LibraCoin_T_ref
    var t8: Value; // int
    var t9: Value; // int
    var t10: Value; // int
    var t11: Value; // bool
    var t12: Value; // bool
    var t13: Value; // int
    var t14: Value; // address
    var t15: Value; // address
    var t16: Reference; // LibraAccount_T_ref
    var t17: Reference; // LibraAccount_T_ref
    var t18: Reference; // LibraAccount_EventHandle_ref
    var t19: Value; // int
    var t20: Value; // address
    var t21: Value; // bytearray
    var t22: Value; // LibraAccount_SentPaymentEvent
    var t23: Value; // address
    var t24: Reference; // LibraAccount_T_ref
    var t25: Reference; // LibraAccount_T_ref
    var t26: Reference; // LibraCoin_T_ref
    var t27: Value; // LibraCoin_T
    var t28: Reference; // LibraAccount_T_ref
    var t29: Reference; // LibraAccount_EventHandle_ref
    var t30: Value; // int
    var t31: Value; // address
    var t32: Value; // bytearray
    var t33: Value; // LibraAccount_ReceivedPaymentEvent

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume is#Map(arg1);
    assume is#ByteArray(arg2);

    old_size := m_size;
    m_size := m_size + 34;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size :=  arg2]);

    // bytecode translation starts here
    call t7 := BorrowLoc(old_size+1);

    call t8 := LibraCoin_value(t7);
    assume is#Integer(t8);

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

    call t16 := BorrowGlobal(contents#Memory(m)[old_size+15], LibraAccount_T);

    call t6 := CopyOrMoveRef(t16);

    call t17 := CopyOrMoveRef(t6);

    call t18 := BorrowField(t17, LibraAccount_T_sent_events);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+19]);

    assume is#Address(contents#Memory(m)[old_size+20]);

    assume is#ByteArray(contents#Memory(m)[old_size+21]);

    call tmp := Pack_LibraAccount_SentPaymentEvent(contents#Memory(m)[old_size+19], contents#Memory(m)[old_size+20], contents#Memory(m)[old_size+21]);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    call LibraAccount_emit_event(t18, contents#Memory(m)[old_size+22]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    call t24 := BorrowGlobal(contents#Memory(m)[old_size+23], LibraAccount_T);

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

    assume is#Integer(contents#Memory(m)[old_size+30]);

    assume is#Address(contents#Memory(m)[old_size+31]);

    assume is#ByteArray(contents#Memory(m)[old_size+32]);

    call tmp := Pack_LibraAccount_ReceivedPaymentEvent(contents#Memory(m)[old_size+30], contents#Memory(m)[old_size+31], contents#Memory(m)[old_size+32]);
    m := Memory(domain#Memory(m)[33+old_size := true], contents#Memory(m)[33+old_size := tmp]);

    call LibraAccount_emit_event(t29, contents#Memory(m)[old_size+33]);

    return;

}

procedure LibraAccount_deposit_with_metadata_verify (arg0: Value, arg1: Value, arg2: Value) returns ()
{
    call LibraAccount_deposit_with_metadata(arg0, arg1, arg2);
}

procedure {:inline 1} LibraAccount_mint_to_address (arg0: Value, arg1: Value) returns ()
{
    // declare local variables
    var t0: Value; // address
    var t1: Value; // int
    var t2: Value; // address
    var t3: Value; // bool
    var t4: Value; // bool
    var t5: Value; // address
    var t6: Value; // address
    var t7: Value; // int
    var t8: Value; // LibraCoin_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume is#Integer(arg1);

    old_size := m_size;
    m_size := m_size + 9;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := Exists(contents#Memory(m)[old_size+2], LibraAccount_T);
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
    assume is#Map(t8);

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
    var t0: Reference; // LibraAccount_T_ref
    var t1: Value; // int
    var t2: Value; // LibraCoin_T
    var t3: Reference; // LibraAccount_T_ref
    var t4: Reference; // LibraCoin_T_ref
    var t5: Value; // int
    var t6: Value; // LibraCoin_T
    var t7: Value; // LibraCoin_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
        assume is#Integer(arg1);

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
    assume is#Map(t6);

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
    var t0: Value; // int
    var t1: Reference; // LibraAccount_T_ref
    var t2: Value; // address
    var t3: Reference; // LibraAccount_T_ref
    var t4: Reference; // LibraAccount_T_ref
    var t5: Reference; // bool_ref
    var t6: Value; // bool
    var t7: Value; // int
    var t8: Reference; // LibraAccount_T_ref
    var t9: Value; // int
    var t10: Value; // LibraCoin_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Integer(arg0);

    old_size := m_size;
    m_size := m_size + 11;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], LibraAccount_T);

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, LibraAccount_T_delegated_withdrawal_capability);

    call tmp := ReadRef(t5);
    assume is#Boolean(tmp);

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
    assume is#Map(t10);

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
    var t0: Reference; // LibraAccount_WithdrawalCapability_ref
    var t1: Value; // int
    var t2: Reference; // LibraAccount_T_ref
    var t3: Reference; // LibraAccount_WithdrawalCapability_ref
    var t4: Reference; // address_ref
    var t5: Value; // address
    var t6: Reference; // LibraAccount_T_ref
    var t7: Reference; // LibraAccount_T_ref
    var t8: Value; // int
    var t9: Value; // LibraCoin_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
        assume is#Integer(arg1);

    old_size := m_size;
    m_size := m_size + 10;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t3 := CopyOrMoveRef(t0);

    call t4 := BorrowField(t3, LibraAccount_WithdrawalCapability_account_address);

    call tmp := ReadRef(t4);
    assume is#Address(tmp);

    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := BorrowGlobal(contents#Memory(m)[old_size+5], LibraAccount_T);

    call t2 := CopyOrMoveRef(t6);

    call t7 := CopyOrMoveRef(t2);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call t9 := LibraAccount_withdraw_from_account(t7, contents#Memory(m)[old_size+8]);
    assume is#Map(t9);

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
    var t0: Value; // address
    var t1: Reference; // LibraAccount_T_ref
    var t2: Reference; // bool_ref
    var t3: Value; // address
    var t4: Value; // address
    var t5: Reference; // LibraAccount_T_ref
    var t6: Reference; // LibraAccount_T_ref
    var t7: Reference; // bool_ref
    var t8: Reference; // bool_ref
    var t9: Value; // bool
    var t10: Value; // int
    var t11: Value; // bool
    var t12: Reference; // bool_ref
    var t13: Value; // address
    var t14: Value; // LibraAccount_WithdrawalCapability

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

    call t5 := BorrowGlobal(contents#Memory(m)[old_size+4], LibraAccount_T);

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call t7 := BorrowField(t6, LibraAccount_T_delegated_withdrawal_capability);

    call t2 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t8);
    assume is#Boolean(tmp);

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

    assume is#Address(contents#Memory(m)[old_size+13]);

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
    var t0: Value; // LibraAccount_WithdrawalCapability
    var t1: Value; // address
    var t2: Reference; // LibraAccount_T_ref
    var t3: Value; // LibraAccount_WithdrawalCapability
    var t4: Value; // address
    var t5: Value; // address
    var t6: Reference; // LibraAccount_T_ref
    var t7: Value; // bool
    var t8: Reference; // LibraAccount_T_ref
    var t9: Reference; // bool_ref

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Map(arg0);

    old_size := m_size;
    m_size := m_size + 10;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t4 := Unpack_LibraAccount_WithdrawalCapability(contents#Memory(m)[old_size+3]);
    assume is#Address(t4);

    m := Memory(domain#Memory(m)[old_size+4 := true], contents#Memory(m)[old_size+4 := t4]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := BorrowGlobal(contents#Memory(m)[old_size+5], LibraAccount_T);

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
    var t0: Value; // address
    var t1: Value; // int
    var t2: Value; // bytearray
    var t3: Value; // address
    var t4: Value; // bool
    var t5: Value; // bool
    var t6: Value; // address
    var t7: Value; // address
    var t8: Value; // int
    var t9: Value; // LibraCoin_T
    var t10: Value; // bytearray

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume is#Integer(arg1);
    assume is#ByteArray(arg2);

    old_size := m_size;
    m_size := m_size + 11;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size :=  arg2]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := Exists(contents#Memory(m)[old_size+3], LibraAccount_T);
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
    assume is#Map(t9);

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
    var t0: Value; // address
    var t1: Value; // int
    var t2: Value; // address
    var t3: Value; // int
    var t4: Value; // bytearray

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume is#Integer(arg1);

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
    var t0: Reference; // LibraAccount_T_ref
    var t1: Value; // bytearray
    var t2: Value; // bytearray
    var t3: Reference; // LibraAccount_T_ref
    var t4: Reference; // bytearray_ref

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
        assume is#ByteArray(arg1);

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
    var t0: Value; // bytearray
    var t1: Reference; // LibraAccount_T_ref
    var t2: Value; // address
    var t3: Reference; // LibraAccount_T_ref
    var t4: Reference; // LibraAccount_T_ref
    var t5: Reference; // bool_ref
    var t6: Value; // bool
    var t7: Value; // int
    var t8: Reference; // LibraAccount_T_ref
    var t9: Value; // bytearray

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#ByteArray(arg0);

    old_size := m_size;
    m_size := m_size + 10;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], LibraAccount_T);

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call t5 := BorrowField(t4, LibraAccount_T_delegated_key_rotation_capability);

    call tmp := ReadRef(t5);
    assume is#Boolean(tmp);

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
    var t0: Reference; // LibraAccount_KeyRotationCapability_ref
    var t1: Value; // bytearray
    var t2: Reference; // LibraAccount_KeyRotationCapability_ref
    var t3: Reference; // address_ref
    var t4: Value; // address
    var t5: Reference; // LibraAccount_T_ref
    var t6: Value; // bytearray

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
        assume is#ByteArray(arg1);

    old_size := m_size;
    m_size := m_size + 7;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t2 := CopyOrMoveRef(t0);

    call t3 := BorrowField(t2, LibraAccount_KeyRotationCapability_account_address);

    call tmp := ReadRef(t3);
    assume is#Address(tmp);

    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t5 := BorrowGlobal(contents#Memory(m)[old_size+4], LibraAccount_T);

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
    var t0: Value; // address
    var t1: Reference; // bool_ref
    var t2: Value; // address
    var t3: Value; // address
    var t4: Reference; // LibraAccount_T_ref
    var t5: Reference; // bool_ref
    var t6: Reference; // bool_ref
    var t7: Value; // bool
    var t8: Value; // int
    var t9: Value; // bool
    var t10: Reference; // bool_ref
    var t11: Value; // address
    var t12: Value; // LibraAccount_KeyRotationCapability

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

    call t4 := BorrowGlobal(contents#Memory(m)[old_size+3], LibraAccount_T);

    call t5 := BorrowField(t4, LibraAccount_T_delegated_key_rotation_capability);

    call t1 := CopyOrMoveRef(t5);

    call t6 := CopyOrMoveRef(t1);

    call tmp := ReadRef(t6);
    assume is#Boolean(tmp);

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

    assume is#Address(contents#Memory(m)[old_size+11]);

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
    var t0: Value; // LibraAccount_KeyRotationCapability
    var t1: Value; // address
    var t2: Reference; // LibraAccount_T_ref
    var t3: Value; // LibraAccount_KeyRotationCapability
    var t4: Value; // address
    var t5: Value; // address
    var t6: Reference; // LibraAccount_T_ref
    var t7: Value; // bool
    var t8: Reference; // LibraAccount_T_ref
    var t9: Reference; // bool_ref

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Map(arg0);

    old_size := m_size;
    m_size := m_size + 10;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t4 := Unpack_LibraAccount_KeyRotationCapability(contents#Memory(m)[old_size+3]);
    assume is#Address(t4);

    m := Memory(domain#Memory(m)[old_size+4 := true], contents#Memory(m)[old_size+4 := t4]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := BorrowGlobal(contents#Memory(m)[old_size+5], LibraAccount_T);

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
    var t0: Value; // address
    var t1: Value; // LibraAccount_EventHandleGenerator
    var t2: Value; // int
    var t3: Value; // LibraAccount_EventHandleGenerator
    var t4: Value; // address
    var t5: Value; // address
    var t6: Value; // bytearray
    var t7: Value; // LibraCoin_T
    var t8: Value; // bool
    var t9: Value; // bool
    var t10: Reference; // LibraAccount_EventHandleGenerator_ref
    var t11: Value; // address
    var t12: Value; // LibraAccount_EventHandle
    var t13: Reference; // LibraAccount_EventHandleGenerator_ref
    var t14: Value; // address
    var t15: Value; // LibraAccount_EventHandle
    var t16: Value; // int
    var t17: Value; // LibraAccount_EventHandleGenerator
    var t18: Value; // LibraAccount_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 19;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+2]);

    call tmp := Pack_LibraAccount_EventHandleGenerator(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := AddressUtil_address_to_bytes(contents#Memory(m)[old_size+5]);
    assume is#ByteArray(t6);

    m := Memory(domain#Memory(m)[old_size+6 := true], contents#Memory(m)[old_size+6 := t6]);

    call t7 := LibraCoin_zero();
    assume is#Map(t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call t10 := BorrowLoc(old_size+1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call t12 := LibraAccount_new_event_handle_impl(t10, contents#Memory(m)[old_size+11]);
    assume is#Map(t12);

    m := Memory(domain#Memory(m)[old_size+12 := true], contents#Memory(m)[old_size+12 := t12]);

    call t13 := BorrowLoc(old_size+1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call t15 := LibraAccount_new_event_handle_impl(t13, contents#Memory(m)[old_size+14]);
    assume is#Map(t15);

    m := Memory(domain#Memory(m)[old_size+15 := true], contents#Memory(m)[old_size+15 := t15]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    assume is#ByteArray(contents#Memory(m)[old_size+6]);

    assume is#Map(contents#Memory(m)[old_size+7]);

    assume is#Boolean(contents#Memory(m)[old_size+8]);

    assume is#Boolean(contents#Memory(m)[old_size+9]);

    assume is#Map(contents#Memory(m)[old_size+12]);

    assume is#Map(contents#Memory(m)[old_size+15]);

    assume is#Integer(contents#Memory(m)[old_size+16]);

    assume is#Map(contents#Memory(m)[old_size+17]);

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
    var t0: Value; // address
    var t1: Value; // int
    var t2: Value; // address
    var t3: Value; // int
    var t4: Value; // int
    var t5: Value; // bool
    var t6: Value; // address
    var t7: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);
    assume is#Integer(arg1);

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
    var t0: Reference; // LibraAccount_T_ref
    var t1: Value; // int
    var t2: Reference; // LibraAccount_T_ref
    var t3: Reference; // LibraCoin_T_ref
    var t4: Value; // int
    var t5: Value; // int

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
    assume is#Integer(t4);

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
    var t0: Value; // address
    var t1: Value; // address
    var t2: Reference; // LibraAccount_T_ref
    var t3: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 4;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t2 := BorrowGlobal(contents#Memory(m)[old_size+1], LibraAccount_T);

    call t3 := LibraAccount_balance_for_account(t2);
    assume is#Integer(t3);

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
    var t0: Reference; // LibraAccount_T_ref
    var t1: Reference; // LibraAccount_T_ref
    var t2: Reference; // int_ref
    var t3: Value; // int

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
    assume is#Integer(tmp);

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
    var t0: Value; // address
    var t1: Value; // address
    var t2: Reference; // LibraAccount_T_ref
    var t3: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 4;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t2 := BorrowGlobal(contents#Memory(m)[old_size+1], LibraAccount_T);

    call t3 := LibraAccount_sequence_number_for_account(t2);
    assume is#Integer(t3);

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
    var t0: Value; // address
    var t1: Value; // address
    var t2: Reference; // LibraAccount_T_ref
    var t3: Reference; // bool_ref
    var t4: Value; // bool

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 5;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t2 := BorrowGlobal(contents#Memory(m)[old_size+1], LibraAccount_T);

    call t3 := BorrowField(t2, LibraAccount_T_delegated_key_rotation_capability);

    call tmp := ReadRef(t3);
    assume is#Boolean(tmp);

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
    var t0: Value; // address
    var t1: Value; // address
    var t2: Reference; // LibraAccount_T_ref
    var t3: Reference; // bool_ref
    var t4: Value; // bool

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 5;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t2 := BorrowGlobal(contents#Memory(m)[old_size+1], LibraAccount_T);

    call t3 := BorrowField(t2, LibraAccount_T_delegated_withdrawal_capability);

    call tmp := ReadRef(t3);
    assume is#Boolean(tmp);

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
    var t0: Reference; // LibraAccount_WithdrawalCapability_ref
    var t1: Reference; // LibraAccount_WithdrawalCapability_ref
    var t2: Reference; // address_ref

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
    var t0: Reference; // LibraAccount_KeyRotationCapability_ref
    var t1: Reference; // LibraAccount_KeyRotationCapability_ref
    var t2: Reference; // address_ref

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
    var t0: Value; // address
    var t1: Value; // address
    var t2: Value; // bool

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 3;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := Exists(contents#Memory(m)[old_size+1], LibraAccount_T);
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
    var t0: Value; // address
    var t1: Value; // bool
    var t2: Reference; // LibraAccount_T_ref
    var t3: Reference; // LibraAccount_T_ref
    var t4: Value; // bytearray
    var t5: Value; // bytearray
    var t6: Value; // int
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // int
    var t10: Value; // int
    var t11: Value; // int
    var t12: Value; // address
    var t13: Value; // address
    var t14: Value; // bool
    var t15: Value; // bool
    var t16: Value; // bool
    var t17: Value; // int
    var t18: Value; // address
    var t19: Reference; // LibraAccount_T_ref
    var t20: Value; // bytearray
    var t21: Value; // bytearray
    var t22: Value; // bytearray
    var t23: Value; // bytearray
    var t24: Reference; // LibraAccount_T_ref
    var t25: Reference; // bytearray_ref
    var t26: Value; // bytearray
    var t27: Value; // bool
    var t28: Value; // bool
    var t29: Value; // int
    var t30: Value; // int
    var t31: Value; // int
    var t32: Value; // int
    var t33: Value; // int
    var t34: Value; // int
    var t35: Reference; // LibraAccount_T_ref
    var t36: Reference; // LibraAccount_T_ref
    var t37: Reference; // LibraAccount_T_ref
    var t38: Value; // int
    var t39: Value; // int
    var t40: Value; // int
    var t41: Value; // bool
    var t42: Value; // bool
    var t43: Value; // int
    var t44: Reference; // LibraAccount_T_ref
    var t45: Reference; // int_ref
    var t46: Value; // int
    var t47: Value; // int
    var t48: Value; // int
    var t49: Value; // int
    var t50: Value; // bool
    var t51: Value; // bool
    var t52: Value; // int
    var t53: Value; // int
    var t54: Value; // int
    var t55: Value; // bool
    var t56: Value; // bool
    var t57: Value; // int

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

    call tmp := Exists(contents#Memory(m)[old_size+13], LibraAccount_T);
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

    call t19 := BorrowGlobal(contents#Memory(m)[old_size+18], LibraAccount_T);

    call t2 := CopyOrMoveRef(t19);

    call tmp := GetTxnPublicKey();
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+20]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    call t22 := Hash_sha3_256(contents#Memory(m)[old_size+21]);
    assume is#ByteArray(t22);

    m := Memory(domain#Memory(m)[old_size+22 := true], contents#Memory(m)[old_size+22 := t22]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+22]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    call t24 := CopyOrMoveRef(t2);

    call t25 := BorrowField(t24, LibraAccount_T_authentication_key);

    call tmp := ReadRef(t25);
    assume is#ByteArray(tmp);

    m := Memory(domain#Memory(m)[26+old_size := true], contents#Memory(m)[26+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+23], contents#Memory(m)[old_size+26]);
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
    assume is#Integer(t38);

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
    assume is#Integer(tmp);

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

    call tmp := Eq(contents#Memory(m)[old_size+53], contents#Memory(m)[old_size+54]);
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
    var t0: Value; // address
    var t1: Reference; // LibraAccount_T_ref
    var t2: Reference; // LibraAccount_T_ref
    var t3: Reference; // LibraAccount_T_ref
    var t4: Value; // int
    var t5: Value; // int
    var t6: Value; // int
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // LibraCoin_T
    var t10: Value; // int
    var t11: Value; // address
    var t12: Value; // address
    var t13: Reference; // LibraAccount_T_ref
    var t14: Value; // int
    var t15: Value; // int
    var t16: Value; // int
    var t17: Value; // int
    var t18: Value; // int
    var t19: Value; // int
    var t20: Value; // int
    var t21: Value; // int
    var t22: Reference; // LibraAccount_T_ref
    var t23: Reference; // LibraAccount_T_ref
    var t24: Reference; // LibraAccount_T_ref
    var t25: Value; // int
    var t26: Value; // int
    var t27: Value; // int
    var t28: Value; // bool
    var t29: Value; // bool
    var t30: Value; // int
    var t31: Reference; // LibraAccount_T_ref
    var t32: Value; // int
    var t33: Value; // LibraCoin_T
    var t34: Value; // int
    var t35: Value; // int
    var t36: Value; // int
    var t37: Value; // int
    var t38: Reference; // LibraAccount_T_ref
    var t39: Reference; // int_ref
    var t40: Value; // address
    var t41: Reference; // LibraAccount_T_ref
    var t42: Reference; // LibraAccount_T_ref
    var t43: Reference; // LibraCoin_T_ref
    var t44: Value; // LibraCoin_T

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

    call t13 := BorrowGlobal(contents#Memory(m)[old_size+12], LibraAccount_T);

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
    assume is#Integer(t25);

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
    assume is#Map(t33);

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

    call t41 := BorrowGlobal(contents#Memory(m)[old_size+40], LibraAccount_T);

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
    var t0: Reference; // LibraAccount_EventHandleGenerator_ref
    var t1: Value; // address
    var t2: Reference; // int_ref
    var t3: Value; // bytearray
    var t4: Value; // bytearray
    var t5: Value; // bytearray
    var t6: Reference; // LibraAccount_EventHandleGenerator_ref
    var t7: Reference; // int_ref
    var t8: Value; // address
    var t9: Value; // bytearray
    var t10: Reference; // int_ref
    var t11: Value; // int
    var t12: Value; // bytearray
    var t13: Reference; // int_ref
    var t14: Value; // int
    var t15: Value; // int
    var t16: Value; // int
    var t17: Reference; // int_ref
    var t18: Value; // bytearray
    var t19: Value; // bytearray
    var t20: Value; // bytearray
    var t21: Value; // bytearray

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
        assume is#Address(arg1);

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
    assume is#ByteArray(t9);

    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t10 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t10);
    assume is#Integer(tmp);

    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call t12 := U64Util_u64_to_bytes(contents#Memory(m)[old_size+11]);
    assume is#ByteArray(t12);

    m := Memory(domain#Memory(m)[old_size+12 := true], contents#Memory(m)[old_size+12 := t12]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+12]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t13 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t13);
    assume is#Integer(tmp);

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
    assume is#ByteArray(t20);

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

procedure {:inline 1} LibraAccount_new_event_handle_impl (arg0: Reference, arg1: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Reference; // LibraAccount_EventHandleGenerator_ref
    var t1: Value; // address
    var t2: Value; // int
    var t3: Reference; // LibraAccount_EventHandleGenerator_ref
    var t4: Value; // address
    var t5: Value; // bytearray
    var t6: Value; // LibraAccount_EventHandle

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
        assume is#Address(arg1);

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
    assume is#ByteArray(t5);

    m := Memory(domain#Memory(m)[old_size+5 := true], contents#Memory(m)[old_size+5 := t5]);

    assume is#Integer(contents#Memory(m)[old_size+2]);

    assume is#ByteArray(contents#Memory(m)[old_size+5]);

    call tmp := Pack_LibraAccount_EventHandle(contents#Memory(m)[old_size+2], contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+6];
    return;

}

procedure LibraAccount_new_event_handle_impl_verify (arg0: Reference, arg1: Value) returns (ret0: Value)
{
    call ret0 := LibraAccount_new_event_handle_impl(arg0, arg1);
}

procedure {:inline 1} LibraAccount_new_event_handle () returns (ret0: Value)
{
    // declare local variables
    var t0: Reference; // LibraAccount_T_ref
    var t1: Value; // bytearray
    var t2: Value; // address
    var t3: Reference; // LibraAccount_T_ref
    var t4: Reference; // LibraAccount_T_ref
    var t5: Reference; // LibraAccount_EventHandleGenerator_ref
    var t6: Value; // address
    var t7: Value; // LibraAccount_EventHandle

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 8;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], LibraAccount_T);

    call t0 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, LibraAccount_T_event_generator);

    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call t7 := LibraAccount_new_event_handle_impl(t5, contents#Memory(m)[old_size+6]);
    assume is#Map(t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    ret0 := contents#Memory(m)[old_size+7];
    return;

}

procedure LibraAccount_new_event_handle_verify () returns (ret0: Value)
{
    call ret0 := LibraAccount_new_event_handle();
}

procedure {:inline 1} LibraAccount_emit_event (arg0: Reference, arg1: Value) returns ()
{
    // declare local variables
    var t0: Reference; // LibraAccount_EventHandle_ref
    var t1: Value; // typeparam
    var t2: Reference; // int_ref
    var t3: Value; // bytearray
    var t4: Reference; // LibraAccount_EventHandle_ref
    var t5: Reference; // bytearray_ref
    var t6: Value; // bytearray
    var t7: Reference; // LibraAccount_EventHandle_ref
    var t8: Reference; // int_ref
    var t9: Value; // bytearray
    var t10: Reference; // int_ref
    var t11: Value; // int
    var t12: Value; // typeparam
    var t13: Reference; // int_ref
    var t14: Value; // int
    var t15: Value; // int
    var t16: Value; // int
    var t17: Reference; // int_ref

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 18;
    t0 := arg0;
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t4 := CopyOrMoveRef(t0);

    call t5 := BorrowField(t4, LibraAccount_EventHandle_guid);

    call tmp := ReadRef(t5);
    assume is#ByteArray(tmp);

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
    assume is#Integer(tmp);

    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call LibraAccount_write_to_event_store(contents#Memory(m)[old_size+9], contents#Memory(m)[old_size+11], contents#Memory(m)[old_size+12]);

    call t13 := CopyOrMoveRef(t2);

    call tmp := ReadRef(t13);
    assume is#Integer(tmp);

    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+14], contents#Memory(m)[old_size+15]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call t17 := CopyOrMoveRef(t2);

    call WriteRef(t17, contents#Memory(m)[old_size+16]);

    return;

}

procedure LibraAccount_emit_event_verify (arg0: Reference, arg1: Value) returns ()
{
    call LibraAccount_emit_event(arg0, arg1);
}

procedure {:inline 1} LibraAccount_write_to_event_store (arg0: Value, arg1: Value, arg2: Value) returns ();
procedure {:inline 1} LibraAccount_destroy_handle (arg0: Value) returns ()
{
    // declare local variables
    var t0: Value; // LibraAccount_EventHandle
    var t1: Value; // bytearray
    var t2: Value; // int
    var t3: Value; // LibraAccount_EventHandle
    var t4: Value; // int
    var t5: Value; // bytearray

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Map(arg0);

    old_size := m_size;
    m_size := m_size + 6;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t4, t5 := Unpack_LibraAccount_EventHandle(contents#Memory(m)[old_size+3]);
    assume is#Integer(t4);

    assume is#ByteArray(t5);

    m := Memory(domain#Memory(m)[old_size+4 := true], contents#Memory(m)[old_size+4 := t4]);
    m := Memory(domain#Memory(m)[old_size+5 := true], contents#Memory(m)[old_size+5 := t5]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    return;

}

procedure LibraAccount_destroy_handle_verify (arg0: Value) returns ()
{
    call LibraAccount_destroy_handle(arg0);
}

procedure {:inline 1} LibraSystem_initialize_validator_set () returns ()
{
    // declare local variables
    var t0: Value; // address
    var t1: Value; // address
    var t2: Value; // bool
    var t3: Value; // bool
    var t4: Value; // int
    var t5: Value; // Vector_T
    var t6: Value; // LibraAccount_EventHandle
    var t7: Value; // LibraSystem_ValidatorSet

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 8;

    // bytecode translation starts here
    call tmp := GetTxnSenderAddress();
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := LdAddr(472);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+0], contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 3];
if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    assert false;

Label_7:
    call t5 := Vector_empty();
    assume is#Vector(t5);

    m := Memory(domain#Memory(m)[old_size+5 := true], contents#Memory(m)[old_size+5 := t5]);

    call t6 := LibraAccount_new_event_handle();
    assume is#Map(t6);

    m := Memory(domain#Memory(m)[old_size+6 := true], contents#Memory(m)[old_size+6 := t6]);

    assume is#Vector(contents#Memory(m)[old_size+5]);

    assume is#Map(contents#Memory(m)[old_size+6]);

    call tmp := Pack_LibraSystem_ValidatorSet(contents#Memory(m)[old_size+5], contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call MoveToSender(LibraSystem_ValidatorSet, contents#Memory(m)[old_size+7]);

    return;

}

procedure LibraSystem_initialize_validator_set_verify () returns ()
{
    call LibraSystem_initialize_validator_set();
}

procedure {:inline 1} LibraSystem_initialize_block_metadata () returns ()
{
    // declare local variables
    var t0: Value; // address
    var t1: Value; // address
    var t2: Value; // bool
    var t3: Value; // bool
    var t4: Value; // int
    var t5: Value; // int
    var t6: Value; // int
    var t7: Value; // bytearray
    var t8: Value; // address
    var t9: Value; // LibraSystem_BlockMetadata

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

    call tmp := Eq(contents#Memory(m)[old_size+0], contents#Memory(m)[old_size+1]);
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

    assume is#Integer(contents#Memory(m)[old_size+5]);

    assume is#Integer(contents#Memory(m)[old_size+6]);

    assume is#ByteArray(contents#Memory(m)[old_size+7]);

    assume is#Address(contents#Memory(m)[old_size+8]);

    call tmp := Pack_LibraSystem_BlockMetadata(contents#Memory(m)[old_size+5], contents#Memory(m)[old_size+6], contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call MoveToSender(LibraSystem_BlockMetadata, contents#Memory(m)[old_size+9]);

    return;

}

procedure LibraSystem_initialize_block_metadata_verify () returns ()
{
    call LibraSystem_initialize_block_metadata();
}

procedure {:inline 1} LibraSystem_get_consensus_pubkey (arg0: Reference) returns (ret0: Reference)
{
    // declare local variables
    var t0: Reference; // LibraSystem_ValidatorInfo_ref
    var t1: Reference; // LibraSystem_ValidatorInfo_ref
    var t2: Reference; // bytearray_ref

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
    var t0: Reference; // LibraSystem_ValidatorInfo_ref
    var t1: Reference; // LibraSystem_ValidatorInfo_ref
    var t2: Reference; // int_ref

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
    var t0: Reference; // LibraSystem_ValidatorInfo_ref
    var t1: Reference; // LibraSystem_ValidatorInfo_ref
    var t2: Reference; // bytearray_ref

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
    var t0: Reference; // LibraSystem_ValidatorInfo_ref
    var t1: Reference; // LibraSystem_ValidatorInfo_ref
    var t2: Reference; // bytearray_ref

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
    var t0: Value; // int
    var t1: Value; // bytearray
    var t2: Value; // bytearray
    var t3: Value; // address
    var t4: Value; // int
    var t5: Value; // bytearray
    var t6: Value; // bytearray
    var t7: Value; // address

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#ByteArray(arg1);
    assume is#ByteArray(arg2);
    assume is#Address(arg3);

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
    var t0: Value; // int
    var t1: Value; // bytearray
    var t2: Value; // bytearray
    var t3: Value; // address
    var t4: Reference; // LibraSystem_BlockMetadata_ref
    var t5: Value; // address
    var t6: Reference; // LibraSystem_BlockMetadata_ref
    var t7: Value; // int
    var t8: Reference; // LibraSystem_BlockMetadata_ref
    var t9: Reference; // int_ref
    var t10: Value; // int
    var t11: Value; // bool
    var t12: Value; // bool
    var t13: Value; // int
    var t14: Value; // address
    var t15: Value; // bool
    var t16: Value; // bool
    var t17: Value; // int
    var t18: Value; // bytearray
    var t19: Reference; // LibraSystem_BlockMetadata_ref
    var t20: Reference; // bytearray_ref
    var t21: Value; // int
    var t22: Reference; // LibraSystem_BlockMetadata_ref
    var t23: Reference; // int_ref
    var t24: Value; // address
    var t25: Reference; // LibraSystem_BlockMetadata_ref
    var t26: Reference; // address_ref
    var t27: Reference; // LibraSystem_BlockMetadata_ref
    var t28: Reference; // int_ref
    var t29: Value; // int
    var t30: Value; // int
    var t31: Value; // int
    var t32: Reference; // LibraSystem_BlockMetadata_ref
    var t33: Reference; // int_ref

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#ByteArray(arg1);
    assume is#ByteArray(arg2);
    assume is#Address(arg3);

    old_size := m_size;
    m_size := m_size + 34;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size :=  arg2]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size :=  arg3]);

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := BorrowGlobal(contents#Memory(m)[old_size+5], LibraSystem_BlockMetadata);

    call t4 := CopyOrMoveRef(t6);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t8 := CopyOrMoveRef(t4);

    call t9 := BorrowField(t8, LibraSystem_BlockMetadata_timestamp);

    call tmp := ReadRef(t9);
    assume is#Integer(tmp);

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
    assume is#Boolean(t15);

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
    assume is#Integer(tmp);

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
    var t0: Value; // address
    var t1: Reference; // LibraSystem_BlockMetadata_ref
    var t2: Reference; // int_ref
    var t3: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call t1 := BorrowGlobal(contents#Memory(m)[old_size+0], LibraSystem_BlockMetadata);

    call t2 := BorrowField(t1, LibraSystem_BlockMetadata_height);

    call tmp := ReadRef(t2);
    assume is#Integer(tmp);

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
    var t0: Value; // address
    var t1: Reference; // LibraSystem_BlockMetadata_ref
    var t2: Reference; // bytearray_ref
    var t3: Value; // bytearray

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call t1 := BorrowGlobal(contents#Memory(m)[old_size+0], LibraSystem_BlockMetadata);

    call t2 := BorrowField(t1, LibraSystem_BlockMetadata_id);

    call tmp := ReadRef(t2);
    assume is#ByteArray(tmp);

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
    var t0: Value; // address
    var t1: Reference; // LibraSystem_BlockMetadata_ref
    var t2: Reference; // int_ref
    var t3: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call t1 := BorrowGlobal(contents#Memory(m)[old_size+0], LibraSystem_BlockMetadata);

    call t2 := BorrowField(t1, LibraSystem_BlockMetadata_timestamp);

    call tmp := ReadRef(t2);
    assume is#Integer(tmp);

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
    var t0: Value; // address
    var t1: Reference; // LibraSystem_BlockMetadata_ref
    var t2: Reference; // address_ref
    var t3: Value; // address

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 4;

    // bytecode translation starts here
    call tmp := LdAddr(173345816);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call t1 := BorrowGlobal(contents#Memory(m)[old_size+0], LibraSystem_BlockMetadata);

    call t2 := BorrowField(t1, LibraSystem_BlockMetadata_proposer);

    call tmp := ReadRef(t2);
    assume is#Address(tmp);

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
    var t0: Reference; // LibraSystem_ValidatorSet_ref
    var t1: Value; // address
    var t2: Reference; // LibraSystem_ValidatorSet_ref
    var t3: Reference; // LibraSystem_ValidatorSet_ref
    var t4: Reference; // Vector_T_ref
    var t5: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 6;

    // bytecode translation starts here
    call tmp := LdAddr(472);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t2 := BorrowGlobal(contents#Memory(m)[old_size+1], LibraSystem_ValidatorSet);

    call t0 := CopyOrMoveRef(t2);

    call t3 := CopyOrMoveRef(t0);

    call t4 := BorrowField(t3, LibraSystem_ValidatorSet_validators);

    call t5 := Vector_length(t4);
    assume is#Integer(t5);

    m := Memory(domain#Memory(m)[old_size+5 := true], contents#Memory(m)[old_size+5 := t5]);

    ret0 := contents#Memory(m)[old_size+5];
    return;

}

procedure LibraSystem_validator_set_size_verify () returns (ret0: Value)
{
    call ret0 := LibraSystem_validator_set_size();
}

procedure {:inline 1} LibraSystem_is_validator (arg0: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // address
    var t1: Value; // int
    var t2: Value; // int
    var t3: Reference; // Vector_T_ref
    var t4: Reference; // LibraSystem_ValidatorInfo_ref
    var t5: Value; // address
    var t6: Reference; // LibraSystem_ValidatorSet_ref
    var t7: Reference; // Vector_T_ref
    var t8: Reference; // Vector_T_ref
    var t9: Value; // int
    var t10: Value; // int
    var t11: Value; // int
    var t12: Value; // bool
    var t13: Value; // bool
    var t14: Value; // int
    var t15: Reference; // Vector_T_ref
    var t16: Value; // int
    var t17: Reference; // LibraSystem_ValidatorInfo_ref
    var t18: Reference; // LibraSystem_ValidatorInfo_ref
    var t19: Reference; // address_ref
    var t20: Value; // address
    var t21: Value; // address
    var t22: Value; // bool
    var t23: Value; // bool
    var t24: Value; // int
    var t25: Value; // int
    var t26: Value; // int
    var t27: Value; // int
    var t28: Value; // int
    var t29: Value; // bool
    var t30: Reference; // Vector_T_ref
    var t31: Value; // int
    var t32: Reference; // LibraSystem_ValidatorInfo_ref
    var t33: Value; // bool

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 34;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := LdAddr(472);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := BorrowGlobal(contents#Memory(m)[old_size+5], LibraSystem_ValidatorSet);

    call t7 := BorrowField(t6, LibraSystem_ValidatorSet_validators);

    call t3 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t3);

    call t9 := Vector_length(t8);
    assume is#Integer(t9);

    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+10], contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 12];
if (!b#Boolean(tmp)) { goto Label_13; }

    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+13];
    return;

Label_13:
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+14]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t15 := CopyOrMoveRef(t3);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call t17 := Vector_borrow(t15, contents#Memory(m)[old_size+16]);


    call t4 := CopyOrMoveRef(t17);

Label_19:
    call t18 := CopyOrMoveRef(t4);

    call t19 := BorrowField(t18, LibraSystem_ValidatorInfo_addr);

    call tmp := ReadRef(t19);
    assume is#Address(tmp);

    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+20], contents#Memory(m)[old_size+21]);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 22];
if (!b#Boolean(tmp)) { goto Label_27; }

    call tmp := LdTrue();
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+23];
    return;

Label_27:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[24+old_size := true], contents#Memory(m)[24+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[25+old_size := true], contents#Memory(m)[25+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+24], contents#Memory(m)[old_size+25]);
    m := Memory(domain#Memory(m)[26+old_size := true], contents#Memory(m)[26+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+26]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[27+old_size := true], contents#Memory(m)[27+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[28+old_size := true], contents#Memory(m)[28+old_size := tmp]);

    call tmp := Ge(contents#Memory(m)[old_size+27], contents#Memory(m)[old_size+28]);
    m := Memory(domain#Memory(m)[29+old_size := true], contents#Memory(m)[29+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 29];
if (!b#Boolean(tmp)) { goto Label_36; }

    goto Label_41;

Label_36:
    call t30 := CopyOrMoveRef(t3);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[31+old_size := true], contents#Memory(m)[31+old_size := tmp]);

    call t32 := Vector_borrow(t30, contents#Memory(m)[old_size+31]);


    call t4 := CopyOrMoveRef(t32);

    goto Label_19;

Label_41:
    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[33+old_size := true], contents#Memory(m)[33+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+33];
    return;

}

procedure LibraSystem_is_validator_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := LibraSystem_is_validator(arg0);
}

procedure {:inline 1} LibraSystem_add_validator (arg0: Value) returns ()
{
    // declare local variables
    var t0: Value; // address
    var t1: Reference; // LibraSystem_ValidatorSet_ref
    var t2: Value; // address
    var t3: Value; // bool
    var t4: Value; // bool
    var t5: Value; // int
    var t6: Value; // address
    var t7: Reference; // LibraSystem_ValidatorSet_ref
    var t8: Reference; // LibraSystem_ValidatorSet_ref
    var t9: Reference; // Vector_T_ref
    var t10: Value; // address
    var t11: Value; // bytearray
    var t12: Value; // int
    var t13: Value; // bytearray
    var t14: Value; // bytearray
    var t15: Value; // LibraSystem_ValidatorInfo

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 16;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := ValidatorConfig_has(contents#Memory(m)[old_size+2]);
    assume is#Boolean(t3);

    m := Memory(domain#Memory(m)[old_size+3 := true], contents#Memory(m)[old_size+3 := t3]);

    call tmp := Not(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 4];
if (!b#Boolean(tmp)) { goto Label_6; }

    call tmp := LdConst(17);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    assert false;

Label_6:
    call tmp := LdAddr(472);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call t7 := BorrowGlobal(contents#Memory(m)[old_size+6], LibraSystem_ValidatorSet);

    call t1 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t1);

    call t9 := BorrowField(t8, LibraSystem_ValidatorSet_validators);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    // unimplemented instruction

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    // unimplemented instruction

    // unimplemented instruction

    assume is#Address(contents#Memory(m)[old_size+10]);

    assume is#ByteArray(contents#Memory(m)[old_size+11]);

    assume is#Integer(contents#Memory(m)[old_size+12]);

    assume is#ByteArray(contents#Memory(m)[old_size+13]);

    assume is#ByteArray(contents#Memory(m)[old_size+14]);

    call tmp := Pack_LibraSystem_ValidatorInfo(contents#Memory(m)[old_size+10], contents#Memory(m)[old_size+11], contents#Memory(m)[old_size+12], contents#Memory(m)[old_size+13], contents#Memory(m)[old_size+14]);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call Vector_push_back(t9, contents#Memory(m)[old_size+15]);

    return;

}

procedure LibraSystem_add_validator_verify (arg0: Value) returns ()
{
    call LibraSystem_add_validator(arg0);
}

procedure {:inline 1} LibraSystem_copy_validator_info (arg0: Reference) returns (ret0: Value)
{
    // declare local variables
    var t0: Reference; // LibraSystem_ValidatorInfo_ref
    var t1: Value; // bytearray
    var t2: Value; // bytearray
    var t3: Value; // bytearray
    var t4: Value; // ValidatorConfig_Config
    var t5: Value; // bool
    var t6: Reference; // LibraSystem_ValidatorInfo_ref
    var t7: Reference; // address_ref
    var t8: Value; // address
    var t9: Value; // ValidatorConfig_Config
    var t10: Reference; // ValidatorConfig_Config_ref
    var t11: Value; // bytearray
    var t12: Reference; // ValidatorConfig_Config_ref
    var t13: Value; // bytearray
    var t14: Reference; // ValidatorConfig_Config_ref
    var t15: Value; // bytearray
    var t16: Value; // bool
    var t17: Reference; // bytearray_ref
    var t18: Reference; // LibraSystem_ValidatorInfo_ref
    var t19: Reference; // bytearray_ref
    var t20: Value; // bool
    var t21: Value; // bytearray
    var t22: Reference; // LibraSystem_ValidatorInfo_ref
    var t23: Reference; // bytearray_ref
    var t24: Value; // bool
    var t25: Reference; // bytearray_ref
    var t26: Reference; // LibraSystem_ValidatorInfo_ref
    var t27: Reference; // bytearray_ref
    var t28: Value; // bool
    var t29: Value; // bytearray
    var t30: Reference; // LibraSystem_ValidatorInfo_ref
    var t31: Reference; // bytearray_ref
    var t32: Value; // bool
    var t33: Reference; // bytearray_ref
    var t34: Reference; // LibraSystem_ValidatorInfo_ref
    var t35: Reference; // bytearray_ref
    var t36: Value; // bool
    var t37: Value; // bytearray
    var t38: Reference; // LibraSystem_ValidatorInfo_ref
    var t39: Reference; // bytearray_ref
    var t40: Value; // bool
    var t41: Value; // bool

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
    assume is#Address(tmp);

    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call t9 := ValidatorConfig_config(contents#Memory(m)[old_size+8]);
    assume is#Map(t9);

    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call t10 := BorrowLoc(old_size+4);

    call t11 := ValidatorConfig_consensus_pubkey(t10);
    assume is#ByteArray(t11);

    m := Memory(domain#Memory(m)[old_size+11 := true], contents#Memory(m)[old_size+11 := t11]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t12 := BorrowLoc(old_size+4);

    call t13 := ValidatorConfig_network_signing_pubkey(t12);
    assume is#ByteArray(t13);

    m := Memory(domain#Memory(m)[old_size+13 := true], contents#Memory(m)[old_size+13 := t13]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+13]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t14 := BorrowLoc(old_size+4);

    call t15 := ValidatorConfig_network_identity_pubkey(t14);
    assume is#ByteArray(t15);

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

    call tmp := Neq(contents#Memory(m)[old_size+17], contents#Memory(m)[old_size+19]);
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

    call tmp := Neq(contents#Memory(m)[old_size+25], contents#Memory(m)[old_size+27]);
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

    call tmp := Neq(contents#Memory(m)[old_size+33], contents#Memory(m)[old_size+35]);
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

procedure {:inline 1} LibraSystem_reconfigure () returns ()
{
    // declare local variables
    var t0: Reference; // LibraSystem_ValidatorSet_ref
    var t1: Reference; // Vector_T_ref
    var t2: Reference; // LibraSystem_ValidatorInfo_ref
    var t3: Value; // int
    var t4: Value; // int
    var t5: Value; // bool
    var t6: Value; // address
    var t7: Reference; // LibraSystem_ValidatorSet_ref
    var t8: Reference; // LibraSystem_ValidatorSet_ref
    var t9: Reference; // Vector_T_ref
    var t10: Value; // int
    var t11: Reference; // Vector_T_ref
    var t12: Reference; // Vector_T_ref
    var t13: Value; // int
    var t14: Value; // bool
    var t15: Reference; // Vector_T_ref
    var t16: Value; // int
    var t17: Reference; // LibraSystem_ValidatorInfo_ref
    var t18: Reference; // LibraSystem_ValidatorInfo_ref
    var t19: Value; // bool
    var t20: Value; // bool
    var t21: Value; // int
    var t22: Value; // int
    var t23: Value; // int
    var t24: Value; // int
    var t25: Value; // int
    var t26: Value; // bool
    var t27: Reference; // Vector_T_ref
    var t28: Value; // int
    var t29: Reference; // LibraSystem_ValidatorInfo_ref
    var t30: Value; // bool
    var t31: Reference; // LibraSystem_ValidatorSet_ref
    var t32: Reference; // LibraAccount_EventHandle_ref
    var t33: Reference; // Vector_T_ref
    var t34: Value; // Vector_T
    var t35: Value; // LibraSystem_ValidatorSetChangeEvent

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 36;

    // bytecode translation starts here
    call tmp := LdAddr(472);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call t7 := BorrowGlobal(contents#Memory(m)[old_size+6], LibraSystem_ValidatorSet);

    call t0 := CopyOrMoveRef(t7);

    call t8 := CopyOrMoveRef(t0);

    call t9 := BorrowField(t8, LibraSystem_ValidatorSet_validators);

    call t1 := CopyOrMoveRef(t9);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+10]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t11 := CopyOrMoveRef(t1);

    call t12 := FreezeRef(t11);

    call t13 := Vector_length(t12);
    assume is#Integer(t13);

    m := Memory(domain#Memory(m)[old_size+13 := true], contents#Memory(m)[old_size+13 := t13]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+13]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+14]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t15 := CopyOrMoveRef(t1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call t17 := Vector_borrow_mut(t15, contents#Memory(m)[old_size+16]);


    call t2 := CopyOrMoveRef(t17);

Label_18:
    call t18 := CopyOrMoveRef(t2);

    call t19 := LibraSystem_copy_validator_info(t18);
    assume is#Boolean(t19);

    m := Memory(domain#Memory(m)[old_size+19 := true], contents#Memory(m)[old_size+19 := t19]);

    tmp := contents#Memory(m)[old_size + 19];
if (!b#Boolean(tmp)) { goto Label_23; }

    call tmp := LdTrue();
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+20]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

Label_23:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    call tmp := Add(contents#Memory(m)[old_size+21], contents#Memory(m)[old_size+22]);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+23]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[24+old_size := true], contents#Memory(m)[24+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[25+old_size := true], contents#Memory(m)[25+old_size := tmp]);

    call tmp := Ge(contents#Memory(m)[old_size+24], contents#Memory(m)[old_size+25]);
    m := Memory(domain#Memory(m)[26+old_size := true], contents#Memory(m)[26+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 26];
if (!b#Boolean(tmp)) { goto Label_32; }

    goto Label_37;

Label_32:
    call t27 := CopyOrMoveRef(t1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[28+old_size := true], contents#Memory(m)[28+old_size := tmp]);

    call t29 := Vector_borrow_mut(t27, contents#Memory(m)[old_size+28]);


    call t2 := CopyOrMoveRef(t29);

    goto Label_18;

Label_37:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[30+old_size := true], contents#Memory(m)[30+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 30];
if (!b#Boolean(tmp)) { goto Label_46; }

    call t31 := CopyOrMoveRef(t0);

    call t32 := BorrowField(t31, LibraSystem_ValidatorSet_change_events);

    call t33 := CopyOrMoveRef(t1);

    call tmp := ReadRef(t33);
    assume is#Vector(tmp);

    m := Memory(domain#Memory(m)[34+old_size := true], contents#Memory(m)[34+old_size := tmp]);

    assume is#Vector(contents#Memory(m)[old_size+34]);

    call tmp := Pack_LibraSystem_ValidatorSetChangeEvent(contents#Memory(m)[old_size+34]);
    m := Memory(domain#Memory(m)[35+old_size := true], contents#Memory(m)[35+old_size := tmp]);

    call LibraAccount_emit_event(t32, contents#Memory(m)[old_size+35]);

    return;

Label_46:
    return;

}

procedure LibraSystem_reconfigure_verify () returns ()
{
    call LibraSystem_reconfigure();
}

procedure {:inline 1} LibraSystem_get_ith_validator_info (arg0: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // int
    var t1: Reference; // Vector_T_ref
    var t2: Value; // address
    var t3: Reference; // LibraSystem_ValidatorSet_ref
    var t4: Reference; // Vector_T_ref
    var t5: Value; // int
    var t6: Reference; // Vector_T_ref
    var t7: Value; // int
    var t8: Value; // bool
    var t9: Value; // bool
    var t10: Value; // int
    var t11: Reference; // Vector_T_ref
    var t12: Value; // int
    var t13: Reference; // LibraSystem_ValidatorInfo_ref
    var t14: Value; // LibraSystem_ValidatorInfo

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Integer(arg0);

    old_size := m_size;
    m_size := m_size + 15;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := LdAddr(472);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := BorrowGlobal(contents#Memory(m)[old_size+2], LibraSystem_ValidatorSet);

    call t4 := BorrowField(t3, LibraSystem_ValidatorSet_validators);

    call t1 := CopyOrMoveRef(t4);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := CopyOrMoveRef(t1);

    call t7 := Vector_length(t6);
    assume is#Integer(t7);

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

    call t13 := Vector_borrow(t11, contents#Memory(m)[old_size+12]);


    call tmp := ReadRef(t13);
    assume is#Map(tmp);

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
    var t0: Value; // int
    var t1: Value; // int
    var t2: Reference; // LibraSystem_ValidatorSet_ref
    var t3: Value; // address
    var t4: Reference; // LibraSystem_ValidatorInfo_ref
    var t5: Value; // address
    var t6: Reference; // LibraSystem_ValidatorSet_ref
    var t7: Reference; // LibraSystem_ValidatorSet_ref
    var t8: Reference; // Vector_T_ref
    var t9: Value; // int
    var t10: Value; // int
    var t11: Value; // int
    var t12: Value; // bool
    var t13: Value; // bool
    var t14: Value; // int
    var t15: Reference; // LibraSystem_ValidatorSet_ref
    var t16: Reference; // Vector_T_ref
    var t17: Value; // int
    var t18: Reference; // LibraSystem_ValidatorInfo_ref
    var t19: Reference; // LibraSystem_ValidatorInfo_ref
    var t20: Reference; // address_ref
    var t21: Value; // address
    var t22: Value; // address

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Integer(arg0);

    old_size := m_size;
    m_size := m_size + 23;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := LdAddr(472);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := BorrowGlobal(contents#Memory(m)[old_size+5], LibraSystem_ValidatorSet);

    call t2 := CopyOrMoveRef(t6);

    call t7 := CopyOrMoveRef(t2);

    call t8 := BorrowField(t7, LibraSystem_ValidatorSet_validators);

    call t9 := Vector_length(t8);
    assume is#Integer(t9);

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

    call t18 := Vector_borrow(t16, contents#Memory(m)[old_size+17]);


    call t4 := CopyOrMoveRef(t18);

    call t19 := CopyOrMoveRef(t4);

    call t20 := BorrowField(t19, LibraSystem_ValidatorInfo_addr);

    call tmp := ReadRef(t20);
    assume is#Address(tmp);

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
    var t0: Value; // int
    var t1: Value; // int
    var t2: Value; // LibraCoin_T
    var t3: Value; // int
    var t4: Reference; // TransactionFeeDistribution_T_ref
    var t5: Value; // address
    var t6: Reference; // TransactionFeeDistribution_T_ref
    var t7: Value; // int
    var t8: Value; // address
    var t9: Value; // int
    var t10: Reference; // TransactionFeeDistribution_T_ref
    var t11: Reference; // LibraAccount_WithdrawalCapability_ref
    var t12: Value; // int
    var t13: Value; // LibraCoin_T
    var t14: Value; // int
    var t15: Value; // int
    var t16: Value; // int
    var t17: Value; // LibraCoin_T
    var t18: Value; // int
    var t19: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 20;

    // bytecode translation starts here
    call tmp := LdAddr(4078);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := BorrowGlobal(contents#Memory(m)[old_size+5], TransactionFeeDistribution_T);

    call t4 := CopyOrMoveRef(t6);

    call t7 := LibraSystem_validator_set_size();
    assume is#Integer(t7);

    m := Memory(domain#Memory(m)[old_size+7 := true], contents#Memory(m)[old_size+7 := t7]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := LdAddr(4078);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call t9 := LibraAccount_balance(contents#Memory(m)[old_size+8]);
    assume is#Integer(t9);

    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call t10 := CopyOrMoveRef(t4);

    call t11 := BorrowField(t10, TransactionFeeDistribution_T_fee_withdrawal_capability);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call t13 := LibraAccount_withdraw_with_capability(t11, contents#Memory(m)[old_size+12]);
    assume is#Map(t13);

    m := Memory(domain#Memory(m)[old_size+13 := true], contents#Memory(m)[old_size+13 := t13]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+13]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    call t16 := TransactionFeeDistribution_per_validator_distribution_amount(contents#Memory(m)[old_size+14], contents#Memory(m)[old_size+15]);
    assume is#Integer(t16);

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
    var t0: Value; // address
    var t1: Value; // address
    var t2: Value; // bool
    var t3: Value; // bool
    var t4: Value; // int
    var t5: Value; // int
    var t6: Value; // LibraAccount_WithdrawalCapability
    var t7: Value; // TransactionFeeDistribution_T

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

    call tmp := Eq(contents#Memory(m)[old_size+0], contents#Memory(m)[old_size+1]);
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
    assume is#Map(t6);

    m := Memory(domain#Memory(m)[old_size+6 := true], contents#Memory(m)[old_size+6 := t6]);

    assume is#Integer(contents#Memory(m)[old_size+5]);

    assume is#Map(contents#Memory(m)[old_size+6]);

    call tmp := Pack_TransactionFeeDistribution_T(contents#Memory(m)[old_size+5], contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call MoveToSender(TransactionFeeDistribution_T, contents#Memory(m)[old_size+7]);

    return;

}

procedure TransactionFeeDistribution_initialize_verify () returns ()
{
    call TransactionFeeDistribution_initialize();
}

procedure {:inline 1} TransactionFeeDistribution_distribute_transaction_fees_internal (arg0: Value, arg1: Value, arg2: Value) returns ()
{
    // declare local variables
    var t0: Value; // LibraCoin_T
    var t1: Value; // int
    var t2: Value; // int
    var t3: Value; // int
    var t4: Value; // address
    var t5: Value; // LibraCoin_T
    var t6: Value; // int
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // bool
    var t10: Value; // int
    var t11: Value; // address
    var t12: Value; // int
    var t13: Value; // int
    var t14: Value; // int
    var t15: Value; // LibraCoin_T
    var t16: Value; // int
    var t17: Value; // LibraCoin_T
    var t18: Value; // LibraCoin_T
    var t19: Value; // address
    var t20: Value; // LibraCoin_T
    var t21: Reference; // LibraCoin_T_ref
    var t22: Value; // int
    var t23: Value; // int
    var t24: Value; // bool
    var t25: Value; // LibraCoin_T
    var t26: Value; // address
    var t27: Value; // LibraCoin_T

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Map(arg0);
    assume is#Integer(arg1);
    assume is#Integer(arg2);

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
    assume is#Address(t11);

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
    assume is#Map(t18);

    assume is#Map(t17);

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
    assume is#Integer(t22);

    m := Memory(domain#Memory(m)[old_size+22 := true], contents#Memory(m)[old_size+22 := t22]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+22], contents#Memory(m)[old_size+23]);
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
    var t0: Value; // int
    var t1: Value; // int
    var t2: Value; // int
    var t3: Value; // int
    var t4: Value; // int
    var t5: Value; // bool
    var t6: Value; // bool
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // int
    var t10: Value; // int
    var t11: Value; // int
    var t12: Value; // int
    var t13: Value; // int
    var t14: Value; // int
    var t15: Value; // bool
    var t16: Value; // bool
    var t17: Value; // int
    var t18: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#Integer(arg1);

    old_size := m_size;
    m_size := m_size + 19;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := Neq(contents#Memory(m)[old_size+3], contents#Memory(m)[old_size+4]);
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
