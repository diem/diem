// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{
    de::{Deserializer, Error},
    Deserialize, Serialize,
};
use serde_repr::{Deserialize_repr, Serialize_repr};
use uuid::Uuid;

use crate::payment_command::Actor;

/// A header set with a unique UUID (according to RFC4122 with "-"'s included) for the request,
/// used for tracking requests and debugging. Responses must have the same string in the
/// X-REQUEST-ID header value as the requests they correspond to.
pub const REQUEST_ID_HEADER: &str = "X-REQUEST-ID";

/// A header with the HTTP request sender's VASP DIP-5 address used in the command object. The HTTP
/// request sender must use the compliance key of the VASP account linked with this address to sign
/// the request JWS body, and the request receiver uses this address to find the request sender's
/// compliance key to verify the JWS signature. For example: VASP A transfers funds to VASP B. The
/// HTTP request A sends to B contains X-REQUEST-SENDER-ADDRESS as VASP A's address. An HTTP
/// request B sends to A should contain VASP B's address as X-REQUEST-SENDER-ADDRESS.
pub const REQUEST_SENDER_ADDRESS: &str = "X-REQUEST-SENDER-ADDRESS";

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
enum ObjectType {
    CommandRequestObject,
    CommandResponseObject,
    PaymentCommand,
}

impl ObjectType {
    fn deserialize_request<'de, D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::deserialize_variant(d, Self::CommandRequestObject)
    }

    fn deserialize_response<'de, D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::deserialize_variant(d, Self::CommandResponseObject)
    }

    fn deserialize_payment<'de, D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::deserialize_variant(d, Self::PaymentCommand)
    }

    fn deserialize_variant<'de, D: Deserializer<'de>>(
        d: D,
        variant: Self,
    ) -> Result<Self, D::Error> {
        let object_type = Self::deserialize(d)?;

        if object_type == variant {
            Ok(object_type)
        } else {
            Err(D::Error::custom(format_args!("expected {:?}", variant)))
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CommandRequestObject {
    #[serde(deserialize_with = "ObjectType::deserialize_request")]
    #[serde(rename = "_ObjectType")]
    object_type: ObjectType,
    #[serde(flatten)]
    command: Command,
    cid: Uuid,
}

impl CommandRequestObject {
    pub fn new(command: Command, cid: Uuid) -> Self {
        Self {
            object_type: ObjectType::CommandRequestObject,
            command,
            cid,
        }
    }

    pub fn command(&self) -> &Command {
        &self.command
    }

    pub fn cid(&self) -> Uuid {
        self.cid
    }

    pub fn into_parts(self) -> (Command, Uuid) {
        (self.command, self.cid)
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    Success,
    Failure,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct CommandResponseObject {
    #[serde(deserialize_with = "ObjectType::deserialize_response")]
    #[serde(rename = "_ObjectType")]
    object_type: ObjectType,
    status: CommandStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<OffChainError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cid: Option<Uuid>,
}

impl CommandResponseObject {
    pub fn new(status: CommandStatus) -> Self {
        Self {
            object_type: ObjectType::CommandResponseObject,
            status,
            error: None,
            cid: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum OffChainErrorType {
    #[serde(rename = "command_error")]
    Command,
    #[serde(rename = "protocol_error")]
    Protocol,
}

// https://dip.diem.com/dip-1/#list-of-error-codes
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    //
    // HTTP Header Validation Error Codes
    //
    /// One of the following potential errors:
    /// * `X-REQUEST-SENDER-ADDRESS` header value is not the request sender’s address in the
    ///    command object. All command objects should have a field that is the request sender’s
    ///    address.
    /// * Could not find Diem's onchain account by the `X-REQUEST-SENDER-ADDRESS` header value.
    /// * Could not find the compliance key of the onchain account found by the
    ///   `X-REQUEST-SENDER-ADDRESS` header value.
    /// * The compliance key found from the onchain account by `X-REQUEST-SENDER-ADDRESS` is not a
    ///   valid ED25519 public key.
    /// * `X-REQUEST-ID` is not a valid UUID format.
    InvalidHttpHeader,

    /// Missing HTTP header `X-REQUEST-ID` or `X-REQUEST-SENDER-ADDRESS`.
    MissingHttpHeader,

    //
    // JWS Validation Error Codes#
    //
    /// Invalid JWS format (compact) or protected header
    InvalidJws,

    /// JWS signature verification failed
    InvalidJwsSignature,

    //
    // Request Object Validation Error Codes#
    //
    /// Request content is not valid Json
    InvalidJson,

    /// Object is not valid, type does not match
    /// The Command request/response object json is not an object, or the command object type does
    /// not match command_type.
    InvalidObject,

    /// Either:
    /// * Missing required field
    /// * An optional field is required to be set for a specific state, e.g. PaymentObject requires
    ///   sender's kyc_data (which is an optional field for PaymentActorObject) when sender init
    ///   the PaymentObject.
    MissingField,

    /// A field is unknown for an object.
    UnknownField,

    /// Invalid/unsupported command_type.
    UnknownCommandType,

    /// * Invalid / unknown enum field values.
    /// * UUID field value does not match UUID format.
    /// * Payment actor address is not a valid DIP-5 account identifier.
    /// * Currency field value is not a valid Diem currency code for the connected network.
    InvalidFieldValue,

    /// The HTTP request sender is not the right actor to send the payment object. For example, if
    /// the actor receiver sends a new command with payment object change that should be done by
    /// actor sender.
    InvalidCommandProducer,

    /// could not find command by reference_id for a non-initial state command object; for example,
    /// actor receiver received a payment command object that actor sender status is
    /// `ready_for_settlement`, but receiver could not find any command object by the reference id.
    InvalidInitialOrPriorNotFound,

    /// PaymentActionObject#amount is under travel rule threshold, no kyc needed for the
    /// transaction
    NoKycNeeded,

    /// Either:
    /// * Field recipient_signature value is not hex-encoded bytes.
    /// * Field recipient_signature value is an invalid signature.
    InvalidRecipientSignature,

    /// * The DIP-5 account identifier address in the command object is not HTTP request sender’s
    ///   address or receiver’s address. For payment object it is sender.address or
    ///   receiver.address.
    /// * Could not find on-chain account by an DIP-5 account identifier address in command object
    ///   address.
    UnknownAddress,
    /// * Command object is in conflict with another different command object by cid, likely a cid
    ///   is reused for different command object.
    /// * Failed to acquire lock for the command object by the reference_id.
    Conflict,

    /// Field payment.action.currency value is a valid Diem currency code, but it is not supported
    /// or acceptable by the receiver VASP.
    UnsupportedCurrency,

    /// * Could not find data by the original_payment_reference_id if the sender set it.
    /// * The status of the original payment object found by original_payment_reference_id is
    ///   aborted instead of ready_for_settlement.
    InvalidOriginalPaymentReferenceId,

    /// Overwrite a write-once/immutable field value
    /// * Overwrite a field that can only be written once.
    /// * Overwrite an immutable field (field can only be set in initial command object), e.g.
    ///   `original_payment_reference_id`).
    /// * Overwrite opponent payment actor's fields.
    InvalidOverwrite,

    /// As we only allow one actor action at a time, and the next states for a given command object
    /// state are limited to specific states. This error indicates the new payment object state is
    /// not valid according to the current object state. For example: VASP A sends RSOFT to VASP B,
    /// VASP B should send the next payment object with ABORT, or SSOFTSEND; VASP A should respond
    /// to this error code if VASP B sends payment object state SSOFT.
    InvalidTransition,

    #[serde(other)]
    /// Unknown Error Code
    Unknown,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct OffChainError {
    #[serde(rename = "type")]
    error_type: OffChainErrorType,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,
    code: ErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "command_type", content = "command")]
pub enum Command {
    PaymentCommand(PaymentCommandObject),
    FundPullPreApprovalCommand,
}

#[derive(Deserialize, Serialize)]
pub struct PaymentCommandObject {
    #[serde(deserialize_with = "ObjectType::deserialize_payment")]
    #[serde(rename = "_ObjectType")]
    object_type: ObjectType,
    payment: PaymentObject,
}

impl PaymentCommandObject {
    pub fn new(payment: PaymentObject) -> Self {
        Self {
            object_type: ObjectType::PaymentCommand,
            payment,
        }
    }

    pub fn payment(&self) -> &PaymentObject {
        &self.payment
    }

    pub fn into_payment(self) -> PaymentObject {
        self.payment
    }
}

/// A `PaymentActorObject` represents a participant in a payment - either sender or receiver. It
/// also includes the status of the actor, indicates missing information or willingness to settle
/// or abort the payment, and the Know-Your-Customer information of the customer involved in the
/// payment.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PaymentActorObject {
    /// Address of the sender/receiver account. Addresses may be single use or valid for a limited
    /// time, and therefore VASPs should not rely on them remaining stable across time or different
    /// VASP addresses. The addresses are encoded using bech32. The bech32 address encodes both the
    /// address of the VASP as well as the specific user's subaddress. They should be no longer
    /// than 80 characters. Mandatory and immutable. For Diem addresses, refer to the "account
    /// identifier" section in DIP-5 for format.
    pub address: Box<str>,

    /// The KYC data for this account. This field is optional but immutable once it is set.
    pub kyc_data: Option<KycDataObject>,

    /// Status of the payment from the perspective of this actor. This field can only be set by the
    /// respective sender/receiver VASP and represents the status on the sender/receiver VASP side.
    /// This field is mandatory by this respective actor (either sender or receiver side) and
    /// mutable. Note that in the first request (which is initiated by the sender), the receiver
    /// status should be set to `None`.
    pub status: StatusObject,

    /// Can be specified by the respective VASP to hold metadata that the sender/receiver VASP
    /// wishes to associate with this payment. It may be set to an empty list (i.e. `[]`). New
    /// `metadata` elements may be appended to the `metadata` list via subsequent commands on an
    /// object.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub metadata: Vec<String>,

    /// Freeform KYC data. If a soft-match occurs, this field can be used to specify additional KYC
    /// data which can be used to clear the soft-match. It is suggested that this data be JSON,
    /// XML, or another human-readable form.
    pub additional_kyc_data: Option<String>,
}

impl PaymentActorObject {
    pub fn status(&self) -> &StatusObject {
        &self.status
    }

    pub fn kyc_data(&self) -> Option<&KycDataObject> {
        self.kyc_data.as_ref()
    }

    pub fn additional_kyc_data(&self) -> Option<&str> {
        self.additional_kyc_data.as_deref()
    }

    pub fn validate_write_once_fields(&self, prior: &Self) -> Result<(), WriteOnceError> {
        if self.address != prior.address {
            return Err(WriteOnceError);
        }

        if prior.kyc_data.is_some() && prior.kyc_data != self.kyc_data {
            return Err(WriteOnceError);
        }

        if !self.metadata.starts_with(&prior.metadata) {
            return Err(WriteOnceError);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Charge,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PaymentActionObject {
    /// Amount of the transfer. Base units are the same as for on-chain transactions for this
    /// currency. For example, if DiemUSD is represented on-chain where “1” equals 1e-6 dollars,
    /// then “1” equals the same amount here. For any currency, the on-chain mapping must be used
    /// for amounts.
    pub amount: u64,

    /// One of the supported on-chain currency types - ex. XUS, etc.
    // TODO Should be an enum per https://dip.diem.com/dip-1/#paymentactionobject
    pub currency: String,

    /// Populated in the request. This value indicates the requested action to perform, and the
    /// only valid value is charge.
    pub action: ActionType,

    /// [Unix time](https://en.wikipedia.org/wiki/Unix_time) indicating the time that the payment
    /// Command was created.
    pub timestamp: u64,
}

/// Some fields are immutable after they are defined once. Others can be updated multiple times
/// (see below). Updating immutable fields with a different value results in a Command error.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PaymentObject {
    /// Information about the sender in this payment
    pub sender: PaymentActorObject,

    /// Information about the receiver in this payment
    pub receiver: PaymentActorObject,

    /// Unique reference ID of this payment on the payment initiator VASP (the VASP which
    /// originally created this payment Object). This value should be globally unique. This field
    /// is mandatory on payment creation and immutable after that. We recommend using a 128 bits
    /// long UUID according to RFC4122 with "-"'s included.
    pub reference_id: Uuid,

    /// Used to refer an old payment known to the other VASP. For example, used for refunds. The
    /// reference ID of the original payment will be placed into this field. This field is
    /// mandatory on refund and immutable
    pub originial_payment_reference_id: Option<Uuid>,

    /// Signature of the recipient of this transaction encoded in hex. The is signed with the
    /// compliance key of the recipient VASP and is used for on-chain attestation from the
    /// recipient party. This may be omitted on blockchains which do not require on-chain
    /// attestation.
    pub recipient_signature: Option<String>,

    /// Number of cryptocurrency + currency type (XUS, etc.)1 + type of action to take. This field is mandatory and immutable
    pub action: PaymentActionObject,

    /// Description of the payment. To be displayed to the user. Unicode utf-8 encoded max length
    /// of 255 characters. This field is optional but can only be written once.
    pub description: Option<String>,
}

impl PaymentObject {
    pub fn sender(&self) -> &PaymentActorObject {
        &self.sender
    }

    pub fn receiver(&self) -> &PaymentActorObject {
        &self.receiver
    }

    pub fn reference_id(&self) -> Uuid {
        self.reference_id
    }

    pub fn actor_object_by_actor(&self, actor: Actor) -> &PaymentActorObject {
        match actor {
            Actor::Sender => self.sender(),
            Actor::Receiver => self.receiver(),
        }
    }

    pub fn recipient_signature(&self) -> Option<&str> {
        self.recipient_signature.as_deref()
    }

    pub fn validate_write_once_fields(&self, prior: &Self) -> Result<(), WriteOnceError> {
        self.sender.validate_write_once_fields(&prior.sender)?;
        self.receiver.validate_write_once_fields(&prior.receiver)?;
        if self.reference_id != prior.reference_id {
            return Err(WriteOnceError);
        }

        if self.originial_payment_reference_id != prior.originial_payment_reference_id {
            return Err(WriteOnceError);
        }

        if self.action != prior.action {
            return Err(WriteOnceError);
        }

        if prior.description.is_some() && prior.description != self.description {
            return Err(WriteOnceError);
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct StatusObject {
    /// Status of the payment from the perspective of this actor. This field can only be set by the
    /// respective sender/receiver VASP and represents the status on the sender/receiver VASP side.
    /// This field is mandatory by this respective actor (either sender or receiver side) and
    /// mutable.
    pub status: Status,

    /// In the case of an `abort` status, this field may be used to describe the reason for the
    /// abort. Represents the error code of the corresponding error.
    pub abort_code: Option<AbortCode>,

    /// Additional details about this error. To be used only when `abort_code` is populated.
    pub abort_message: Option<String>,
}

impl StatusObject {
    pub fn status(&self) -> Status {
        self.status
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// No status is yet set from this actor.
    None,

    /// KYC data about the subaddresses is required by this actor.
    NeedsKycData,

    /// Transaction is ready for settlement according to this actor (i.e. the requried
    /// signatures/KYC data has been provided.
    ReadyForSettlement,

    /// Indicates the actor wishes to abort this payment, instaed of settling it.
    Abort,

    /// Actor's KYC data resulted in a soft-match, request additional KYC data.
    SoftMatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AbortCode {
    /// The payment is rejected. It should not be used in the `original_payment_reference_id` field
    /// of a new payment
    Rejected,
}

/// Represents a national ID.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct NationalIdObject {
    /// Indicates the national ID value - for example, a social security number
    pub id_value: String,

    /// Two-letter country code (https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2)
    pub country: Option<String>,

    /// Indicates the type of the ID
    #[serde(rename = "type")]
    pub id_type: Option<String>,
}

/// Represents a physical address
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct AddressObject {
    /// The city, district, suburb, town, or village
    pub city: Option<String>,

    /// Two-letter country code (https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2)
    pub country: Option<String>,

    /// Address line 1
    pub line1: Option<String>,

    /// Address line 2 - apartment, unit, etc.
    pub line2: Option<String>,

    /// ZIP or postal code
    pub postal_code: Option<String>,

    /// State, county, province, region.
    pub state: Option<String>,
}

/// A `KycDataObject` represents the required information for a single subaddress. Proof of
/// non-repudiation is provided by the signatures included in the JWS payloads. The only mandatory
/// fields are `payload_version` and `type`. All other fields are optional from the point of view of
/// the protocol -- however they may need to be included for another VASP to be ready to settle the
/// payment.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct KycDataObject {
    /// Version identifier to allow modifications to KYC data Object without needing to bump
    /// version of entire API set. Set to 1
    payload_version: KycDataObjectVersion,

    pub kyc_data_type: KycDataObjectType,

    /// Legal given name of the user for which this KYC data Object applies.
    pub given_name: Option<String>,

    /// Legal surname of the user for which this KYC data Object applies.
    pub surname: Option<String>,

    /// Physical address data for this account
    pub address: Option<AddressObject>,

    /// Date of birth for the holder of this account. Specified as an ISO 8601 calendar date
    /// format: https://en.wikipedia.org/wiki/ISO_8601
    pub dob: Option<String>,

    /// Place of birth for this user. line1 and line2 fields should not be populated for this usage
    /// of the address Object
    pub place_of_birth: Option<String>,

    /// National ID information for the holder of this account
    pub national_id: Option<NationalIdObject>,

    /// Name of the legal entity. Used when subaddress represents a legal entity rather than an
    /// individual. KYCDataObject should only include one of legal_entity_name OR
    /// given_name/surname
    pub legal_entity_name: Option<String>,
}

impl KycDataObject {
    pub fn new_entity() -> Self {
        Self {
            payload_version: KycDataObjectVersion::V1,
            kyc_data_type: KycDataObjectType::Entity,
            given_name: None,
            surname: None,
            address: None,
            dob: None,
            place_of_birth: None,
            national_id: None,
            legal_entity_name: None,
        }
    }

    pub fn new_individual() -> Self {
        Self {
            payload_version: KycDataObjectVersion::V1,
            kyc_data_type: KycDataObjectType::Individual,
            given_name: None,
            surname: None,
            address: None,
            dob: None,
            place_of_birth: None,
            national_id: None,
            legal_entity_name: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KycDataObjectType {
    Individual,
    Entity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum KycDataObjectVersion {
    V1 = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteOnceError;

#[cfg(test)]
mod tests {
    use super::{KycDataObjectType, KycDataObjectVersion};
    use serde_json::json;

    #[test]
    fn kyc_data_object_type() {
        use KycDataObjectType::*;

        let variants = [(Individual, "individual"), (Entity, "entity")];
        for (variant, s) in &variants {
            let json = json! { s };
            assert_eq!(serde_json::to_value(variant).unwrap(), json);
            assert_eq!(
                serde_json::from_value::<KycDataObjectType>(json).unwrap(),
                *variant
            );
        }

        let invalid = json! { "Organization" };
        serde_json::from_value::<KycDataObjectType>(invalid).unwrap_err();
    }

    #[test]
    fn kyc_data_object_version() {
        let v1_json = json! { 1 };

        let v1: KycDataObjectVersion = serde_json::from_value(v1_json.clone()).unwrap();
        assert_eq!(serde_json::to_value(&v1).unwrap(), v1_json);

        let invalid_version = json! { 52 };
        serde_json::from_value::<KycDataObjectVersion>(invalid_version).unwrap_err();

        let invalid_type = json! { "1" };
        serde_json::from_value::<KycDataObjectVersion>(invalid_type).unwrap_err();
    }
}
