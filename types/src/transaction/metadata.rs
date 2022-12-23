// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file implements Diem transaction metadata types to allow
//! easy parsing and introspection into metadata, whether the transaction
//! is using regular subaddressing, is subject to travel rule or corresponds
//! to an on-chain payment refund.

use serde::{Deserialize, Serialize};

/// List of all supported metadata types
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Metadata {
    Undefined,
    GeneralMetadata(GeneralMetadata),
    TravelRuleMetadata(TravelRuleMetadata),
    UnstructuredBytesMetadata(UnstructuredBytesMetadata),
    RefundMetadata(RefundMetadata),
    CoinTradeMetadata(CoinTradeMetadata),
    PaymentMetadata(PaymentMetadata),
}

/// List of supported transaction metadata format versions for regular
/// addressing with optional subaddressing or refund reference
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GeneralMetadata {
    GeneralMetadataVersion0(GeneralMetadataV0),
}

/// Transaction metadata for regular addressing with optional subaddressing
/// or refunded transaction reference
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeneralMetadataV0 {
    /// Subaddress to which the funds are being sent
    #[serde(with = "serde_bytes")]
    to_subaddress: Option<Vec<u8>>,
    /// Subaddress from which the funds are being sent
    #[serde(with = "serde_bytes")]
    from_subaddress: Option<Vec<u8>>,
    /// In the case of refunds, referenced_event refers to the event sequence
    /// number of the sender’s original sent payment event.
    /// Since refunds are just another form of P2P transfer, the referenced
    /// event field allows a refunded payment to refer back to the original
    /// payment
    referenced_event: Option<u64>,
}

impl GeneralMetadataV0 {
    pub fn new(
        to_subaddress: Option<Vec<u8>>,
        from_subaddress: Option<Vec<u8>>,
        referenced_event: Option<u64>,
    ) -> Self {
        GeneralMetadataV0 {
            to_subaddress,
            from_subaddress,
            referenced_event,
        }
    }

    pub fn to_subaddress(&self) -> &Option<Vec<u8>> {
        &self.to_subaddress
    }

    pub fn from_subaddress(&self) -> &Option<Vec<u8>> {
        &self.from_subaddress
    }

    pub fn referenced_event(&self) -> &Option<u64> {
        &self.referenced_event
    }
}

/// List of supported transaction metadata format versions for transactions
/// subject to travel rule
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TravelRuleMetadata {
    TravelRuleMetadataVersion0(TravelRuleMetadataV0),
}

/// Transaction metadata format for transactions subject to travel rule
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TravelRuleMetadataV0 {
    /// Off-chain reference_id.  Used when off-chain APIs are used.
    /// Specifies the off-chain reference ID that was agreed upon in off-chain APIs.
    pub off_chain_reference_id: Option<String>,
}

/// Opaque binary transaction metadata
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UnstructuredBytesMetadata {
    /// Unstructured byte vector metadata
    #[serde(with = "serde_bytes")]
    pub metadata: Option<Vec<u8>>,
}

/// List of supported transaction metadata format versions for refund transaction
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RefundMetadata {
    RefundMetadataV0(RefundMetadataV0),
}

/// Transaction metadata format for transactions subject to refund transaction
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RefundMetadataV0 {
    /// Transaction version that is refunded
    pub transaction_version: u64,
    /// The reason of the refund
    pub reason: RefundReason,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RefundReason {
    OtherReason,
    InvalidSubaddress,
    UserInitiatedPartialRefund,
    UserInitiatedFullRefund,
    InvalidReferenceId,
}

/// List of supported transaction metadata format versions for coin trade transaction
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CoinTradeMetadata {
    CoinTradeMetadataV0(CoinTradeMetadataV0),
}

/// Transaction metadata format for coin trades (purchases/sells)
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoinTradeMetadataV0 {
    /// A list of trade_ids this transaction wants to settle
    pub trade_ids: Vec<String>,
}

/// List of supported transaction metadata format versions for transactions for payments
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PaymentMetadata {
    PaymentMetadataVersion0(PaymentMetadataV0),
}

/// Transaction metadata format for transactions for payments
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PaymentMetadataV0 {
    /// Reference ID needed for off-chain reference ID exchange.
    pub reference_id: [u8; 16],
}
