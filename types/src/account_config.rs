// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_address::AccountAddress,
    event::EventHandle,
    language_storage::{ModuleId, ResourceKey, StructTag, TypeTag},
    move_resource::MoveResource,
};
use anyhow::Result;
use move_core_types::identifier::{IdentStr, Identifier};
use once_cell::sync::Lazy;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

pub const LBR_NAME: &str = "LBR";

pub static LBR_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, Identifier::new(LBR_NAME).unwrap()));
pub static LBR_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

const ACCOUNT_MODULE_NAME: &str = "LibraAccount";

// Libra
static COIN_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("Libra").unwrap());
static COIN_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());
pub static COIN_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, COIN_MODULE_NAME.clone()));

// Account
static ACCOUNT_MODULE_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("LibraAccount").unwrap());
static ACCOUNT_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());
static ACCOUNT_BALANCE_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("Balance").unwrap());
static ACCOUNT_TYPE_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("AccountType").unwrap());
pub static ACCOUNT_TYPE_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, ACCOUNT_TYPE_MODULE_NAME.clone()));
pub static ACCOUNT_TYPE_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

static UNHOSTED_TYPE_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("Unhosted").unwrap());
pub static UNHOSTED_TYPE_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, UNHOSTED_TYPE_MODULE_NAME.clone()));
pub static UNHOSTED_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

static EMPTY_ACCOUNT_TYPE_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("Empty").unwrap());
pub static EMPTY_ACCOUNT_TYPE_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, EMPTY_ACCOUNT_TYPE_MODULE_NAME.clone()));
pub static EMPTY_ACCOUNT_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("T").unwrap());

static GENESIS_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("Genesis").unwrap());
pub static GENESIS_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, GENESIS_MODULE_NAME.clone()));

static ACCOUNT_LIMITS_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("AccountLimits").unwrap());
pub static ACCOUNT_LIMITS_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, ACCOUNT_LIMITS_MODULE_NAME.clone()));
pub static ACCOUNT_LIMITS_WINDOW_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("Window").unwrap());

/// The ModuleId for the Account module.
pub static ACCOUNT_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, ACCOUNT_MODULE_IDENTIFIER.clone()));

// Debug
pub static DEBUG_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("Debug").unwrap());

pub static DEBUG_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, DEBUG_MODULE_NAME.clone()));
static EVENT_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("Event").unwrap());
pub static EVENT_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, EVENT_MODULE_NAME.clone()));

static EVENT_HANDLE_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("EventHandle").unwrap());
static EVENT_HANDLE_GENERATOR_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("EventHandleGenerator").unwrap());

// Payment Events
static SENT_EVENT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("SentPaymentEvent").unwrap());
static RECEIVED_EVENT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("ReceivedPaymentEvent").unwrap());

pub fn account_type_module_name() -> &'static IdentStr {
    &*ACCOUNT_TYPE_MODULE_NAME
}

pub fn account_type_struct_name() -> &'static IdentStr {
    &*ACCOUNT_TYPE_STRUCT_NAME
}

pub fn account_limits_module_name() -> &'static IdentStr {
    &*ACCOUNT_LIMITS_MODULE_NAME
}

pub fn account_limits_window_struct_name() -> &'static IdentStr {
    &*ACCOUNT_LIMITS_WINDOW_STRUCT_NAME
}

pub fn account_balance_struct_name() -> &'static IdentStr {
    &*ACCOUNT_BALANCE_STRUCT_NAME
}

pub fn unhosted_type_module_name() -> &'static IdentStr {
    &*UNHOSTED_TYPE_MODULE_NAME
}

pub fn unhosted_type_struct_name() -> &'static IdentStr {
    &*UNHOSTED_STRUCT_NAME
}

pub fn empty_account_type_module_name() -> &'static IdentStr {
    &*EMPTY_ACCOUNT_TYPE_MODULE_NAME
}

pub fn empty_account_type_struct_name() -> &'static IdentStr {
    &*EMPTY_ACCOUNT_STRUCT_NAME
}

pub fn coin_module_name() -> &'static IdentStr {
    &*COIN_MODULE_NAME
}

pub fn coin_struct_name() -> &'static IdentStr {
    &*COIN_STRUCT_NAME
}

pub fn sent_event_name() -> &'static IdentStr {
    &*SENT_EVENT_NAME
}

pub fn received_event_name() -> &'static IdentStr {
    &*RECEIVED_EVENT_NAME
}

pub fn event_module_name() -> &'static IdentStr {
    &*EVENT_MODULE_NAME
}

pub fn event_handle_generator_struct_name() -> &'static IdentStr {
    &*EVENT_HANDLE_GENERATOR_STRUCT_NAME
}

pub fn event_handle_struct_name() -> &'static IdentStr {
    &*EVENT_HANDLE_STRUCT_NAME
}

pub const CORE_CODE_ADDRESS: AccountAddress = AccountAddress::DEFAULT;

pub fn association_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xA550C18")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn transaction_fee_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xFEE")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn validator_set_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0x1D8")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn discovery_set_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xD15C0")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn account_type_struct_tag(is_empty_account_type: bool) -> StructTag {
    let inner_struct_tag = if is_empty_account_type {
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: empty_account_type_module_name().to_owned(),
            type_params: vec![],
            name: empty_account_type_struct_name().to_owned(),
        }
    } else {
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: unhosted_type_module_name().to_owned(),
            type_params: vec![],
            name: unhosted_type_struct_name().to_owned(),
        }
    };
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: account_type_module_name().to_owned(),
        type_params: vec![TypeTag::Struct(inner_struct_tag)],
        name: account_type_struct_name().to_owned(),
    }
}

pub fn account_struct_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: ACCOUNT_MODULE_IDENTIFIER.clone(),
        name: ACCOUNT_STRUCT_NAME.to_owned(),
        type_params: vec![],
    }
}

pub fn lbr_type_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: from_currency_code_string(LBR_NAME).unwrap(),
        name: coin_struct_name().to_owned(),
        type_params: vec![],
    })
}

// TODO: This imposes a few implied restrictions:
//   1) The currency module must be published under the core code address.
//   2) The module name must be the same as the gas specifier.
//   3) The struct name must be "T"
// We need to consider whether we want to switch to a more or fully qualified name.
pub fn account_balance_struct_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: ACCOUNT_MODULE_IDENTIFIER.to_owned(),
        name: account_balance_struct_name().to_owned(),
        type_params: vec![lbr_type_tag()],
    }
}

pub fn sent_payment_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: ACCOUNT_MODULE_IDENTIFIER.clone(),
        name: sent_event_name().to_owned(),
        type_params: vec![],
    }
}

pub fn received_payment_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: ACCOUNT_MODULE_IDENTIFIER.clone(),
        name: received_event_name().to_owned(),
        type_params: vec![],
    }
}

pub fn event_handle_generator_struct_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: event_module_name().to_owned(),
        name: event_handle_generator_struct_name().to_owned(),
        type_params: vec![],
    }
}

pub fn type_tag_for_currency_code(currency_code: Identifier) -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: currency_code,
        name: coin_struct_name().to_owned(),
        type_params: vec![],
    })
}

pub fn from_currency_code_string(currency_code_string: &str) -> Result<Identifier> {
    Identifier::new(currency_code_string)
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AssociationCapabilityResource {
    is_certified: bool,
}

impl AssociationCapabilityResource {
    pub fn is_certified(&self) -> bool {
        self.is_certified
    }
}

/// A Rust representation of an Account resource.
/// This is not how the Account is represented in the VM but it's a convenient representation.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AccountResource {
    authentication_key: Vec<u8>,
    delegated_key_rotation_capability: bool,
    delegated_withdrawal_capability: bool,
    received_events: EventHandle,
    sent_events: EventHandle,
    sequence_number: u64,
    is_frozen: bool,
    balance_currency_code: Identifier,
}

impl AccountResource {
    /// Constructs an Account resource.
    pub fn new(
        sequence_number: u64,
        authentication_key: Vec<u8>,
        delegated_key_rotation_capability: bool,
        delegated_withdrawal_capability: bool,
        sent_events: EventHandle,
        received_events: EventHandle,
        is_frozen: bool,
        balance_currency_code: Identifier,
    ) -> Self {
        AccountResource {
            sequence_number,
            authentication_key,
            delegated_key_rotation_capability,
            delegated_withdrawal_capability,
            sent_events,
            received_events,
            is_frozen,
            balance_currency_code,
        }
    }

    /// Return the sequence_number field for the given AccountResource
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Return the authentication_key field for the given AccountResource
    pub fn authentication_key(&self) -> &[u8] {
        &self.authentication_key
    }

    /// Return the sent_events handle for the given AccountResource
    pub fn sent_events(&self) -> &EventHandle {
        &self.sent_events
    }

    /// Return the received_events handle for the given AccountResource
    pub fn received_events(&self) -> &EventHandle {
        &self.received_events
    }

    /// Return the delegated_key_rotation_capability field for the given AccountResource
    pub fn delegated_key_rotation_capability(&self) -> bool {
        self.delegated_key_rotation_capability
    }

    /// Return the delegated_withdrawal_capability field for the given AccountResource
    pub fn delegated_withdrawal_capability(&self) -> bool {
        self.delegated_withdrawal_capability
    }

    /// Return the the is_frozen flag for the given AccountResource
    pub fn is_frozen(&self) -> bool {
        self.is_frozen
    }

    /// Return the currency code for the currency held by this account
    pub fn balance_currency_code(&self) -> &IdentStr {
        &self.balance_currency_code
    }
}

impl MoveResource for AccountResource {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "T";
}

/// The balance resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct BalanceResource {
    coin: u64,
}

impl BalanceResource {
    pub fn new(coin: u64) -> Self {
        Self { coin }
    }

    pub fn coin(&self) -> u64 {
        self.coin
    }

    // TODO/XXX: remove this once the MoveResource trait allows type arguments to `struct_tag`.
    pub fn struct_tag_for_currency(currency_typetag: TypeTag) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            name: BalanceResource::struct_identifier(),
            module: BalanceResource::module_identifier(),
            type_params: vec![currency_typetag],
        }
    }

    // TODO: remove this once the MoveResource trait allows type arguments to `resource_path`.
    pub fn access_path_for(currency_typetag: TypeTag) -> Vec<u8> {
        AccessPath::resource_access_vec(
            &BalanceResource::struct_tag_for_currency(currency_typetag),
            &Accesses::empty(),
        )
    }
}

impl MoveResource for BalanceResource {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "Balance";

    fn type_params() -> Vec<TypeTag> {
        vec![lbr_type_tag()]
    }
}

/// The path to the sent event counter for an Account resource.
/// It can be used to query the event DB for the given event.
pub static ACCOUNT_SENT_EVENT_PATH: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut path = AccountResource::resource_path();
    path.extend_from_slice(b"/sent_events_count/");
    path
});

/// Returns the path to the received event counter for an Account resource.
/// It can be used to query the event DB for the given event.
pub static ACCOUNT_RECEIVED_EVENT_PATH: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut path = AccountResource::resource_path();
    path.extend_from_slice(b"/received_events_count/");
    path
});

/// Struct that represents a SentPaymentEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct SentPaymentEvent {
    amount: u64,
    currency_code: Identifier,
    receiver: AccountAddress,
    metadata: Vec<u8>,
}

impl SentPaymentEvent {
    // TODO: should only be used for libra client testing and be removed eventually
    pub fn new(
        amount: u64,
        currency_code: Identifier,
        receiver: AccountAddress,
        metadata: Vec<u8>,
    ) -> Self {
        Self {
            amount,
            currency_code,
            receiver,
            metadata,
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }

    /// Get the sender of this transaction event.
    pub fn receiver(&self) -> AccountAddress {
        self.receiver
    }

    /// Get the amount sent or received
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Get the metadata associated with this event
    pub fn metadata(&self) -> &Vec<u8> {
        &self.metadata
    }

    /// Return the currency currency_code symbol that the payment was made in.
    pub fn currency_code(&self) -> &IdentStr {
        &self.currency_code
    }
}

impl MoveResource for SentPaymentEvent {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "SentPaymentEvent";
}

/// Struct that represents a ReceivedPaymentEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReceivedPaymentEvent {
    amount: u64,
    currency_code: Identifier,
    sender: AccountAddress,
    metadata: Vec<u8>,
}

impl ReceivedPaymentEvent {
    // TODO: should only be used for libra client testing and be removed eventually
    pub fn new(
        amount: u64,
        currency_code: Identifier,
        sender: AccountAddress,
        metadata: Vec<u8>,
    ) -> Self {
        Self {
            amount,
            currency_code,
            sender,
            metadata,
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }

    /// Get the receiver of this transaction event.
    pub fn sender(&self) -> AccountAddress {
        self.sender
    }

    /// Get the amount sent or received
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Get the metadata associated with this event
    pub fn metadata(&self) -> &Vec<u8> {
        &self.metadata
    }

    /// Return the currency code that the payment was made in.
    pub fn currency_code(&self) -> &IdentStr {
        &self.currency_code
    }
}

impl MoveResource for ReceivedPaymentEvent {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "ReceivedPaymentEvent";
}

/// Struct that represents a ReceivedPaymentEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct CurrencyInfoResource {
    total_value: u128,
    preburn_value: u64,
    to_lbr_exchange_rate: u64,
    is_synthetic: bool,
    scaling_factor: u64,
    fractional_part: u64,
    currency_code: Identifier,
    can_mint: bool,
}

impl MoveResource for CurrencyInfoResource {
    const MODULE_NAME: &'static str = "Libra";
    const STRUCT_NAME: &'static str = "CurrencyInfo";
}

impl CurrencyInfoResource {
    pub fn currency_code(&self) -> &IdentStr {
        &self.currency_code
    }

    pub fn scaling_factor(&self) -> u64 {
        self.scaling_factor
    }

    pub fn fractional_part(&self) -> u64 {
        self.fractional_part
    }

    pub fn convert_to_lbr(&self, amount: u64) -> u64 {
        let mut mult = (amount as u128) * (self.scaling_factor as u128);
        mult >>= 32;
        mult as u64
    }

    pub fn struct_tag_for(currency_code: Identifier) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: CurrencyInfoResource::module_identifier(),
            name: CurrencyInfoResource::struct_identifier(),
            type_params: vec![type_tag_for_currency_code(currency_code)],
        }
    }

    pub fn resource_path_for(currency_code: Identifier) -> AccessPath {
        let resource_key = ResourceKey::new(
            association_address(),
            CurrencyInfoResource::struct_tag_for(currency_code),
        );
        AccessPath::resource_access_path(&resource_key, &Accesses::empty())
    }

    pub fn access_path_for(currency_code: Identifier) -> Vec<u8> {
        AccessPath::resource_access_vec(
            &CurrencyInfoResource::struct_tag_for(currency_code),
            &Accesses::empty(),
        )
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }
}
