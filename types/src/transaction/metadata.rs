// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file implements Libra transaction metadata types to allow
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
    off_chain_reference_id: Option<String>,
}

/// Opaque binary transaction metadata
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UnstructuredBytesMetadata {
    /// Unstructured byte vector metadata
    #[serde(with = "serde_bytes")]
    metadata: Option<Vec<u8>>,
}
